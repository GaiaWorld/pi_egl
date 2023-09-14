use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::{Context, PowerPreference, Surface};

// use crate::GL;
#[cfg(target_os = "windows")]
use crate::platform::windows::instance::WglInstance as InstanceInner;

#[cfg(target_arch = "wasm32")]
use crate::platform::web::instance::WebInstance as InstanceInner;

#[cfg(target_os = "android")]
use crate::platform::android::instance::EglInstance as InstanceInner;

#[derive(Debug)]
pub struct Instance {
    instance: InstanceInner,
}

unsafe impl Sync for Instance {}
unsafe impl Send for Instance {}

impl Drop for Instance {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {}
    }
}

impl Instance {
    // Display 封装在里面，不对外曝露
    // is_vsync: SwapBuffers 是否 重置同步
    pub fn new(power: PowerPreference, is_vsync: bool) -> Result<Self, InstanceError> {
        // Windows下: LowPower 集显, HighPerformance 独显
        {
            Ok(Self {
                instance: InstanceInner::new(power, is_vsync)?,
            })
        }
    }

    // 带双缓冲的 Surface
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<Surface, InstanceError> {
        {
            let surface = self.instance.create_surface(window)?;
            Ok(Surface { surface })
        }
    }

    // GLES 3.0 / WebGL2
    pub fn create_context(&self) -> Result<Context, InstanceError> {
        {
            let context = self.instance.create_context()?;
            Ok(Context { context })
        }
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current<'a>(
        &'a mut self,
        surface: Option<&'a Surface>,
        context: Option<&Context>,
    ) -> Option<&glow::Context> {
        {
            let mut s = None;
            if let Some(t) = surface {
                s = Some(&t.surface)
            }

            let mut c = None;
            if let Some(t) = context {
                c = Some(&t.context)
            }

            self.instance.make_current(s, c)
        }
    }

    // 交换 Surface 中的 双缓冲
    // wasm32 cfg 空实现
    pub fn swap_buffers(&self, surface: &Surface) {
        {
            self.instance.swap_buffers(&surface.surface)
        }
    }
}

#[derive(Debug)]
pub enum InstanceError {
    // TODO
    RequiredExtensionUnavailable,
    IncompatibleWindowHandle,
    ContextCreationFailed,
}

// impl Display for InstanceError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {

//         }
//     }
// }
