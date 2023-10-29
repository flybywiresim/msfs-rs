//! Bindings for the commbus API available in the MSFS SDK.
use crate::sys;
use std::{
    ffi::{self, CString},
    slice,
};

type CommBusCallback = Box<dyn Fn(&[u8])>;

/// Used to specify the type of module/gauge to broadcast an event to.
#[derive(Default)]
pub enum CommBusBroadcastFlags {
    JS,
    WASM,
    WASMSelfCall,
    #[default]
    Default,
    AllWASM,
    All,
}

impl From<CommBusBroadcastFlags> for sys::FsCommBusBroadcastFlags {
    fn from(value: CommBusBroadcastFlags) -> Self {
        match value {
            CommBusBroadcastFlags::JS => sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_JS,
            CommBusBroadcastFlags::WASM => sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_Wasm,
            CommBusBroadcastFlags::WASMSelfCall => {
                sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_WasmSelfCall
            }
            CommBusBroadcastFlags::Default => {
                sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_Default
            }
            CommBusBroadcastFlags::AllWASM => {
                sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_AllWasm
            }
            CommBusBroadcastFlags::All => sys::FsCommBusBroadcastFlags_FsCommBusBroadcast_All,
        }
    }
}

/// CommBus handle. When this handle goes out of scope the callback will be unregistered.
pub struct CommBus {
    event_name: CString,
    callback: Box<CommBusCallback>,
}
impl CommBus {
    /// Register to a communication event.
    pub fn register(event_name: &str, callback: impl Fn(&[u8]) + 'static) -> Option<Self> {
        let this = Self {
            event_name: CString::new(event_name).ok()?,
            callback: Box::new(Box::new(callback)),
        };
        let res = unsafe {
            sys::fsCommBusRegister(
                this.event_name.as_ptr(),
                Some(Self::c_callback),
                this.callback.as_ref() as *const _ as *mut _,
            )
        };
        if res {
            Some(this)
        } else {
            None
        }
    }

    /// Call a communication event.
    pub fn call(event_name: &str, args: &[i8], called: CommBusBroadcastFlags) -> bool {
        if let Ok(event_name) = CString::new(event_name) {
            unsafe {
                sys::fsCommBusCall(
                    event_name.as_ptr(),
                    args.as_ptr(),
                    args.len() as ffi::c_uint,
                    called.into(),
                )
            }
        } else {
            false
        }
    }

    extern "C" fn c_callback(args: *const ffi::c_char, size: ffi::c_uint, ctx: *mut ffi::c_void) {
        if !ctx.is_null() {
            let (callback, args) = unsafe {
                (
                    Box::from_raw(ctx as *mut CommBusCallback),
                    // SAFETY: because i8/u8 is 1 byte we can use size directly as length of the slice
                    slice::from_raw_parts(args as *const u8, size as usize),
                )
            };
            callback(args);
            // Don't free callback as it's still registered
            Box::leak(callback);
        }
    }
}
impl Drop for CommBus {
    fn drop(&mut self) {
        unsafe {
            sys::fsCommBusUnregister(self.event_name.as_ptr(), Some(Self::c_callback));
        }
    }
}
