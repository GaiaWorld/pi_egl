pub struct Surface {
    // #[cfg(not(target_arch = "wasm32"))]
    // TODO
}

unsafe impl Sync for Surface {}
unsafe impl Send for Surface {}

impl Drop for Surface {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }
}
