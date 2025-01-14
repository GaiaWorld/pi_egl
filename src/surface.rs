use pi_share::Share;

#[cfg(target_os = "windows")]
use crate::platform::windows::surface::WglSurface as SurfaceInner;

#[cfg(target_os = "linux")]
use crate::platform::android::surface::EglSurface as SurfaceInner;

#[cfg(target_arch = "wasm32")]
use crate::platform::web::surface::WebSurface as SurfaceInner;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Surface {
    pub(crate) surface: Share<SurfaceInner>,
}

unsafe impl Sync for Surface {}
unsafe impl Send for Surface {}
