#[derive(Debug, Eq, PartialEq)]
pub struct WglSurface(pub u64);

impl Drop for WglSurface {
    #[inline]
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // todo!()
        }
    }
}
