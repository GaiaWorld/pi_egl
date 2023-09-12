#[cfg(target_os = "windows")]
use crate::platform::windows::context::WglContext as WglContextInner;

pub struct Context {
    
    #[cfg(not(target_arch = "wasm32"))]
    pub context: WglContextInner,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
        
        }
    }
}
