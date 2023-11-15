//! Bindings for the commbus API available in the MSFS SDK.
use crate::sys;
use std::{
    ffi::{self, CString},
    slice,
};

// SAFETY: It should be safe to use `FnMut` as callback
// as the execution is not happen in parallel (hopefully).
type CommBusCallback<'a> = Box<dyn FnMut(&str) + 'a>;

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
pub struct CommBus<'a> {
    event_name: CString,
    callback: Box<CommBusCallback<'a>>,
}
impl<'a> CommBus<'a> {
    /// Registers to a communication event.
    pub fn register(event_name: &str, callback: impl FnMut(&str) + 'a) -> Option<Self> {
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

    /// Calls a communication event.
    /// Returns `true` if the call was successful.
    pub fn call(event_name: &str, args: &str, called: CommBusBroadcastFlags) -> bool {
        if let (Ok(event_name), Ok(args_cstr)) = (CString::new(event_name), CString::new(args)) {
            unsafe {
                sys::fsCommBusCall(
                    event_name.as_ptr(),
                    args_cstr.as_ptr(),
                    (args.len() + 1) as ffi::c_uint,
                    called.into(),
                )
            }
        } else {
            false
        }
    }

    extern "C" fn c_callback(args: *const ffi::c_char, size: ffi::c_uint, ctx: *mut ffi::c_void) {
        if !ctx.is_null() {
            let (mut callback, args) = unsafe {
                (
                    Box::from_raw(ctx as *mut CommBusCallback<'a>),
                    // SAFETY: because i8/u8 is 1 byte we can use size directly as length of the slice
                    slice::from_raw_parts(args as *const u8, size as usize),
                )
            };
            callback(&String::from_utf8_lossy(args));
            // Don't free callback as it's still registered
            Box::leak(callback);
        }
    }
}
impl Drop for CommBus<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::fsCommBusUnregister(self.event_name.as_ptr(), Some(Self::c_callback));
        }
    }
}
