use std::os::raw::c_int;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use winapi::{
    shared::{
        minwindef::{FALSE, UINT},
        windef::{HDC, HGLRC, HWND},
    },
    um::{
        errhandlingapi::GetLastError,
        wingdi::{self, wglMakeCurrent, SwapBuffers, PIXELFORMATDESCRIPTOR},
        winuser,
    },
};

use super::{
    context::WglContext,
    surface::WglSurface,
    util::{get_proc_address, set_exported_variables, HiddenWindow, WGL_EXTENSION_FUNCTIONS},
};
use crate::{platform::windows::util::set_dc_pixel_format, InstanceError, PowerPreference};

type GLenum = u32;

const WGL_DRAW_TO_WINDOW_ARB: GLenum = 0x2001;
const WGL_ACCELERATION_ARB: GLenum = 0x2003;
const WGL_SUPPORT_OPENGL_ARB: GLenum = 0x2010;
const WGL_DOUBLE_BUFFER_ARB: GLenum = 0x2011;
const WGL_PIXEL_TYPE_ARB: GLenum = 0x2013;
const WGL_COLOR_BITS_ARB: GLenum = 0x2014;
// const WGL_ALPHA_BITS_ARB: GLenum = 0x201b;
const WGL_DEPTH_BITS_ARB: GLenum = 0x2022;
const WGL_STENCIL_BITS_ARB: GLenum = 0x2023;
const WGL_FULL_ACCELERATION_ARB: GLenum = 0x2027;
const WGL_TYPE_RGBA_ARB: GLenum = 0x202b;
const WGL_CONTEXT_MAJOR_VERSION_ARB: GLenum = 0x2091;
const WGL_CONTEXT_MINOR_VERSION_ARB: GLenum = 0x2092;
const WGL_CONTEXT_PROFILE_MASK_ARB: GLenum = 0x9126;
const WGL_ALPHA_BITS_ARB: GLenum = 0x201b;

const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: GLenum = 0x00000001;
// const WGL_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB: GLenum = 0x00000002;

#[derive(Debug)]
pub struct WglInstance {
    context: Option<glow::Context>, // 可选的OpenGL上下文（通过glow库管理）

    window_hwnd: HWND, // 隐藏窗口的句柄
    window_hdc: HDC, // 隐藏窗口的设备上下文句柄

    is_vsync: bool, // 是否启用垂直同步

    #[cfg(feature = "fps")]
    fps: AtomicU32, // (条件编译): 帧率计数器
    #[cfg(feature = "fps")]
    time: pi_share::ShareCell<std::time::Instant>, // (条件编译): 用于计算FPS的时间戳
}

impl Drop for WglInstance {
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { winuser::DestroyWindow(self.window_hwnd) };
    }
}

impl WglInstance {
    /// 创建一个新的WGL实例
    ///
    /// # 参数
    /// - `power`: 显卡性能偏好（如高性能或节能）
    /// - `is_vsync`: 是否启用垂直同步
    ///
    /// # 返回值
    /// - `Ok(Self)`: 成功创建的实例
    /// - `Err(InstanceError)`: 创建失败的错误类型
    ///
    /// # 注意
    /// 内部会创建一个隐藏窗口用于初始化OpenGL环境
    #[inline]
    pub fn new(power: PowerPreference, is_vsync: bool) -> Result<Self, InstanceError> {
        log::error!("new");
        set_exported_variables(power);

        let window_hwnd = HiddenWindow::create();
        let window_hdc = unsafe { winuser::GetDC(window_hwnd) };

        Ok(WglInstance {
            context: None,
            window_hwnd,
            window_hdc,

            is_vsync,
            #[cfg(feature = "fps")]
            fps: AtomicU32::new(0),
            #[cfg(feature = "fps")]
            time: pi_share::ShareCell::new(std::time::Instant::now()),
        })
    }

    /// 为指定窗口创建渲染表面
    ///
    /// # 参数
    /// - `window`: 实现了`HasWindowHandle`和`HasDisplayHandle` trait的窗口对象
    ///
    /// # 返回值
    /// - `Ok(WglSurface)`: 成功创建的渲染表面
    /// - `Err(InstanceError::IncompatibleWindowHandle)`: 窗口句柄类型不兼容
    ///
    /// # 安全要求
    /// 传入的窗口必须持有有效的Win32窗口句柄
    #[inline]
    pub fn create_surface<W: HasWindowHandle + HasDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WglSurface, InstanceError> {
        log::error!("create_surface");
        let real_dc = if let Ok(h) = window.window_handle() {
			if let RawWindowHandle::Win32(handle) = h.as_raw() {
				unsafe { winuser::GetDC((handle.hwnd.get()) as HWND) }
			} else {
				return Err(InstanceError::IncompatibleWindowHandle);
			}
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };

        let context_dc = unsafe { winuser::GetDC(self.window_hwnd as HWND) };
        let pixel_format = unsafe { wingdi::GetPixelFormat(context_dc) };
        set_dc_pixel_format(real_dc, pixel_format);

        Ok(WglSurface(real_dc as u64))
    }

    /// 创建OpenGL 3.3核心配置文件上下文
    ///
    /// # 返回值
    /// - `Ok(WglContext)`: 成功创建的OpenGL上下文
    /// - `Err(InstanceError::RequiredExtensionUnavailable)`: 缺少必要WGL扩展
    /// - `Err(InstanceError::ContextCreationFailed)`: 上下文创建失败
    ///
    /// # 像素格式属性
    /// 使用32位颜色、24位深度缓冲、8位模板缓冲、8位Alpha通道
    #[allow(non_snake_case)]
    pub fn create_context(&self) -> Result<WglContext, InstanceError> {
        log::error!("create_context");
        let pixel_format_attribs = [
            WGL_DRAW_TO_WINDOW_ARB as c_int,
            1 as c_int,
            WGL_SUPPORT_OPENGL_ARB as c_int,
            1 as c_int,
            WGL_DOUBLE_BUFFER_ARB as c_int,
            1 as c_int,
            WGL_ACCELERATION_ARB as c_int,
            WGL_FULL_ACCELERATION_ARB as c_int,
            WGL_PIXEL_TYPE_ARB as c_int,
            WGL_TYPE_RGBA_ARB as c_int,
            WGL_COLOR_BITS_ARB as c_int,
            32,
            WGL_DEPTH_BITS_ARB as c_int,
            24,
            WGL_STENCIL_BITS_ARB as c_int,
            8,
            WGL_ALPHA_BITS_ARB as c_int,
            8,
            0,
        ];
        // int pixel_format;
        // UINT num_formats;

        let wglChoosePixelFormatARB = match WGL_EXTENSION_FUNCTIONS.wglChoosePixelFormatARB {
            None => return Err(InstanceError::RequiredExtensionUnavailable),
            Some(ref func) => func,
        };

        let real_dc = unsafe { winuser::GetDC(self.window_hwnd) };
        let (mut pixel_format, mut pixel_format_count) = (0, 0);
        let ok = unsafe {
            wglChoosePixelFormatARB(
                real_dc,
                pixel_format_attribs.as_ptr(),
                std::ptr::null(),
                1,
                &mut pixel_format,
                &mut pixel_format_count,
            )
        };
        assert_ne!(ok, FALSE);

        let mut pixel_format_descriptor = unsafe { std::mem::zeroed() };
        unsafe {
            wingdi::DescribePixelFormat(
                real_dc,
                pixel_format,
                std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as UINT,
                &mut pixel_format_descriptor,
            )
        };
        let ok =
            unsafe { wingdi::SetPixelFormat(real_dc, pixel_format, &mut pixel_format_descriptor) };
        // println!()
        assert_ne!(ok, FALSE);

        // Specify that we want to create an OpenGL 3.3 core profile context
        let gl33_attribs = [
            WGL_CONTEXT_MAJOR_VERSION_ARB as c_int,
            3,
            WGL_CONTEXT_MINOR_VERSION_ARB as c_int,
            3,
            WGL_CONTEXT_PROFILE_MASK_ARB as c_int,
            WGL_CONTEXT_CORE_PROFILE_BIT_ARB as c_int,
            0,
        ];

        let wglCreateContextAttribsARB = match WGL_EXTENSION_FUNCTIONS.wglCreateContextAttribsARB {
            None => return Err(InstanceError::RequiredExtensionUnavailable),
            Some(ref func) => func,
        };

        let gl33_context = unsafe {
            wglCreateContextAttribsARB(real_dc, std::ptr::null_mut(), gl33_attribs.as_ptr())
        };

        if gl33_context.is_null() {
            return Err(InstanceError::ContextCreationFailed);
        }

        Ok(WglContext(gl33_context as u64))
    }

    /// 设置当前渲染上下文和表面
    ///
    /// # 参数
    /// - `surface`: 可选的要绑定的渲染表面
    /// - `context`: 可选的要激活的OpenGL上下文
    ///
    /// # Panics
    /// - 如果绑定操作失败会触发断言
    ///
    /// # 注意
    /// 当首次绑定上下文时会初始化glow上下文
    pub fn make_current(&mut self, surface: Option<&WglSurface>, context: Option<&WglContext>) {
        if let Some(context) = context {
            if let Some(surface) = surface {
                let ok = unsafe { wglMakeCurrent(surface.0 as HDC, context.0 as HGLRC) };
                // set_dc_pixel_format(dc, pixel_format)
                if !self.is_vsync {
                    if let Some(func) = WGL_EXTENSION_FUNCTIONS.wglSwapIntervalEXT {
                        let ok = unsafe { func(0) };
                        if ok == 0 {
                            let err = unsafe { GetLastError() };
                            println!("vsync closed failed!!! error code: {}", err);
                        } else {
                            // println!("vsync closed successfully!!!");
                        }
                    }
                }
                assert_ne!(ok, FALSE);
            } else {
                let ok = unsafe { wglMakeCurrent(self.window_hdc, context.0 as HGLRC) };
                assert_ne!(ok, FALSE);
            }
            if self.context.is_none() {
                let gl = unsafe {
                    glow::Context::from_loader_function(|symbol_name| get_proc_address(symbol_name))
                };
                self.context.replace(gl);
            }
        } else {
            let ok = unsafe { wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut()) };
            assert_ne!(ok, FALSE);
        }
    }

    /// 获取当前glow OpenGL上下文
    ///
    /// # Panics
    /// 如果上下文尚未初始化（未调用过make_current）
    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.context.as_ref().unwrap()
    }

    /// 交换前后缓冲区并处理垂直同步
    ///
    /// # 参数
    /// - `surface`: 要交换缓冲区的渲染表面
    ///
    /// # 功能
    /// - 当启用`fps`特性时，会自动计算并打印帧率
    #[inline]
    pub fn swap_buffers(&self, surface: &WglSurface) {
        unsafe { SwapBuffers(surface.0 as HDC) };

        #[cfg(feature = "fps")]
        {
            self.fps.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut time = self.time.borrow_mut();
            if time.elapsed().as_millis() > 1000 {
                println!("PI_EGL FPS: {:?}", self.fps);
                self.fps.store(0, std::sync::atomic::Ordering::Relaxed);
                *time = std::time::Instant::now();
            }
        }
    }
}
