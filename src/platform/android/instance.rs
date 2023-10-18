use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
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

#[derive(Debug)]
pub struct EglInstance(EGLDisplay, Option<glow::Context>);

impl Drop for EglInstance {
    fn drop(&mut self) {
        let egl = &EGL_FUNCTIONS.0;
        unsafe {
            let result = egl.Terminate(self.0);
            assert_ne!(result, egl::FALSE);
            self.0 = egl::NO_DISPLAY;
            #[cfg(feature = "swappy")]
            {
                SwappyGL_destroy();
            }
        }
    }
}

impl EglInstance {
    pub fn new(_power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        #[cfg(feature = "swappy")]
        {
            let _ = swappy_init();
        }

        let egl = &EGL_FUNCTIONS.0;

        unsafe {
            let egl_display = egl.GetDisplay(egl::DEFAULT_DISPLAY);
            assert_ne!(egl_display, egl::NO_DISPLAY);

            // I don't think this should ever fail.
            let (mut major_version, mut minor_version) = (0, 0);
            let result = egl.Initialize(egl_display, &mut major_version, &mut minor_version);
            assert_ne!(result, egl::FALSE);

            Ok(EglInstance(egl_display, None))
        }
    }

    // 带双缓冲的 Surface
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<EglSurface, InstanceError> {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.0;
        let native_window = if let RawWindowHandle::AndroidNdk(handle) = window.raw_window_handle()
        {
            handle.a_native_window
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };
        unsafe {
            #[cfg(feature = "swappy")]
            {
                SwappyGL_setWindow(native_window);
                let enable = SwappyGL_isEnabled();
                println!("======SwappyGL enable: {}", enable);
            }

            let egl_config = egl_config_from_display(egl_display);

            let attributes = [egl::NONE as EGLint];
            let egl_surface = egl.CreateWindowSurface(
                egl_display,
                egl_config,
                native_window,
                attributes.as_ptr(),
            );

            assert_ne!(egl_surface, egl::NO_SURFACE);

            let mut width = 0;
            let mut height = 0;
            egl.QuerySurface(egl_display, egl_surface, egl::WIDTH as EGLint, &mut width);
            egl.QuerySurface(egl_display, egl_surface, egl::HEIGHT as EGLint, &mut height);
            assert_ne!(width, 0);
            assert_ne!(height, 0);

            Ok(EglSurface {
                width,
                height,
                egl_surface,
                egl_display,
            })
        }
    }

    #[allow(non_snake_case)]
    pub fn create_context(&self) -> Result<EglContext, InstanceError> {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.0;
        // Now we can choose a pixel format the modern way, using wglChoosePixelFormatARB.
        unsafe {
            egl.BindAPI(egl::OPENGL_ES_API);

            let egl_config = egl_config_from_display(egl_display);

            let egl_context_attributes = [
                egl::CONTEXT_CLIENT_VERSION as EGLint,
                2, // Request opengl ES2.0
                egl::NONE as EGLint,
            ];

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

    // 调用了这个之后，gl的函数才能用；
    // wasm32 cfg 空实现
    pub fn make_current(&mut self, surface: Option<&EglSurface>, context: Option<&EglContext>) {
        let egl = &EGL_FUNCTIONS.0;
        let egl_display = self.0;

        if let Some(context) = context {
            if let Some(surface) = surface {
                let ok = unsafe {
                    egl.MakeCurrent(
                        egl_display,
                        surface.egl_surface,
                        surface.egl_surface,
                        context.egl_context,
                    )
                };

                assert_ne!(ok, egl::FALSE);
            } else {
                let ok = unsafe {
                    egl.MakeCurrent(
                        egl_display,
                        egl::NO_SURFACE,
                        egl::NO_SURFACE,
                        context.egl_context,
                    )
                };
                assert_ne!(ok, egl::FALSE);
            }

            if self.1.is_none() {
                let context = unsafe {
                    glow::Context::from_loader_function(|symbol_name| get_gl_address(symbol_name))
                };
                let _ = self.1.replace(context);
            }
        } else {
            unsafe {
                let ok = egl.MakeCurrent(
                    egl_display,
                    egl::NO_SURFACE,
                    egl::NO_SURFACE,
                    egl::NO_CONTEXT,
                );
                assert_ne!(ok, egl::FALSE);
            }
        }
    }

    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.1.as_ref().unwrap()
    }

    // 交换 Surface 中的 双缓冲
    // wasm32 cfg 空实现
    pub fn swap_buffers(&self, surface: &EglSurface) {
        let egl_display = self.0;
        #[cfg(feature = "swappy")]
        {
            unsafe { SwappyGL_swap(egl_display, surface.egl_surface) };
        }
        #[cfg(not(feature = "swappy"))]
        {
            let egl = &EGL_FUNCTIONS.0;
            unsafe { egl.SwapBuffers(egl_display, surface.egl_surface) };
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
