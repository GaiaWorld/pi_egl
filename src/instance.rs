use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

use super::{Context, PowerPreference, Surface};

pub struct Instance {
    // #[cfg(not(target_arch = "wasm32"))]
    // TODO
    
    // #[cfg(not(target_arch = "wasm32"))]
    // TODO
}

unsafe impl Sync for Instance {}
unsafe impl Send for Instance {}

impl Drop for Instance {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }
}

impl Instance {
    // Display 封装在里面，不对外曝露
    // is_vsync: SwapBuffers 是否 重置同步
    pub fn new(power: PowerPreference, is_vsync: bool) -> Result<Self, InstanceError> {
        // Windows下: LowPower 集显, HighPerformance 独显
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }

    // 带双缓冲的 Surface
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &W,
    ) -> Result<Surface, InstanceError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }

    // GLES 3.0 / WebGL2
    pub fn create_context(&self) -> Result<Context, InstanceError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current(&self, surface: Option<&Surface>, context: Option<&Context>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }

    // 交换 Surface 中的 双缓冲
    // wasm32 cfg 空实现
    pub fn swap_buffers(&self, surface: &Surface) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            todo!()
        }
    }
}

#[derive(Debug, Error)]
pub enum InstanceError {
    // TODO
}
