#[cfg(target_os = "windows")]
use crate::platform::windows::context::WglContext as ContextInner;

#[cfg(any(target_os = "linux", target_os = "android"))]
use crate::platform::android::context::EglContext as ContextInner;

#[cfg(target_arch = "wasm32")]
use crate::platform::web::context::WebContext as ContextInner;

#[derive(Debug, Eq, PartialEq)]
pub struct Context {
    pub context: ContextInner,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        {}
    }
}
