#[cfg(target_os = "windows")]
use crate::platform::windows::context::WglContext as ContextInner;

#[cfg(target_os = "android")]
use crate::platform::android::context::EglContext as ContextInner;

#[cfg(target_arch = "wasm32")]
use crate::platform::web::context::WebContext as ContextInner;

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
