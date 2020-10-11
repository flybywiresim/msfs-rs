use crate::sys;

#[inline(always)]
fn check(result: sys::HRESULT) {
    assert!(result >= 0);
}

/// Callback provided to SimConnect session.
type SimConnectRecvCallback = fn(&SimConnect, SimConnectRecv);

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect {
    handle: sys::HANDLE,
    callback: SimConnectRecvCallback,
}

extern "C" fn dispatch_cb(
    recv: *mut sys::SIMCONNECT_RECV,
    _cb_data: sys::DWORD,
    p_context: *mut std::ffi::c_void,
) {
    let sim = unsafe { &*(p_context as *mut SimConnect) };
    let recv = unsafe {
        match (*recv).dwID {
            sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_NULL => Some(SimConnectRecv::Null),
            sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_OPEN => Some(SimConnectRecv::Open(
                &*(recv as *mut sys::SIMCONNECT_RECV_OPEN),
            )),
            sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_QUIT => Some(SimConnectRecv::Quit(
                &*(recv as *mut sys::SIMCONNECT_RECV_QUIT),
            )),
            sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_EVENT => Some(SimConnectRecv::Event(
                &*(recv as *mut sys::SIMCONNECT_RECV_EVENT),
            )),
            _ => None,
        }
    };
    if let Some(recv) = recv {
        (sim.callback)(sim, recv);
    }
}

impl SimConnect {
    /// The `SimConnect::open` function is used to send a request to the Microsoft
    /// Flight Simulator server to open up communications with a new client.
    pub fn open(name: &str, callback: SimConnectRecvCallback) -> Result<SimConnect, ()> {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let name = std::ffi::CString::new(name).unwrap();
            if sys::SimConnect_Open(
                &mut ptr,
                name.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
            ) >= 0
            {
                debug_assert!(!ptr.is_null());
                let mut sim = SimConnect {
                    handle: ptr,
                    callback,
                };
                check(sys::SimConnect_CallDispatch(
                    ptr,
                    Some(dispatch_cb),
                    &mut sim as *mut SimConnect as *mut std::ffi::c_void,
                ));
                Ok(sim)
            } else {
                Err(())
            }
        }
    }

    /// Register a sim event to be relayed to the `callback`, mapped by the client event `id`.
    pub fn map_client_event_to_sim_event(&self, id: u32, name: &str) {
        unsafe {
            let name = std::ffi::CString::new(name).unwrap();
            check(sys::SimConnect_MapClientEventToSimEvent(
                self.handle,
                id,
                name.as_ptr(),
            ));
        }
    }
}

/// Message received from `SimConnect::get_next_dispatch`.
#[derive(Debug)]
pub enum SimConnectRecv<'a> {
    Null,
    Exception(&'a sys::SIMCONNECT_RECV_EXCEPTION),
    Open(&'a sys::SIMCONNECT_RECV_OPEN),
    Quit(&'a sys::SIMCONNECT_RECV_QUIT),
    Event(&'a sys::SIMCONNECT_RECV_EVENT),
}

impl Drop for SimConnect {
    fn drop(&mut self) {
        assert!(unsafe { sys::SimConnect_Close(self.handle) } >= 0);
    }
}
