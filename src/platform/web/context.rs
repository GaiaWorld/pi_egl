
pub struct WebContext;

unsafe impl Sync for WebContext {}
unsafe impl Send for WebContext {}

impl Drop for WebContext {
    fn drop(&mut self) {

    }
}
