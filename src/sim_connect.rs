// use crate::sys;

// FIXME: these should be provided in `sys`, but bindgen is ignoring them for some reason.
extern "C" {
    fn SimConnect_Open(
        sim_connect: *mut u8,
        name: *const std::os::raw::c_char,
        hwnd: *mut u8,
        user_event_win32: i32,
        event_handle: *mut u8,
        config_index: i32,
    ) -> i32;
    fn SimConnect_Close(sim_connect: *mut u8) -> i32;
}

/// A SimConnect session. This provides access to data within the MSFS sim.
pub struct SimConnect(*mut u8);

impl SimConnect {
    /// The `SimConnect::open` function is used to send a request to the Microsoft
    /// Flight Simulator server to open up communications with a new client.
    pub fn open(name: &str) -> Result<SimConnect, ()> {
        unsafe {
            let ptr = std::ptr::null_mut();
            let name = std::ffi::CString::new(name).unwrap();
            if SimConnect_Open(
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
        assert!(unsafe { SimConnect_Close(self.0) } >= 0);
    }
}
