
pub struct WebContext(pub web_sys::WebGl2RenderingContext);

unsafe impl Sync for WebContext {}
unsafe impl Send for WebContext {}

impl Drop for WebContext {
    fn drop(&mut self) {

    }
}
