use crate::sys;

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect(*mut sys::HANDLE);

impl SimConnect {
    /// The `SimConnect::open` function is used to send a request to the Microsoft
    /// Flight Simulator server to open up communications with a new client.
    pub fn open(name: &str) -> Result<SimConnect, ()> {
        unsafe {
            let ptr = std::ptr::null_mut();
            let name = std::ffi::CString::new(name).unwrap();
            if sys::SimConnect_Open(
                ptr,
                name.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
            ) >= 0
            {
                Ok(SimConnect(ptr))
            } else {
                Err(())
            }
        }
    }
}

impl Drop for SimConnect {
    fn drop(&mut self) {
        assert!(unsafe { sys::SimConnect_Close(*self.0) } >= 0);
    }
}
