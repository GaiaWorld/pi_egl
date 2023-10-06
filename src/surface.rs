#[cfg(target_os = "windows")]
use crate::platform::windows::surface::WglSurface as SurfaceInner;

#[cfg(target_os = "android")]
use crate::platform::android::surface::EglSurface as SurfaceInner;

#[cfg(target_arch = "wasm32")]
use crate::platform::web::surface::WebSurface as SurfaceInner;

#[derive(Debug, Eq, PartialEq)]
pub struct Surface {
    pub(crate) surface: SurfaceInner
}

unsafe impl Sync for Surface {}
unsafe impl Send for Surface {}
