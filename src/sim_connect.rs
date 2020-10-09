use crate::sys;

pub struct SimConnect(sys::phSimConnect);

impl SimConnect {
    pub fn open(name: &str) -> Result<SimConnect, ()> {
        unsafe {
            let sc: sys::phSimConnect = std::mem::MaybeUninit::uninit();
            if sys::SimConnect_Open(
                sc.as_mut_ptr(),
                name,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            ) >= 0 {
                SimConnect(sc.assume_init())
            } else {
                Err(())
            }
        }
    }
}
