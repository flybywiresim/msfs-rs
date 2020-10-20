//! Bindings to the Legacy/gauges.h API

use crate::sys;

/// aircraft_varget
pub fn aircraft_varget(simvar: sys::ENUM, units: sys::ENUM, index: sys::SINT32) -> f64 {
    unsafe { sys::aircraft_varget(simvar, units, index) }
}

/// get_aircraft_var_enum
pub fn get_aircraft_var_enum(name: &str) -> sys::ENUM {
    unsafe {
        let name = std::ffi::CString::new(name).unwrap();
        sys::get_aircraft_var_enum(name.as_ptr())
    }
}

/// get_units_enum
pub fn get_units_enum(unitname: &str) -> sys::ENUM {
    unsafe {
        let name = std::ffi::CString::new(unitname).unwrap();
        sys::get_units_enum(name.as_ptr())
    }
}

#[doc(hidden)]
pub trait ExecuteCalculatorCodeImpl {
    fn execute(code: &str) -> Option<Self>
    where
        Self: Sized;
}

#[doc(hidden)]
impl ExecuteCalculatorCodeImpl for f64 {
    fn execute(code: &str) -> Option<Self> {
        unsafe {
            let code = std::ffi::CString::new(code).unwrap();
            let mut n = 0.0;
            if sys::execute_calculator_code(
                code.as_ptr(),
                &mut n,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 1
            {
                Some(n)
            } else {
                None
            }
        }
    }
}

#[doc(hidden)]
impl ExecuteCalculatorCodeImpl for i32 {
    fn execute(code: &str) -> Option<Self> {
        unsafe {
            let code = std::ffi::CString::new(code).unwrap();
            let mut n = 0;
            if sys::execute_calculator_code(
                code.as_ptr(),
                std::ptr::null_mut(),
                &mut n,
                std::ptr::null_mut(),
            ) == 1
            {
                Some(n)
            } else {
                None
            }
        }
    }
}

#[doc(hidden)]
impl ExecuteCalculatorCodeImpl for String {
    fn execute(code: &str) -> Option<Self> {
        unsafe {
            let code = std::ffi::CString::new(code).unwrap();
            let mut s = std::ptr::null();
            if sys::execute_calculator_code(
                code.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut s,
            ) == 1
            {
                Some(std::ffi::CStr::from_ptr(s).to_str().unwrap().to_owned())
            } else {
                None
            }
        }
    }
}

#[doc(hidden)]
impl ExecuteCalculatorCodeImpl for () {
    fn execute(code: &str) -> Option<Self> {
        unsafe {
            let code = std::ffi::CString::new(code).unwrap();
            if sys::execute_calculator_code(
                code.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 1
            {
                Some(())
            } else {
                None
            }
        }
    }
}

/// execute_calculator_code
pub fn execute_calculator_code<T>(code: &str) -> Option<T>
where
    T: ExecuteCalculatorCodeImpl,
{
    ExecuteCalculatorCodeImpl::execute(code)
}
