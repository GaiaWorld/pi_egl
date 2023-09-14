use std::ptr;

use winapi::{
    shared::windef::HGLRC,
    um::wingdi::{wglDeleteContext, wglGetCurrentContext, wglMakeCurrent},
};

#[derive(Debug)]
pub struct WglContext(pub u64);

impl Drop for WglContext {
    fn drop(&mut self) {
        if unsafe { wglGetCurrentContext() } == self.0 as HGLRC {
            unsafe { wglMakeCurrent(ptr::null_mut(), ptr::null_mut()) };
        }

        unsafe { wglDeleteContext(self.0 as HGLRC) };
    }
}
