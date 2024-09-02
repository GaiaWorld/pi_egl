use crate::platform::android::egl::types::{EGLDisplay, EGLSurface};

use super::util::EGL_FUNCTIONS;

#[derive(Debug, Eq, PartialEq)]
pub struct EglSurface {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) egl_surface: EGLSurface,
    pub(crate) egl_display: EGLDisplay,
}

unsafe impl Sync for EglSurface {}
unsafe impl Send for EglSurface {}

impl Drop for EglSurface {
    fn drop(&mut self) {
        let egl = &EGL_FUNCTIONS.0;
        // todo： 安卓某些设备释放这个会导致崩溃
        // unsafe { egl.DestroySurface(self.egl_display, self.egl_surface) };
    }
}
