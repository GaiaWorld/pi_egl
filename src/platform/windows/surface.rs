pub struct WglSurface(pub u64);

unsafe impl Sync for WglSurface {}
unsafe impl Send for WglSurface {}

impl Drop for WglSurface {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // todo!()
        }
    }
}
