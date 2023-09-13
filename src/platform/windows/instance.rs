use std::os::raw::c_int;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
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

use crate::{platform::windows::util::set_dc_pixel_format, InstanceError, PowerPreference};

use super::{
    context::WglContext,
    surface::WglSurface,
    util::{get_proc_address, set_exported_variables, HiddenWindow, WGL_EXTENSION_FUNCTIONS},
};

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

pub struct WglInstance(Option<glow::Context>, HWND);

impl Drop for WglInstance {
    fn drop(&mut self) {
        let _ = unsafe { winuser::DestroyWindow(self.1) };
    }
}

impl WglInstance {
    pub fn new(power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        set_exported_variables(power);
        Ok(WglInstance(None, HiddenWindow::create()))
    }

    // 带双缓冲的 Surface
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WglSurface, InstanceError> {
        let real_dc = if let RawWindowHandle::Win32(handle) = window.raw_window_handle() {
            unsafe { winuser::GetDC(handle.hwnd as HWND) }
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };

        let context_dc =  unsafe { winuser::GetDC(self.1 as HWND) };
        let pixel_format = unsafe { wingdi::GetPixelFormat(context_dc) };
        set_dc_pixel_format(real_dc, pixel_format);

        Ok(WglSurface(real_dc as u64))
    }

    #[allow(non_snake_case)]
    pub fn create_context(&self) -> Result<WglContext, InstanceError> {
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

        let real_dc = unsafe { winuser::GetDC(self.1) };
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

        // // Specify that we want to create an OpenGL 3.3 core profile context
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
        let err = unsafe { GetLastError() };
        if gl33_context.is_null() {
            return Err(InstanceError::ContextCreationFailed);
        }

        Ok(WglContext(gl33_context as u64))
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current(
        &mut self,
        surface: Option<&WglSurface>,
        context: Option<&WglContext>,
    ) -> Option<&glow::Context> {
        if let Some(context) = context {
            if let Some(surface) = surface {
                let ok = unsafe { wglMakeCurrent(surface.0 as HDC, context.0 as HGLRC) };
                // set_dc_pixel_format(dc, pixel_format)
                assert_ne!(ok, FALSE);
                if self.0.is_none() {
                    let gl = unsafe {
                        glow::Context::from_loader_function(|symbol_name| {
                            get_proc_address(symbol_name)
                        })
                    };
                    self.0.replace(gl);
                }

                return Some(self.0.as_ref().unwrap());
            } else {
                let ok = unsafe { wglMakeCurrent(std::ptr::null_mut(), context.0 as HGLRC) };
                assert_ne!(ok, FALSE);
            }
        } else {
            let ok = unsafe { wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut()) };
            assert_ne!(ok, FALSE);
        }
        None
    }

    // 交换 Surface 中的 双缓冲
    // wasm32 cfg 空实现
    pub fn swap_buffers(&self, surface: &WglSurface) {
        unsafe { SwapBuffers(surface.0 as HDC) };
    }
}
