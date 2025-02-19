//! Bindings to the Legacy/gauges.h API

use crate::sys;

#[doc(hidden)]
pub trait SimVarF64 {
    fn to(self) -> f64;
    fn from(v: f64) -> Self;
}

impl SimVarF64 for f64 {
    fn to(self) -> f64 {
        self
    }

    fn from(v: f64) -> Self {
        v
    }
}

impl SimVarF64 for bool {
    fn to(self) -> f64 {
        if self {
            1.0
        } else {
            0.0
        }
    }

    fn from(v: f64) -> Self {
        v != 0.0
    }
}

impl SimVarF64 for u8 {
    fn to(self) -> f64 {
        self as f64
    }

    fn from(v: f64) -> Self {
        v as Self
    }
}

/// aircraft_varget
/// get_aircraft_var_enum
#[derive(Debug)]
pub struct AircraftVariable {
    simvar: sys::ENUM,
    units: sys::ENUM,
    index: sys::SINT32,
}
impl AircraftVariable {
    pub fn from(name: &str, units: &str, index: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let name = std::ffi::CString::new(name).unwrap();
        let units = std::ffi::CString::new(units).unwrap();

        let simvar = unsafe { sys::get_aircraft_var_enum(name.as_ptr()) };
        if simvar == -1 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid name",
            )));
        }

        let units = unsafe { sys::get_units_enum(units.as_ptr()) };
        if units == -1 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid units",
            )));
        }
        Ok(Self {
            simvar,
            units,
            index: index as sys::SINT32,
        })
    }

    pub fn get<T: SimVarF64>(&self) -> T {
        let v = unsafe { sys::aircraft_varget(self.simvar, self.units, self.index) };
        T::from(v)
    }
}


pub fn fs_events_trigger_key_event(event_id: sys::FsEventId, value: sys::FsVarParamArray) {
    unsafe {
        sys::fsEventsTriggerKeyEvent(event_id, value);
    }
} 

/// register_named_variable
/// set_named_variable_typed_value
/// get_named_variable_value
/// set_named_variable_value
#[derive(Debug)]
pub struct NamedVariable(sys::ID);
impl NamedVariable {
    pub fn from(name: &str) -> Self {
        Self(unsafe {
            let name = std::ffi::CString::new(name).unwrap();
            sys::register_named_variable(name.as_ptr())
        })
    }

    pub fn get_value<T: SimVarF64>(&self) -> T {
        let v = unsafe { sys::get_named_variable_value(self.0) };
        T::from(v)
    }

    pub fn set_value(&self, v: impl SimVarF64) {
        let v = v.to();
        unsafe { sys::set_named_variable_value(self.0, v) }
    }
}

pub struct NamedVariableApi(sys::FsNamedVarId, sys::FsUnitId);
impl NamedVariableApi {
    pub fn from(name: &str, units: &str) -> Self {
        let name = std::ffi::CString::new(name).unwrap();
        let units = std::ffi::CString::new(units).unwrap();
        let var = unsafe { sys::fsVarsRegisterNamedVar(name.as_ptr()) };
        let unit = unsafe { sys::fsVarsGetUnitId(units.as_ptr()) };
        Self(var, unit)
    }

    pub fn get<T: SimVarF64>(&self) -> T {
        let mut v = 0.0;
        unsafe { sys::fsVarsNamedVarGet(self.0, self.1, &mut v) };
        T::from(v)
    }

    pub fn set(&self, v: impl SimVarF64) {
        let v = v.to();
        unsafe { sys::fsVarsNamedVarSet(self.0, self.1, v) };
    }
}


pub struct AircraftVariableApi {simvar: sys::FsSimVarId , units: sys::FsUnitId, index: u32, params: sys::FsVarParamArray, name: String}

impl AircraftVariableApi {
    pub fn from(name: &str, units: &str, index: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let name_cstr = std::ffi::CString::new(name).unwrap();
        let units_cstr = std::ffi::CString::new(units).unwrap();
        let var = unsafe { let varResult = sys::fsVarsGetAircraftVarId(name_cstr.as_ptr());
            if (varResult == -1) {
                println!("Error getting aircraft var id for {} with error {}", name, varResult);
            }
            varResult
         };
        let unit = unsafe { let result = sys::fsVarsGetUnitId(units_cstr.as_ptr());
            if (result == -1) {
                println!("Error getting unit id for {} with error {}", units, result);
            }
            result
         };

        let param1 = sys::FsVarParamVariant {
            type_: 0,
            __bindgen_anon_1: sys::FsVarParamVariant__bindgen_ty_1 { intValue: index},
        };

        let mut paramsArray = vec![param1; 100].into_boxed_slice();

        //let mut paramsArrayP = paramsArray.into_boxed_slice();


        let params = sys::FsVarParamArray {
            size: 1 as usize,
            array: Box::into_raw(paramsArray) as *mut sys::FsVarParamVariant,
        };

       // std::mem::forget(paramsArray);
       
        
      // unsafe { println!("INIT MSFS var: {}, unit: {} param {}", var, unit, (*param.array).__bindgen_anon_1.intValue) };
        Ok(Self {
            simvar: var,
            units: unit,
            index: index,
            params,
            name: name.to_string()
        })

    }

    pub fn get<T: SimVarF64>(&self) -> T {
        let mut v = 0.0;

     //   unsafe { println!("var: {}, unit: {} param {}", self.simvar, self.units, (*self.params.array).__bindgen_anon_1.intValue) };

  
        unsafe { sys::fsVarsAircraftVarGet(self.simvar, self.units, self.params, &mut v) };
        T::from(v)
    }
/* 
    extern "C" {
        pub fn fsVarsAircraftVarSet(
            simvar: FsSimVarId,
            unit: FsUnitId,
            param: FsVarParamArray,
            value: f64,
        ) -> FsVarError; */

     pub fn set(&self, value: f64) {

   /*      let param1 = sys::FsVarParamVariant {
            type_: 0,
            __bindgen_anon_1: sys::FsVarParamVariant__bindgen_ty_1 { intValue: self.index},
        }; */

     //   let layout = Layout::array::<sys::FsVarParamVariant>(1).unwrap();

       // let array = unsafe { alloc(layout) as *mut sys::FsVarParamVariant };
       // let paramsArray = [param1];
       // let boxParam = Box::new(paramsArray);
       // let ptr = Box::into_raw(boxParam) as *mut sys::FsVarParamVariant;
   /*     let variant = unsafe { &mut *array.add(0) };
       variant.type_ = 0;
       variant.__bindgen_anon_1 = sys::FsVarParamVariant__bindgen_ty_1 { intValue: self.index};


       
        let params = sys::FsVarParamArray {
            size: 1 as u32,
            array,
        }; */

        //std::mem::forget(array);


        unsafe { 
            let retval = sys::fsVarsAircraftVarSet(self.simvar, self.units, self.params, value);

            if (retval != 0) {
                println!("Error setting aircraft var: {:?} for {:?} : {:?}, value {:?}", retval, self.name, self.index, value);
            }
          /*   if(!value.is_finite()) {
                println!("Value is not finite, wtf unsafe");
            
            }  */
         //   drop(Box::from_raw(ptr)); 
        };

    /*     if(!value.is_finite()) {
            println!("Value is not finite, wtf");
        
        }  */

      //  unsafe { dealloc(array as *mut u8, layout) };
        
    } 
}

/* extern "C" {
    pub fn fsVarsGetRegisteredNamedVarId(name: *const ::std::os::raw::c_char) -> FsNamedVarId;
}
extern "C" {
    pub fn fsVarsRegisterNamedVar(name: *const ::std::os::raw::c_char) -> FsNamedVarId;
}
extern "C" {
    pub fn fsVarsNamedVarGet(var: FsNamedVarId, unit: FsUnitId, result: *mut f64);
}
extern "C" {
    pub fn fsVarsNamedVarSet(var: FsNamedVarId, unit: FsUnitId, value: f64);
} */

/// trigger_key_event
pub fn trigger_key_event(event_id: sys::ID32, value: sys::UINT32) {
    unsafe {
        sys::trigger_key_event(event_id, value);
    }
}

/// trigger_key_event_EX1
pub fn trigger_key_event_ex1(
    event_id: sys::ID32,
    value0: sys::UINT32,
    value1: sys::UINT32,
    value2: sys::UINT32,
    value3: sys::UINT32,
    value4: sys::UINT32,
) {
    unsafe {
        sys::trigger_key_event_EX1(event_id, value0, value1, value2, value3, value4);
    }
}

#[doc(hidden)]
pub trait ExecuteCalculatorCodeImpl {
    fn execute(code: &std::ffi::CStr) -> Option<Self>
    where
        Self: Sized;
}

#[doc(hidden)]
impl ExecuteCalculatorCodeImpl for f64 {
    fn execute(code: &std::ffi::CStr) -> Option<Self> {
        unsafe {
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
    fn execute(code: &std::ffi::CStr) -> Option<Self> {
        unsafe {
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
    fn execute(code: &std::ffi::CStr) -> Option<Self> {
        unsafe {
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
    fn execute(code: &std::ffi::CStr) -> Option<Self> {
        unsafe {
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
pub fn execute_calculator_code<T: ExecuteCalculatorCodeImpl>(code: &str) -> Option<T> {
    let code = std::ffi::CString::new(code).unwrap();
    ExecuteCalculatorCodeImpl::execute(code.as_c_str())
}

/// Holds compiled calculator code, wraps `gauge_calculator_code_precompile`.
#[derive(Debug)]
pub struct CompiledCalculatorCode {
    p_compiled: sys::PCSTRINGZ,
    _p_compiled_size: sys::UINT32,
}

impl CompiledCalculatorCode {
    /// Create a new CompiledCalculatorCode instance.
    pub fn new(code: &str) -> Option<Self> {
        let mut p_compiled = std::mem::MaybeUninit::uninit();
        let mut p_compiled_size = std::mem::MaybeUninit::uninit();
        unsafe {
            let code = std::ffi::CString::new(code).unwrap();
            if sys::gauge_calculator_code_precompile(
                p_compiled.as_mut_ptr(),
                p_compiled_size.as_mut_ptr(),
                code.as_ptr(),
            ) != 0
            {
                Some(CompiledCalculatorCode {
                    p_compiled: p_compiled.assume_init(),
                    _p_compiled_size: p_compiled_size.assume_init(),
                })
            } else {
                None
            }
        }
    }

    /// Execute this CompiledCalculatorCode instance.
    pub fn execute<T: ExecuteCalculatorCodeImpl>(&self) -> Option<T> {
        ExecuteCalculatorCodeImpl::execute(unsafe { std::ffi::CStr::from_ptr(self.p_compiled) })
    }
}
