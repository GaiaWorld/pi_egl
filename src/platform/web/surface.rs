pub struct WebSurface;

unsafe impl Sync for WebSurface {}
unsafe impl Send for WebSurface {}

impl Drop for WebSurface {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // todo!()
        }
    }
    
}