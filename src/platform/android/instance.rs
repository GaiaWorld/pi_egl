#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle, XlibWindowHandle};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[cfg(feature = "swappy")]
use super::swappy::{
    SwappyGL_destroy, SwappyGL_enableStats, SwappyGL_init, SwappyGL_isEnabled,
    SwappyGL_setAutoPipelineMode, SwappyGL_setAutoSwapInterval, SwappyGL_setSwapIntervalNS,
    SwappyGL_setWindow, SwappyGL_swap,
};

use super::{context::EglContext, surface::EglSurface, util::EGL_FUNCTIONS};
use crate::{
    platform::android::egl::{self, EGLint},
    platform::android::{egl::types::EGLDisplay, util::egl_config_from_display},
    InstanceError, PowerPreference,
};
lazy_static! {
    /// 全局存储EGL表面（surface）的指针，使用读写锁确保线程安全。
    static ref SURFACE_PTR: std::sync::RwLock<u64> = std::sync::RwLock::new(0);
}

/// OpenGL ES 实例，管理 EGL 显示和上下文。
#[derive(Debug)]
pub struct EglInstance {
    /// EGL 显示，表示与显示系统的连接。
    display: EGLDisplay,
    /// OpenGL ES 上下文，用于渲染。
    context: Option<glow::Context>,
    /// 是否启用了垂直同步（VSync）。
    is_vsync: bool,
    /// 帧率统计功能，仅在 `fps` 特性启用时存在。
    #[cfg(feature = "fps")]
    fps: AtomicU32,
    /// 时间记录，用于计算帧率，仅在 `fps` 特性启用时存在。
    #[cfg(feature = "fps")]
    time: pi_share::ShareCell<std::time::Instant>,
}

impl Drop for EglInstance {
    /// 实现 `Drop` 特征，负责释放资源。
    fn drop(&mut self) {
        /// 获取 EGL 函数指针。
        let egl = &EGL_FUNCTIONS.0;
        unsafe {
            /// 终止 EGL 显示连接。
            let result = egl.Terminate(self.display);
            /// 确保终止操作成功。
            assert_ne!(result, egl::FALSE);
            /// 重置显示为无效值。
            self.display = egl::NO_DISPLAY;
            /// 如果启用了 `swappy` 特性，销毁 SwappyGL。
            #[cfg(feature = "swappy")]
            {
                SwappyGL_destroy();
            }
        }
    }
}
impl EglInstance {
    /// 创建新的 EGL 实例。
    ///
    /// # 参数
    /// - `_power`: 表示电源偏好（未使用，保留供未来扩展）。
    /// - `is_vsync`: 是否启用垂直同步（VSync）功能。
    ///
    /// # 返回值
    /// - `Result<Self, InstanceError>`: 创建成功则返回 `EglInstance` 实例，失败则返回错误信息。
    pub fn new(_power: PowerPreference, is_vsync: bool) -> Result<Self, InstanceError> {
        #[cfg(feature = "swappy")]
        {
            let _ = swappy_init();
        }

        let egl = &EGL_FUNCTIONS.0;

        unsafe {
            let egl_display = egl.GetDisplay(egl::DEFAULT_DISPLAY);
            assert_ne!(egl_display, egl::NO_DISPLAY, "无法获取默认显示");

            // 初始化 EGL
            let (mut major_version, mut minor_version) = (0, 0);
            let result = egl.Initialize(egl_display, &mut major_version, &mut minor_version);
            assert_ne!(result, egl::FALSE, "EGL 初始化失败");

            Ok(EglInstance {
                display: egl_display,
                context: None,
                is_vsync,
                #[cfg(feature = "fps")]
                fps: AtomicU32::new(0),
                #[cfg(feature = "fps")]
                time: pi_share::ShareCell::new(std::time::Instant::now()),
            })
        }
    }

    /// 创建带双缓冲的 EGL 表面（Surface）。
    ///
    /// # 参数
    /// - `window`: 窗口句柄，必须实现 `HasRawWindowHandle` 和 `HasRawDisplayHandle`。
    ///
    /// # 返回值
    /// - `Result<EglSurface, InstanceError>`: 创建成功则返回 `EglSurface` 实例，失败则返回错误信息。
    #[allow(deprecated)]
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<EglSurface, InstanceError> {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.display;

        // 获取本地窗口句柄
        let handle = window.raw_window_handle();
        let native_window = if let Ok(RawWindowHandle::AndroidNdk(handle)) = handle {
            handle.a_native_window.as_ptr()
        } else if let Ok(RawWindowHandle::Xlib(XlibWindowHandle{window, .. })) = handle {
            window as *mut c_void
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };
        unsafe {
            #[cfg(feature = "swappy")]
            {
                SwappyGL_setWindow(native_window);
                let enable = SwappyGL_isEnabled();
                println!("SwappyGL 启用状态: {}", enable);
            }

            // 获取适合的 EGL 配置
            let egl_config = egl_config_from_display(egl_display);

            // 创建窗口表面
            let attributes = [egl::NONE as EGLint];
            let mut egl_surface = egl.CreateWindowSurface(
                egl_display,
                egl_config,
                native_window,
                attributes.as_ptr(),
            );
            if egl_surface == egl::NO_SURFACE {
                // 检查是否有缓存的 surface指针
                let ptr = *SURFACE_PTR.read().unwrap();
                if ptr != 0 {
                    egl_surface =  ptr as *mut c_void;
                }
            } else {
                // 缓存创建的 surface 指针
                *SURFACE_PTR.write().unwrap() = egl_surface as u64;
            }

            assert_ne!(egl_surface, egl::NO_SURFACE, "创建窗口表面失败");
            // 获取表面尺寸
            let mut width = 0;
            let mut height = 0;
            egl.QuerySurface(egl_display, egl_surface, egl::WIDTH as EGLint, &mut width);
            egl.QuerySurface(egl_display, egl_surface, egl::HEIGHT as EGLint, &mut height);
            assert_ne!(width, 0, "表面宽度为0");
            assert_ne!(height, 0, "表面高度为0");
            Ok(EglSurface {
                width,
                height,
                egl_surface,
                egl_display,
            })
        }
    }

    /// 创建 OpenGL ES 上下文。
    ///
    /// # 返回值
    /// - `Result<EglContext, InstanceError>`: 创建成功则返回 `EglContext` 实例，失败则返回错误信息。
    #[allow(non_snake_case)]
    pub fn create_context(&self) -> Result<EglContext, InstanceError> {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.display;

        unsafe {
            // 绑定 OpenGL ES API
            egl.BindAPI(egl::OPENGL_ES_API);

            // 获取适合的 EGL 配置
            let egl_config = egl_config_from_display(egl_display);

            // 上下文属性
            let egl_context_attributes = [
                egl::CONTEXT_CLIENT_VERSION as EGLint,
                3, // 请求 OpenGL ES 3.0
                egl::NONE as EGLint,
            ];

            // 创建上下文
            let egl_context = egl.CreateContext(
                egl_display,
                egl_config,
                std::ptr::null_mut(),
                egl_context_attributes.as_ptr(),
            );

            if egl_context == egl::NO_CONTEXT {
                let _err = egl.GetError();
                return Err(InstanceError::ContextCreationFailed);
        }

            Ok(EglContext {
                egl_context,
                egl_display,
            })
        }
    }

    /// 将上下文绑定到目标表面，使其成为当前上下文。
    /// 在 WASM 环境中为空实现。
    ///
    /// # 参数
    /// - `surface`: 可选的表面，如果为 `None`，则解除绑定。
    /// - `context`: 可选的上下文，如果为 `None`，则清除当前上下文。
    pub fn make_current(&mut self, surface: Option<&EglSurface>, context: Option<&EglContext>) {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.display;

        if let Some(context) = context {
            if let Some(surface) = surface {
                // 绑定到目标表面和上下文
                let ok = unsafe {
                    egl.MakeCurrent(
                        egl_display,
                        surface.egl_surface,
                        surface.egl_surface,
                        context.egl_context,
                    )
                };
                assert_ne!(ok, egl::FALSE, "绑定上下文失败");

                // 禁用 VSync（如果需要）
                if !self.is_vsync {
                    let ok = unsafe { egl.SwapInterval(egl_display, 0) };
                    if ok != egl::TRUE {
                        println!("禁用 VSync 失败。错误码: {}", ok);
                    }
                }
            } else {
                // 只绑定到上下文，而不绑定到表面
                let ok = unsafe {
                    egl.MakeCurrent(
                        egl_display,
                        egl::NO_SURFACE,
                        egl::NO_SURFACE,
                        context.egl_context,
                    )
                };
                assert_ne!(ok, egl::FALSE, "绑定上下文失败");
            }

            // 初始化 Glow 上下文
            if self.context.is_none() {
                let context = unsafe {
                    glow::Context::from_loader_function(|symbol_name| get_gl_address(symbol_name))
                };
                let _ = self.context.replace(context);
            }
        } else {
            // 清除当前上下文
            unsafe {
                let ok = egl.MakeCurrent(
                    egl_display,
                    egl::NO_SURFACE,
                    egl::NO_SURFACE,
                    egl::NO_CONTEXT,
                );
                assert_ne!(ok, egl::FALSE, "清除当前上下文失败");
            }
        }
    }

    /// 获取 Glow OpenGL 上下文。
    ///
    /// # 返回值
    /// - `&glow::Context`: Returns the OpenGL context.
    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.context.as_ref().unwrap()
    }

    /// 交换双缓冲区的内容，将后缓冲区显示到屏幕。
    /// 在 WASM 环境中为空实现。
    ///
    /// # 参数
    /// - `surface`: 需要交换缓冲区的表面。
    pub fn swap_buffers(&self, surface: &EglSurface) {
        let egl_display = self.display;
        #[cfg(feature = "swappy")]
        {
            unsafe { SwappyGL_swap(egl_display, surface.egl_surface) };
        }
        #[cfg(not(feature = "swappy"))]
        {
            let egl = &EGL_FUNCTIONS.0;
            unsafe { egl.SwapBuffers(egl_display, surface.egl_surface) };
        }

        #[cfg(feature = "fps")]
        {
            // 增加帧率计数器
            self.fps.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // 检查是否需要更新 FPS 显示
            let mut time = self.time.borrow_mut();
            if time.elapsed().as_millis() > 1000 {
                println!("PI_EGL FPS: {:?}", self.fps);
                self.fps.store(0, std::sync::atomic::Ordering::Relaxed);
                *time = std::time::Instant::now();
            }
        }
        
    }
}

fn get_gl_address(symbol_name: &str) -> *const c_void {
    unsafe {
        let egl = &EGL_FUNCTIONS.0;
        let symbol_name: CString = CString::new(symbol_name).unwrap();
        let v =
            egl.GetProcAddress(symbol_name.as_ptr() as *const u8 as *const c_char) as *const c_void;
        // println!("gl symbol_name {:?} ptr is {:?}!!", symbol_name, v);
        v
    }
}

#[cfg(feature = "swappy")]
fn swappy_init() -> Result<(), InstanceError> {
    let native_activity = ndk_glue::native_activity();
    let vm_ptr = native_activity.vm();
    let vm = match unsafe { jni::JavaVM::from_raw(vm_ptr) } {
        Ok(vm) => vm,
        Err(err) => {
            println!(
                "====== on_app_start: get java_vm failed!! reason: {:?}",
                err
            );
            return Err(InstanceError::JNIFailed);
        }
    };

    let env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(err) => {
            println!(
                "====== on_app_start: get jni_env failed!! reason: {:?}",
                err
            );
            return Err(InstanceError::JNIFailed);
        }
    };

    unsafe {
        SwappyGL_init(*env, native_activity.activity());
        SwappyGL_setAutoSwapInterval(0);
        SwappyGL_setAutoPipelineMode(0);
        SwappyGL_enableStats(0);
        SwappyGL_setSwapIntervalNS(1000000000 / 120);
    }
    vm.detach_current_thread();
    Ok(())
}
