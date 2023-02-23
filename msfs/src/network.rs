use std::{
    ffi::{self, CStr, CString},
    ptr,
};

use crate::sys;

/// A builder to build network requests
pub struct NetworkRequestBuilder<'a> {
    url: CString,
    headers: Vec<CString>,
    data: Option<&'a mut [u8]>,
}
impl<'a> NetworkRequestBuilder<'a> {
    /// Create a new network request
    pub fn new(url: &str) -> Option<Self> {
        Some(Self {
            url: CString::new(url).ok()?,
            headers: vec![],
            data: None,
        })
    }

    pub fn with_header(mut self, header: &str) -> Option<Self> {
        self.headers.push(CString::new(header).ok()?);
        Some(self)
    }

    pub fn with_data(mut self, data: &'a mut [u8]) -> Self {
        self.data = Some(data);
        self
    }

    /// Do HTTP GET request
    pub fn get(mut self) -> Option<NetworkRequest> {
        let mut params = self.generate_params(std::ptr::null_mut());
        let request_id = unsafe {
            sys::fsNetworkHttpRequestGet(
                self.url.as_ptr(),
                &mut params as *mut sys::FsNetworkHttpRequestParam,
                None,
                ptr::null_mut(),
            )
        };
        if request_id == 0 {
            None
        } else {
            Some(NetworkRequest(request_id))
        }
    }

    /// Do HTTP POST request
    pub fn post(mut self, post_field: &str) -> Option<NetworkRequest> {
        let post_field = CString::new(post_field).unwrap();
        let raw_post_field = post_field.into_raw();
        let mut params = self.generate_params(raw_post_field);
        let request_id = unsafe {
            let id = sys::fsNetworkHttpRequestPost(
                self.url.as_ptr(),
                &mut params as *mut sys::FsNetworkHttpRequestParam,
                None,
                ptr::null_mut(),
            );
            drop(CString::from_raw(raw_post_field));
            id
        };
        if request_id == 0 {
            None
        } else {
            Some(NetworkRequest(request_id))
        }
    }

    /// Do HTTP PUT request
    pub fn put(mut self) -> Option<NetworkRequest> {
        let mut params = self.generate_params(ptr::null_mut());
        let request_id = unsafe {
            sys::fsNetworkHttpRequestPut(
                self.url.as_ptr(),
                &mut params as *mut sys::FsNetworkHttpRequestParam,
                None,
                ptr::null_mut(),
            )
        };
        if request_id == 0 {
            None
        } else {
            Some(NetworkRequest(request_id))
        }
    }

    fn generate_params(&mut self, post_field: *mut i8) -> sys::FsNetworkHttpRequestParam {
        // Safety: Because the struct in the C code is not defined as const char* we need to cast
        // the *const into *mut which should be safe because the function should not change it anyway
        let mut headers: Vec<_> = self
            .headers
            .iter_mut()
            .map(|h| h.as_ptr() as *mut i8)
            .collect();
        let data_len = self.data.as_ref().map_or(0, |d| d.len());
        sys::FsNetworkHttpRequestParam {
            postField: post_field,
            headerOptions: headers.as_mut_ptr(),
            headerOptionsSize: self.headers.len() as std::os::raw::c_uint,
            data: self
                .data
                .as_mut()
                .map_or(ptr::null_mut(), |d| d.as_mut_ptr()),
            dataSize: data_len as std::os::raw::c_uint,
        }
    }
}

/// Network request handle
pub struct NetworkRequest(sys::FsNetworkRequestId);
impl NetworkRequest {
    pub fn cancel(&self) -> bool {
        unsafe { sys::fsNetworkHttpCancelRequest(self.0) }
    }

    pub fn data_size(&self) -> u32 {
        unsafe { sys::fsNetworkHttpRequestGetDataSize(self.0) }
    }

    pub fn data(&self) -> Option<OwnedCVec> {
        let data_size = self.data_size() as usize;
        if data_size == 0 {
            return None;
        }
        unsafe {
            let data = sys::fsNetworkHttpRequestGetData(self.0);
            if data.is_null() {
                None
            } else {
                Some(OwnedCVec::from_raw(data, data_size))
            }
        }
    }

    pub fn error_code(&self) -> i32 {
        unsafe { sys::fsNetworkHttpRequestGetErrorCode(self.0) }
    }

    pub fn state(&self) -> sys::FsNetworkHttpRequestState {
        unsafe { sys::fsNetworkHttpRequestGetState(self.0) }
    }

    pub fn header_section(&self, section: &str) -> Option<OwnedCStr> {
        let section = CString::new(section).ok()?;
        unsafe {
            let a = sys::fsNetworkHttpRequestGetHeaderSection(self.0, section.as_ptr());
            if a.is_null() {
                None
            } else {
                Some(OwnedCStr::from_ptr(a))
            }
        }
    }
}

pub struct OwnedCStr<'a>(&'a CStr);
impl OwnedCStr<'_> {
    fn from_ptr(data: *const ffi::c_char) -> Self {
        Self(unsafe { CStr::from_ptr(data) })
    }
}
impl std::ops::Deref for OwnedCStr<'_> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl Drop for OwnedCStr<'_> {
    fn drop(&mut self) {
        // SAFETY: the CStr itself will be droped right afterwards which doesn't have any deconstructor
        unsafe {
            libc::free(self.0.as_ptr() as *mut ffi::c_void);
        }
    }
}

pub struct OwnedCVec {
    data: *const u8,
    size: usize,
}
impl OwnedCVec {
    fn from_raw(data: *const u8, size: usize) -> Self {
        Self { data, size }
    }
}
impl std::ops::Deref for OwnedCVec {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }
}
impl Drop for OwnedCVec {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.data as *mut ffi::c_void);
        }
    }
}
