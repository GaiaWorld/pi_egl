use std::ptr;

use winapi::{
    shared::windef::HGLRC,
    um::wingdi::{wglDeleteContext, wglGetCurrentContext, wglMakeCurrent},
};

#[derive(Debug, Eq, PartialEq)]
pub struct WglContext(pub u64);

impl Drop for WglContext {
    #[inline]
    fn drop(&mut self) {
        if unsafe { wglGetCurrentContext() } == self.0 as HGLRC {
            unsafe { wglMakeCurrent(ptr::null_mut(), ptr::null_mut()) };
        }

        unsafe { wglDeleteContext(self.0 as HGLRC) };
    }
}
