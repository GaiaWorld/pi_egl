pub struct Context {
    // #[cfg(not(target_arch = "wasm32"))]
    // TODO
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }
}
