use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WebSurface {
    pub context: Arc<glow::Context>,
    pub id: u64,
}

unsafe impl Sync for WebSurface {}
unsafe impl Send for WebSurface {}

impl Drop for WebSurface {
    fn drop(&mut self) {}
}

impl PartialEq for WebSurface {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for WebSurface {}
