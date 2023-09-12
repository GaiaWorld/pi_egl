use std::fmt::Display;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

use super::{Context, PowerPreference, Surface};

use crate::GL;
#[cfg(target_os = "windows")]
use crate::platform::windows::instance::WglInstance as InstanceInner;

pub struct Instance {
    #[cfg(not(target_arch = "wasm32"))]
    instance: InstanceInner, // #[cfg(not(target_arch = "wasm32"))]
                             // TODO
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
    
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            let surface = self.instance.create_surface(window)?;
            Ok(Surface { surface })
        }
    }

    // GLES 3.0 / WebGL2
    pub fn create_context<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<Context, InstanceError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let context = self.instance.create_context(window)?;
            Ok(Context { context })
        }
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current(&self, surface: Option<&Surface>, context: Option<&Context>) -> Option<&GL>{
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
