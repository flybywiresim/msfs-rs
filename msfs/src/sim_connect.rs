#![allow(clippy::too_many_arguments)]

use crate::sys;
use std::collections::HashMap;
use std::pin::Pin;

pub use sys::SIMCONNECT_OBJECT_ID_USER;

pub use msfs_derive::sim_connect_data_definition as data_definition;

pub type DataXYZ = sys::SIMCONNECT_DATA_XYZ;

/// A trait implemented by the `data_definition` attribute.
pub trait DataDefinition: 'static {
    #[doc(hidden)]
    const DEFINITIONS: &'static [(&'static str, &'static str, f32, sys::SIMCONNECT_DATATYPE)];
}

/// Rusty HRESULT wrapper.
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
fn map_err(result: sys::HRESULT) -> Result<()> {
    if result >= 0 {
        Ok(())
    } else {
        Err(HResult(result))
    }
}

/// Callback provided to SimConnect session.
pub type SimConnectRecvCallback = dyn Fn(&mut SimConnect, SimConnectRecv);

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect {
    handle: sys::HANDLE,
    callback: Box<SimConnectRecvCallback>,
    definitions: HashMap<std::any::TypeId, sys::SIMCONNECT_DATA_DEFINITION_ID>,
    event_id_counter: sys::DWORD,
}

impl SimConnect {
    /// Send a request to the Microsoft Flight Simulator server to open up communications with a new client.
    pub fn open<F>(name: &str, callback: F) -> Result<Pin<Box<SimConnect>>>
    where
        F: Fn(&mut SimConnect, SimConnectRecv) + 'static,
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
                definitions: HashMap::new(),
                event_id_counter: 0,
            });
            sim.call_dispatch()?;
            Ok(sim)
        }
    }

    #[doc(hidden)]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut SimConnect {
        self
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
        let key = std::any::TypeId::of::<T>();
        let maybe_define_id = self.definitions.len() as sys::SIMCONNECT_DATA_DEFINITION_ID;
        match self.definitions.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                unsafe {
                    map_err(sys::SimConnect_ClearDataDefinition(
                        self.handle,
                        maybe_define_id,
                    ))?;
                }
                for (datum_name, units_type, epsilon, datatype) in T::DEFINITIONS {
                    let datum_name = std::ffi::CString::new(*datum_name).unwrap();
                    let units_type = std::ffi::CString::new(*units_type).unwrap();
                    unsafe {
                        map_err(sys::SimConnect_AddToDataDefinition(
                            self.handle,
                            maybe_define_id,
                            datum_name.as_ptr(),
                            units_type.as_ptr(),
                            *datatype,
                            *epsilon,
                            sys::SIMCONNECT_UNUSED,
                        ))?;
                    }
                }
                entry.insert(maybe_define_id);
                Ok(maybe_define_id)
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
        unsafe {
            let event_name = std::ffi::CString::new(event_name).unwrap();
            map_err(sys::SimConnect_MapClientEventToSimEvent(
                self.handle,
                event_id,
                event_name.as_ptr(),
            ))?;

            map_err(sys::SimConnect_AddClientEventToNotificationGroup(
                self.handle,
                0,
                event_id,
                if mask { 1 } else { 0 },
            ))?;

            map_err(sys::SimConnect_SetNotificationGroupPriority(
                self.handle,
                0,
                sys::SIMCONNECT_GROUP_PRIORITY_HIGHEST_MASKABLE,
            ))?;
        }
        Ok(event_id)
    }
}

impl Drop for SimConnect {
    fn drop(&mut self) {
        unsafe {
            map_err(sys::SimConnect_Close(self.handle)).expect("SimConnect_Close");
        }
    }
}

macro_rules! recv {
    ($V:ident) => {
        $V!(
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
                SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_SIMOBJECT_DATA,
                SIMCONNECT_RECV_SIMOBJECT_DATA,
                SimObjectData
            ),
        );
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
                    _ => None,
                }
            }
        }
    }
    let recv = recv!(recv_cb);

    if let Some(recv) = recv {
        let sim = unsafe { &*(p_context as *const SimConnect) };
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

impl sys::SIMCONNECT_RECV_SIMOBJECT_DATA {
    /// The ID for this data.
    pub fn id(&self) -> sys::DWORD {
        self.dwRequestID
    }

    /// Convert a SimObjectData event into the data it contains.
    pub fn into<T: DataDefinition>(&self, sim: &SimConnect) -> Option<&T> {
        let define_id = sim.definitions[&std::any::TypeId::of::<T>()];
        if define_id == self.dwDefineID {
            Some(unsafe { &*(&self.dwData as *const sys::DWORD as *const T) })
        } else {
            None
        }
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
