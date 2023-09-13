use crate::platform::android::egl::types::{EGLContext, EGLDisplay};

use super::util::EGL_FUNCTIONS;

pub struct EglContext {
    pub egl_context: EGLContext,
    pub egl_display: EGLDisplay,
}

unsafe impl Sync for EglContext {}
unsafe impl Send for EglContext {}

impl Drop for EglContext {
    fn drop(&mut self) {
        let egl = &EGL_FUNCTIONS.0;
        let _ = unsafe { egl.DestroyContext(self.egl_display, self.egl_context) };
    }
}
