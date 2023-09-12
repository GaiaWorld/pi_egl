#[cfg(target_os = "windows")]
use crate::platform::windows::surface::WglSurface as WglSurfaceInner;

pub struct Surface {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) surface: WglSurfaceInner
}

unsafe impl Sync for Surface {}
unsafe impl Send for Surface {}

impl Drop for Surface {
    fn drop(&mut self) {
        // #[cfg(not(target_arch = "wasm32"))]
        // {
        //     todo!()
        // }
    }
}
