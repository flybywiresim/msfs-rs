#![allow(clippy::too_many_arguments)]

use crate::sys;

use std::{any::TypeId, collections::HashMap, ffi, pin::Pin, ptr};

pub use sys::SIMCONNECT_OBJECT_ID_USER;

pub use msfs_derive::sim_connect_client_data_definition as client_data_definition;
pub use msfs_derive::sim_connect_data_definition as data_definition;
pub use msfs_derive::sim_connect_facility_definition as facility_definition;

pub type DataXYZ = sys::SIMCONNECT_DATA_XYZ;
pub type InitPosition = sys::SIMCONNECT_DATA_INITPOSITION;

/// A trait implemented by the `data_definition` attribute.
pub trait DataDefinition: 'static {
    #[doc(hidden)]
    const DEFINITIONS: &'static [(&'static str, &'static str, f32, sys::SIMCONNECT_DATATYPE)];
}

/// A trait implemented by the `client_data_definition` attribute.
pub trait ClientDataDefinition: 'static {
    #[doc(hidden)]
    fn get_definitions() -> Vec<(usize, usize, f32)>;
}

/// A trait implemented by the `facility_definition` attribute.
pub trait FacilityDefinition: 'static {
    #[doc(hidden)]
    type RawType;

    #[doc(hidden)]
    fn add_facility_definitions(
        handle: sys::HANDLE,
        define_id: sys::SIMCONNECT_DATA_DEFINITION_ID,
    ) -> Result<()>;
}

/// Rusty HRESULT wrapper.
#[allow(dead_code)]
#[derive(Debug)]
pub struct HResult(sys::HRESULT);
impl std::fmt::Display for HResult {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, fmt)
    }
}
impl std::error::Error for HResult {}

pub type Result<T> = std::result::Result<T, HResult>;
#[inline(always)]
pub fn map_err(result: sys::HRESULT) -> Result<()> {
    if result >= 0 {
        Ok(())
    } else {
        Err(HResult(result))
    }
}

type SimConnectCallback<'a> = dyn FnMut(&mut SimConnect, SimConnectRecv) + 'a;

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect<'a> {
    handle: sys::HANDLE,
    callback: Box<SimConnectCallback<'a>>,
    data_definitions: HashMap<TypeId, sys::SIMCONNECT_DATA_DEFINITION_ID>,
    client_data_definitions: HashMap<TypeId, sys::SIMCONNECT_CLIENT_DATA_DEFINITION_ID>,
    facility_data_definitions: HashMap<
        TypeId,
        (
            sys::SIMCONNECT_DATA_REQUEST_ID,
            sys::SIMCONNECT_DATA_DEFINITION_ID,
        ),
    >,
    event_id_counter: sys::DWORD,
    client_data_id_counter: sys::DWORD,
}

impl std::fmt::Debug for SimConnect<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("SimConnect").finish()
    }
}

impl<'a> SimConnect<'a> {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open<F>(name: &str, callback: F) -> Result<Pin<Box<SimConnect<'a>>>>
    where
        F: FnMut(&mut SimConnect, SimConnectRecv) + 'a,
    {
        unsafe {
            let mut handle = 0;
            let name = std::ffi::CString::new(name).unwrap();
            map_err(sys::SimConnect_Open(
                &mut handle,
                name.as_ptr(),
                std::ptr::null_mut(),
                0,
                0,
                0,
            ))?;
            debug_assert!(handle != 0);
            let mut sim = Box::pin(SimConnect {
                handle,
                callback: Box::new(callback),
                data_definitions: HashMap::new(),
                client_data_definitions: HashMap::new(),
                facility_data_definitions: HashMap::new(),
                event_id_counter: 0,
                client_data_id_counter: 0,
            });
            sim.call_dispatch()?;
            Ok(sim)
        }
    }

    /// Used to process the next SimConnect message received. Only needed when not using the gauge API.
    pub fn call_dispatch(&mut self) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_CallDispatch(
                self.handle,
                Some(dispatch_cb),
                self as *mut SimConnect as *mut std::ffi::c_void,
            ))
        }
    }

    fn get_define_id<T: DataDefinition>(&mut self) -> Result<sys::SIMCONNECT_DATA_DEFINITION_ID> {
        let handle = self.handle;
        SimConnect::get_id::<T, _, _>(&mut self.data_definitions, |define_id| {
            /*
            unsafe {
                map_err(sys::SimConnect_ClearDataDefinition(handle, define_id))?;
            }
            */
            for (datum_name, units_type, epsilon, datatype) in T::DEFINITIONS {
                let datum_name = std::ffi::CString::new(*datum_name).unwrap();
                let units_type = std::ffi::CString::new(*units_type).unwrap();
                unsafe {
                    map_err(sys::SimConnect_AddToDataDefinition(
                        handle,
                        define_id,
                        datum_name.as_ptr(),
                        units_type.as_ptr(),
                        *datatype,
                        *epsilon,
                        sys::SIMCONNECT_UNUSED,
                    ))?;
                }
            }
            Ok(())
        })
    }

    fn get_client_data_define_id<T: ClientDataDefinition>(
        &mut self,
    ) -> Result<sys::SIMCONNECT_CLIENT_DATA_DEFINITION_ID> {
        let handle = self.handle;
        SimConnect::get_id::<T, _, _>(&mut self.client_data_definitions, |define_id| {
            /*
            unsafe {
                map_err(sys::SimConnect_ClearClientDataDefinition(handle, define_id))?;
            }
            */

            // Rust may reorder fields, so padding has to be calculated as min of
            // all fields instead of the last field.
            let mut padding = usize::MAX;
            for (offset, size, epsilon) in T::get_definitions() {
                padding = padding.min(std::mem::size_of::<T>() - (offset + size));
                unsafe {
                    map_err(sys::SimConnect_AddToClientDataDefinition(
                        handle,
                        define_id,
                        offset as sys::DWORD,
                        size as sys::DWORD,
                        epsilon,
                        sys::SIMCONNECT_UNUSED,
                    ))?;
                }
            }
            if padding > 0 && padding != usize::MAX {
                unsafe {
                    map_err(sys::SimConnect_AddToClientDataDefinition(
                        handle,
                        define_id,
                        (std::mem::size_of::<T>() - padding) as sys::DWORD,
                        padding as sys::DWORD,
                        0.0,
                        sys::SIMCONNECT_UNUSED,
                    ))?;
                }
            }
            Ok(())
        })
    }

    fn get_facility_define_id<T: FacilityDefinition>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<(
        sys::SIMCONNECT_DATA_REQUEST_ID,
        sys::SIMCONNECT_DATA_DEFINITION_ID,
    )> {
        let handle = self.handle;

        self.get_facility_id::<T, _>(request_id, |define_id| {
            T::add_facility_definitions(handle, define_id)
        })
    }

    fn get_id<T: 'static, U: std::convert::TryFrom<usize> + Copy, F: Fn(U) -> Result<()>>(
        map: &mut HashMap<TypeId, U>,
        insert_fn: F,
    ) -> Result<U> {
        let key = TypeId::of::<T>();
        let maybe_id = U::try_from(map.len()).unwrap_or_else(|_| unreachable!());
        match map.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                insert_fn(maybe_id)?;
                entry.insert(maybe_id);
                Ok(maybe_id)
            }
            std::collections::hash_map::Entry::Occupied(entry) => Ok(*entry.get()),
        }
    }

    fn get_facility_id<T: 'static, F: Fn(sys::SIMCONNECT_DATA_DEFINITION_ID) -> Result<()>>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
        insert_fn: F,
    ) -> Result<(
        sys::SIMCONNECT_DATA_REQUEST_ID,
        sys::SIMCONNECT_DATA_DEFINITION_ID,
    )> {
        let key = TypeId::of::<T>();
        let maybe_id =
            sys::SIMCONNECT_DATA_DEFINITION_ID(self.facility_data_definitions.len() as u32);

        match self.facility_data_definitions.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                insert_fn(maybe_id)?;
                entry.insert((request_id, maybe_id));
                Ok((request_id, maybe_id))
            }
            std::collections::hash_map::Entry::Occupied(entry) => Ok(*entry.get()),
        }
    }

    /// Make changes to the data properties of an object.
    pub fn set_data_on_sim_object<T: DataDefinition>(
        &mut self,
        object_id: sys::SIMCONNECT_OBJECT_ID,
        data: &T,
    ) -> Result<()> {
        let define_id = self.get_define_id::<T>()?;
        unsafe {
            map_err(sys::SimConnect_SetDataOnSimObject(
                self.handle,
                define_id,
                object_id,
                0,
                0,
                std::mem::size_of_val(data) as sys::DWORD,
                data as *const T as *mut std::ffi::c_void,
            ))
        }
    }

    /// Retrieve information about simulation objects of a given type that are
    /// within a specified radius of the user's aircraft.
    pub fn request_data_on_sim_object_type<T: DataDefinition>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
        radius: sys::DWORD,
        r#type: sys::SIMCONNECT_SIMOBJECT_TYPE,
    ) -> Result<()> {
        let define_id = self.get_define_id::<T>()?;
        unsafe {
            map_err(sys::SimConnect_RequestDataOnSimObjectType(
                self.handle,
                request_id,
                define_id,
                radius,
                r#type,
            ))
        }
    }

    /// Request when the SimConnect client is to receive data values for a specific object
    pub fn request_data_on_sim_object<T: DataDefinition>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
        object_id: sys::SIMCONNECT_OBJECT_ID,
        period: Period,
    ) -> Result<()> {
        let define_id = self.get_define_id::<T>()?;

        unsafe {
            map_err(sys::SimConnect_RequestDataOnSimObject(
                self.handle,
                request_id,
                define_id,
                object_id,
                period as sys::SIMCONNECT_PERIOD,
                sys::SIMCONNECT_DATA_REQUEST_FLAG_CHANGED,
                0,
                0,
                0,
            ))
        }
    }

    /// Map a Prepar3D event to a specific ID. If `mask` is true, the sim itself
    /// will ignore the event, and only this SimConnect instance will receive it.
    pub fn map_client_event_to_sim_event(
        &mut self,
        event_name: &str,
        mask: bool,
    ) -> Result<sys::DWORD> {
        let event_id = self.event_id_counter;
        self.event_id_counter += 1;
        let event_name = std::ffi::CString::new(event_name).unwrap();

        unsafe {
            map_err(sys::SimConnect_MapClientEventToSimEvent(
                self.handle,
                event_id,
                event_name.as_ptr(),
            ))?;

            map_err(sys::SimConnect_AddClientEventToNotificationGroup(
                self.handle,
                0,
                event_id,
                mask.into(),
            ))?;

            map_err(sys::SimConnect_SetNotificationGroupPriority(
                self.handle,
                0,
                sys::SIMCONNECT_GROUP_PRIORITY_HIGHEST_MASKABLE,
            ))?;
        }
        Ok(event_id)
    }

    /// Trigger an event, previously mapped with `map_client_event_to_sim_event`
    pub fn transmit_client_event(
        &mut self,
        object_id: sys::SIMCONNECT_OBJECT_ID,
        event_id: sys::DWORD,
        data: sys::DWORD,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_TransmitClientEvent(
                self.handle,
                object_id,
                event_id,
                data,
                0,
                0,
            ))
        }
    }

    pub fn transmit_client_event_ex1(
        &mut self,
        object_id: sys::SIMCONNECT_OBJECT_ID,
        event_id: sys::DWORD,
        data: [sys::DWORD; 5],
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_TransmitClientEvent_EX1(
                self.handle,
                object_id,
                event_id,
                0,
                0,
                data[0],
                data[1],
                data[2],
                data[3],
                data[4],
            ))
        }
    }

    fn get_client_data_id(&mut self, name: &str) -> Result<sys::SIMCONNECT_CLIENT_DATA_ID> {
        let client_id = self.client_data_id_counter;
        self.client_data_id_counter += 1;
        let name = std::ffi::CString::new(name).unwrap();

        unsafe {
            map_err(sys::SimConnect_MapClientDataNameToID(
                self.handle,
                name.as_ptr(),
                client_id,
            ))?;
        }
        Ok(client_id)
    }

    /// Allocate a region of memory in the sim with the given `name`. Other
    /// SimConnect modules can use the `name` to read data from this memory
    /// using `request_client_data`. This memory cannot be deallocated.
    pub fn create_client_data<T: ClientDataDefinition>(
        &mut self,
        name: &str,
    ) -> Result<ClientDataArea<T>> {
        let client_id = self.get_client_data_id(name)?;
        unsafe {
            map_err(sys::SimConnect_CreateClientData(
                self.handle,
                client_id,
                std::mem::size_of::<T>() as sys::DWORD,
                0,
            ))?;
        }
        Ok(ClientDataArea {
            client_id,
            phantom: std::marker::PhantomData,
        })
    }

    /// Create a handle to a region of memory allocated by another module with
    /// the given `name`.
    pub fn get_client_area<T: ClientDataDefinition>(
        &mut self,
        name: &str,
    ) -> Result<ClientDataArea<T>> {
        let client_id = self.get_client_data_id(name)?;
        Ok(ClientDataArea {
            client_id,
            phantom: std::marker::PhantomData,
        })
    }

    /// Request a pre-allocated region of memory from the sim with the given
    /// `name`. A module must have already used `create_client_data` to
    /// allocate this memory.
    pub fn request_client_data<T: ClientDataDefinition>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
        name: &str,
    ) -> Result<()> {
        let define_id = self.get_client_data_define_id::<T>()?;
        let client_id = self.get_client_data_id(name)?;
        unsafe {
            map_err(sys::SimConnect_RequestClientData(
                self.handle,
                client_id,
                request_id,
                define_id,
                sys::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET,
                sys::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED,
                0,
                0,
                0,
            ))?;
        }
        Ok(())
    }

    /// Set the data of an area acquired by `create_client_data` or
    /// `get_client_data`.
    pub fn set_client_data<T: ClientDataDefinition>(
        &mut self,
        area: &ClientDataArea<T>,
        data: &T,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_SetClientData(
                self.handle,
                area.client_id,
                self.get_client_data_define_id::<T>()?,
                0,
                0,
                std::mem::size_of::<T>() as sys::DWORD,
                data as *const _ as *mut std::ffi::c_void,
            ))?;
        }
        Ok(())
    }

    pub fn ai_create_non_atc_aircraft(
        &mut self,
        container_title: &str,
        tail_number: &str,
        init_position: sys::SIMCONNECT_DATA_INITPOSITION,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<()> {
        let container_title = std::ffi::CString::new(container_title).unwrap();
        let tail_number = std::ffi::CString::new(tail_number).unwrap();

        unsafe {
            map_err(sys::SimConnect_AICreateNonATCAircraft(
                self.handle,
                container_title.as_ptr(),
                tail_number.as_ptr(),
                init_position,
                request_id,
            ))?;
        }
        Ok(())
    }

    pub fn ai_create_parked_atc_aircraft(
        &mut self,
        container_title: &str,
        tail_number: &str,
        icao: &str,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<()> {
        let container_title = std::ffi::CString::new(container_title).unwrap();
        let tail_number = std::ffi::CString::new(tail_number).unwrap();
        let icao = std::ffi::CString::new(icao).unwrap();

        unsafe {
            map_err(sys::SimConnect_AICreateParkedATCAircraft(
                self.handle,
                container_title.as_ptr(),
                tail_number.as_ptr(),
                icao.as_ptr(),
                request_id,
            ))?;
        }
        Ok(())
    }

    pub fn ai_remove_object(
        &mut self,
        object_id: sys::SIMCONNECT_OBJECT_ID,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_AIRemoveObject(
                self.handle,
                object_id,
                request_id,
            ))?;
        }
        Ok(())
    }

    pub fn subscribe_to_system_event(&mut self, system_event_name: &str) -> Result<sys::DWORD> {
        let event_id = self.event_id_counter;
        self.event_id_counter += 1;
        let system_event_name = std::ffi::CString::new(system_event_name).unwrap();

        unsafe {
            map_err(sys::SimConnect_SubscribeToSystemEvent(
                self.handle,
                event_id,
                system_event_name.as_ptr(),
            ))?;
        }
        Ok(event_id)
    }

    pub fn unsubscribe_from_system_event(
        &mut self,
        event_id: sys::SIMCONNECT_CLIENT_EVENT_ID,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_UnsubscribeFromSystemEvent(
                self.handle,
                event_id,
            ))?;
        }
        Ok(())
    }

    pub fn set_system_event_state(
        &mut self,
        event_id: sys::SIMCONNECT_CLIENT_EVENT_ID,
        on: bool,
    ) -> Result<()> {
        let state = on.into();
        unsafe {
            map_err(sys::SimConnect_SetSystemEventState(
                self.handle,
                event_id,
                state,
            ))?;
        }
        Ok(())
    }

    /// Load a .FLT file from disk
    pub fn load_flight(&mut self, flight_file_path: &str) -> Result<()> {
        let flight_file_path = std::ffi::CString::new(flight_file_path).unwrap();

        unsafe {
            map_err(sys::SimConnect_FlightLoad(
                self.handle,
                flight_file_path.as_ptr(),
            ))?;
        }
        Ok(())
    }

    /// Save the current sim state to a .FLT file
    pub fn save_flight(
        &mut self,
        flight_file_path: &str,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let flight_file_path = std::ffi::CString::new(flight_file_path).unwrap();
        let title = title.map(|x| std::ffi::CString::new(x).unwrap());
        let description = description.map(|x| std::ffi::CString::new(x).unwrap());

        unsafe {
            map_err(sys::SimConnect_FlightSave(
                self.handle,
                flight_file_path.as_ptr(),
                title
                    .as_ref()
                    .map(|x| x.as_ptr())
                    .unwrap_or(std::ptr::null()),
                description
                    .as_ref()
                    .map(|x| x.as_ptr())
                    .unwrap_or(std::ptr::null()),
                0,
            ))?;
        }
        Ok(())
    }

    /// Load a .PLN file from disk
    pub fn load_flight_plan(&mut self, flight_plan_file_path: &str) -> Result<()> {
        let flight_plan_file_path = std::ffi::CString::new(flight_plan_file_path).unwrap();

        unsafe {
            map_err(sys::SimConnect_FlightPlanLoad(
                self.handle,
                flight_plan_file_path.as_ptr(),
            ))?;
        }
        Ok(())
    }

    // Request information about facilities of a given type within the reality bubble cache.
    pub fn request_facilities_list_ex1(
        &mut self,
        facility_list_type: sys::SIMCONNECT_FACILITY_LIST_TYPE,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_RequestFacilitiesList_EX1(
                self.handle,
                facility_list_type,
                request_id,
            ))?;
        }

        Ok(())
    }

    // Request information about all facilities of a given type.
    pub fn request_facilities_list(
        &mut self,
        facility_list_type: sys::SIMCONNECT_FACILITY_LIST_TYPE,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
    ) -> Result<()> {
        unsafe {
            map_err(sys::SimConnect_RequestFacilitiesList(
                self.handle,
                facility_list_type,
                request_id,
            ))?;
        }

        Ok(())
    }

    // Request information about a specific facility.
    pub fn request_facility_data<T: FacilityDefinition>(
        &mut self,
        request_id: sys::SIMCONNECT_DATA_REQUEST_ID,
        icao: &str,
        region: Option<&str>,
    ) -> Result<()> {
        let (_, define_id) = self.get_facility_define_id::<T>(request_id)?;

        let icao = ffi::CString::new(icao).unwrap();
        let region = match region {
            Some(x) => {
                let c_str = ffi::CString::new(x).unwrap();
                c_str.as_ptr()
            }
            None => ptr::null(),
        };

        unsafe {
            map_err(sys::SimConnect_RequestFacilityData(
                self.handle,
                define_id,
                request_id,
                icao.as_ptr(),
                region,
            ))?;
        }

        Ok(())
    }
}

impl Drop for SimConnect<'_> {
    fn drop(&mut self) {
        unsafe {
            map_err(sys::SimConnect_Close(self.handle)).expect("SimConnect_Close");
        }
    }
}

macro_rules! recv {
    ($V:ident) => {
        $V! {
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_EXCEPTION,
                SIMCONNECT_RECV_EXCEPTION,
                Exception
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_OPEN,
                SIMCONNECT_RECV_OPEN,
                Open
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_QUIT,
                SIMCONNECT_RECV_QUIT,
                Quit
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_EVENT,
                SIMCONNECT_RECV_EVENT,
                Event
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_EVENT_EX1,
                SIMCONNECT_RECV_EVENT_EX1,
                EventEx1
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_SIMOBJECT_DATA,
                SIMCONNECT_RECV_SIMOBJECT_DATA,
                SimObjectData
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_CLIENT_DATA,
                SIMCONNECT_RECV_CLIENT_DATA,
                ClientData
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_ASSIGNED_OBJECT_ID,
                SIMCONNECT_RECV_ASSIGNED_OBJECT_ID,
                AssignedObjectId
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_AIRPORT_LIST,
                SIMCONNECT_RECV_AIRPORT_LIST,
                AirportList
            ),
            (
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_FACILITY_DATA,
                SIMCONNECT_RECV_FACILITY_DATA,
                FacilityData
            ),
        }
    };
}

extern "C" fn dispatch_cb(
    recv: *mut sys::SIMCONNECT_RECV,
    _cb_data: sys::DWORD,
    p_context: *mut std::ffi::c_void,
) {
    macro_rules! recv_cb {
        ($( ($ID:ident, $T:ident, $E:ident), )*) => {
            unsafe {
                match (*recv).dwID as sys::SIMCONNECT_RECV_ID {
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_NULL => Some(SimConnectRecv::Null),
                    $(
                        sys::$ID => Some(SimConnectRecv::$E(&*(recv as *mut sys::$T))),
                    )*
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_SIMOBJECT_DATA_BYTYPE => {
                        Some(SimConnectRecv::SimObjectData(&*(recv as *mut sys::SIMCONNECT_RECV_SIMOBJECT_DATA)))
                    }
                    _ => None,
                }
            }
        }
    }
    let recv = recv!(recv_cb);

    if let Some(recv) = recv {
        let sim = unsafe { &mut *(p_context as *mut SimConnect) };
        (sim.callback)(unsafe { &mut *(p_context as *mut SimConnect) }, recv);
    }
}

macro_rules! recv_enum {
    ($( ($ID:ident, $T:ident, $E:ident), )*) => {
        /// Message received from SimConnect.
        #[derive(Debug)]
        pub enum SimConnectRecv<'a> {
            Null,
            $(
                $E(&'a sys::$T),
            )*
        }
    }
}
recv!(recv_enum);

impl sys::SIMCONNECT_RECV_EVENT {
    /// The ID for this event.
    pub fn id(&self) -> sys::DWORD {
        self.uEventID
    }

    /// The data for this event.
    pub fn data(&self) -> sys::DWORD {
        self.dwData
    }
}

impl sys::SIMCONNECT_RECV_EVENT_EX1 {
    /// The ID for this event.
    pub fn id(&self) -> sys::DWORD {
        self.uEventID
    }

    /// The data for this event.
    pub fn data(&self) -> [sys::DWORD; 5] {
        [
            self.dwData0,
            self.dwData1,
            self.dwData2,
            self.dwData3,
            self.dwData4,
        ]
    }
}

impl sys::SIMCONNECT_RECV_ASSIGNED_OBJECT_ID {
    pub fn id(&self) -> sys::DWORD {
        self.dwRequestID
    }

    pub fn object_id(&self) -> sys::DWORD {
        self.dwObjectID
    }
}

impl sys::SIMCONNECT_RECV_SIMOBJECT_DATA {
    /// The ID for this data.
    pub fn id(&self) -> sys::DWORD {
        self.dwRequestID
    }

    /// Convert a SimObjectData event into the data it contains.
    pub fn into<T: DataDefinition>(&self, sim: &SimConnect) -> Option<&T> {
        let define_id = sim.data_definitions[&TypeId::of::<T>()];
        if define_id == sys::SIMCONNECT_DATA_DEFINITION_ID(self.dwDefineID) {
            // UB: creates unaligned reference
            Some(unsafe { &*(&raw const self.dwData as *const T) })
        } else {
            None
        }
    }
}

impl sys::SIMCONNECT_RECV_CLIENT_DATA {
    /// The ID for this data.
    pub fn id(&self) -> sys::DWORD {
        self._base.dwRequestID
    }

    /// Convert a ClientData event into the data it contains.
    pub fn into<T: ClientDataDefinition>(&self, sim: &SimConnect) -> Option<&T> {
        let define_id = sim.client_data_definitions[&TypeId::of::<T>()];
        if define_id == self._base.dwDefineID {
            // UB: creates unaligned reference
            Some(unsafe { &*(&raw const self._base.dwData as *const T) })
        } else {
            None
        }
    }
}

impl sys::SIMCONNECT_RECV_AIRPORT_LIST {
    pub fn id(&self) -> sys::DWORD {
        self._base.dwRequestID
    }

    pub fn data(&self) -> &[sys::SIMCONNECT_DATA_FACILITY_AIRPORT] {
        let array_size = self._base.dwArraySize as usize;
        let data_ptr: *const sys::SIMCONNECT_DATA_FACILITY_AIRPORT = self.rgData.as_ptr();

        unsafe { std::slice::from_raw_parts(data_ptr, array_size) }
    }
}

impl sys::SIMCONNECT_RECV_FACILITY_DATA {
    /// The ID for this data.
    pub fn id(&self) -> sys::SIMCONNECT_DATA_REQUEST_ID {
        sys::SIMCONNECT_DATA_REQUEST_ID(self.UserRequestId)
    }

    pub fn into<T>(&self, sim: &SimConnect) -> Option<T>
    where
        T: FacilityDefinition,
        T: From<T::RawType>,
    {
        // todo: fix this check for children of FacilityDefinition
        // Check if this facility type has been defined/requested
        // if sim
        //     .facility_data_definitions
        //     .contains_key(&TypeId::of::<T>())
        // {
        // read raw data and
        let raw_data = unsafe { std::ptr::read(&raw const self.Data as *const T::RawType) };
        Some(raw_data.into())
        // } else {
        //     None
        // }
    }
}

/// Specify how often data is to be sent to the client.
#[derive(Debug)]
pub enum Period {
    /// Specifies that the data is not to be sent
    Never = sys::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_NEVER as isize,
    /// Specifies that the data should be sent once only. Note that this is not
    /// an efficient way of receiving data frequently, use one of the other
    /// periods if there is a regular frequency to the data request.
    Once = sys::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_ONCE as isize,
    /// Specifies that the data should be sent every visual (rendered) frame.
    VisualFrame = sys::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_VISUAL_FRAME as isize,
    /// Specifies that the data should be sent every simulated frame, whether that frame is
    /// rendered or not.
    SimFrame = sys::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME as isize,
    /// Specifies that the data should be sent once every second.
    Second = sys::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SECOND as isize,
}

/// An allocated client data memory region. Dropping this struct will not
/// deallocate the memory which has been allocated in the sim.
pub struct ClientDataArea<T: ClientDataDefinition> {
    client_id: sys::SIMCONNECT_CLIENT_DATA_ID,
    phantom: std::marker::PhantomData<T>,
}

impl From<usize> for sys::SIMCONNECT_DATA_DEFINITION_ID {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
