use crate::sys;

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect(sys::HANDLE);

impl SimConnect {
    /// The `SimConnect::open` function is used to send a request to the Microsoft
    /// Flight Simulator server to open up communications with a new client.
    pub fn open(name: &str) -> Result<SimConnect, ()> {
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
                Ok(SimConnect(ptr))
            } else {
                Err(())
            }
        }
    }

    /// Receive and process the next SimConnect message.
    pub fn get_next_dispatch(&self) -> Option<SimConnectRecv> {
        unsafe {
            let mut recv = std::mem::MaybeUninit::uninit();
            let mut size = std::mem::MaybeUninit::uninit();
            if sys::SimConnect_GetNextDispatch(self.0, recv.as_mut_ptr(), size.as_mut_ptr()) >= 0 {
                let recv = recv.assume_init();
                let _ = size.assume_init();
                match (*recv).dwID {
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_NULL => Some(SimConnectRecv::Null),
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_OPEN => Some(SimConnectRecv::Open(
                        Box::from_raw(recv as *mut sys::SIMCONNECT_RECV_OPEN),
                    )),
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_QUIT => Some(SimConnectRecv::Quit),
                    sys::SIMCONNECT_RECV_ID_SIMCONNECT_RECV_ID_EVENT => {
                        Some(SimConnectRecv::Event(Box::from_raw(
                            recv as *mut sys::SIMCONNECT_RECV_EVENT,
                        )))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

/// Message received from `SimConnect::get_next_dispatch`.
#[derive(Debug)]
pub enum SimConnectRecv {
    Null,
    Exception(Box<sys::SIMCONNECT_RECV_EXCEPTION>),
    Open(Box<sys::SIMCONNECT_RECV_OPEN>),
    Quit,
    Event(Box<sys::SIMCONNECT_RECV_EVENT>),
}

impl Drop for SimConnect {
    fn drop(&mut self) {
        assert!(unsafe { sys::SimConnect_Close(self.0) } >= 0);
    }
}
