use std::{
    ffi::CString,
    mem,
    os::raw::{c_int, c_void},
    ptr, thread,
};

use log::warn;
use winapi::{
    shared::{
        minwindef::{BOOL, FALSE, FLOAT, HMODULE, LPARAM, LPVOID, LRESULT, UINT, WORD, WPARAM},
        ntdef::LPCSTR,
        windef::{HBRUSH, HDC, HGLRC, HWND},
    },
    um::{
        libloaderapi,
        wingdi::{
            self, wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent,
            PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL,
            PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
        },
        winuser::{
            self, COLOR_BACKGROUND, CREATESTRUCTA, CS_OWNDC, WNDCLASSA, WS_OVERLAPPEDWINDOW,
            WS_VISIBLE, WM_CREATE, CS_HREDRAW, CS_VREDRAW,
        }, errhandlingapi::GetLastError,
    },
};

use crate::PowerPreference;

static NVIDIA_GPU_SELECT_SYMBOL: &[u8] = b"NvOptimusEnablement\0";
static AMD_GPU_SELECT_SYMBOL: &[u8] = b"AmdPowerXpressRequestHighPerformance\0";

lazy_static! {
    pub(crate) static ref WGL_EXTENSION_FUNCTIONS: WGLExtensionFunctions =
        thread::spawn(extension_loader_thread).join().unwrap();
    pub(crate) static ref OPENGL_LIBRARY: u64 =
        { unsafe { libloaderapi::LoadLibraryA(&b"opengl32.dll\0"[0] as *const u8 as LPCSTR) } }
            as u64;
}

#[allow(non_snake_case)]
#[derive(Default)]
pub(crate) struct WGLExtensionFunctions {
    pub wglCreateContextAttribsARB: Option<
        unsafe extern "C" fn(hDC: HDC, shareContext: HGLRC, attribList: *const c_int) -> HGLRC,
    >,
    pub wglChoosePixelFormatARB: Option<
        unsafe extern "C" fn(
            hdc: HDC,
            piAttribIList: *const c_int,
            pfAttribFList: *const FLOAT,
            nMaxFormats: UINT,
            piFormats: *mut c_int,
            nNumFormats: *mut UINT,
        ) -> BOOL,
    >,
}

fn extension_loader_thread() -> WGLExtensionFunctions {
    unsafe {
        let instance = libloaderapi::GetModuleHandleA(ptr::null_mut());
        let window_class_name = &b"SurfmanFalseWindow\0"[0] as *const u8 as LPCSTR;
        let window_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(extension_loader_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: COLOR_BACKGROUND as HBRUSH,
            lpszMenuName: ptr::null_mut(),
            lpszClassName: window_class_name,
        };
        let window_class_atom = winuser::RegisterClassA(&window_class);
        assert_ne!(window_class_atom, 0);

        let mut extension_functions = WGLExtensionFunctions::default();
        let window = winuser::CreateWindowExA(
            0,
            window_class_atom as LPCSTR,
            window_class_name,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            0,
            0,
            640,
            480,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            &mut extension_functions as *mut WGLExtensionFunctions as LPVOID,
        );

        winuser::DestroyWindow(window);

        extension_functions
    }
}

#[allow(non_snake_case)]
extern "system" fn extension_loader_window_proc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    unsafe {
        match uMsg {
            WM_CREATE => {
                let pixel_format_descriptor = PIXELFORMATDESCRIPTOR {
                    nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
                    nVersion: 1,
                    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                    iPixelType: PFD_TYPE_RGBA,
                    cColorBits: 32,
                    cRedBits: 0,
                    cRedShift: 0,
                    cGreenBits: 0,
                    cGreenShift: 0,
                    cBlueBits: 0,
                    cBlueShift: 0,
                    cAlphaBits: 0,
                    cAlphaShift: 0,
                    cAccumBits: 0,
                    cAccumRedBits: 0,
                    cAccumGreenBits: 0,
                    cAccumBlueBits: 0,
                    cAccumAlphaBits: 0,
                    cDepthBits: 24,
                    cStencilBits: 8,
                    cAuxBuffers: 0,
                    iLayerType: PFD_MAIN_PLANE,
                    bReserved: 0,
                    dwLayerMask: 0,
                    dwVisibleMask: 0,
                    dwDamageMask: 0,
                };

                // Create a false GL context.
                let dc = winuser::GetDC(hwnd);
                let r = GetLastError();
                let pixel_format = wingdi::ChoosePixelFormat(dc, &pixel_format_descriptor);
                let r = GetLastError();
                assert_ne!(pixel_format, 0);
                let mut ok = wingdi::SetPixelFormat(dc, pixel_format, &pixel_format_descriptor);
                
                assert_ne!(ok, FALSE);
                let gl_context = wglCreateContext(dc);
                assert!(!gl_context.is_null());
                ok = wglMakeCurrent(dc, gl_context);
                assert_ne!(ok, FALSE);

                // Detect extensions.
                let create_struct = lParam as *mut CREATESTRUCTA;
                let wgl_extension_functions =
                    (*create_struct).lpCreateParams as *mut WGLExtensionFunctions;
                (*wgl_extension_functions).wglCreateContextAttribsARB = Some(mem::transmute(
                    wglGetProcAddress(&b"wglCreateContextAttribsARB\0"[0] as *const u8 as LPCSTR),
                ));
                (*wgl_extension_functions).wglChoosePixelFormatARB = Some(mem::transmute(
                    wglGetProcAddress(&b"wglChoosePixelFormatARB\0"[0] as *const u8 as LPCSTR),
                ));

                wglMakeCurrent(dc, std::ptr::null_mut());
                wglDeleteContext(gl_context);
                0
            }
            _ => winuser::DefWindowProcA(hwnd, uMsg, wParam, lParam),
        }
    }
}

pub(crate) fn set_exported_variables(preference: PowerPreference) {
    unsafe {
        let current_module = libloaderapi::GetModuleHandleA(ptr::null());
        assert!(!current_module.is_null());
        let nvidia_gpu_select_variable: *mut i32 = libloaderapi::GetProcAddress(
            current_module,
            NVIDIA_GPU_SELECT_SYMBOL.as_ptr() as LPCSTR,
        ) as *mut i32;
        let amd_gpu_select_variable: *mut i32 =
            libloaderapi::GetProcAddress(current_module, AMD_GPU_SELECT_SYMBOL.as_ptr() as LPCSTR)
                as *mut i32;
        if nvidia_gpu_select_variable.is_null() || amd_gpu_select_variable.is_null() {
            println!(
                "surfman: Could not find the NVIDIA and/or AMD GPU selection symbols. \
                   Your application may end up using the wrong GPU (discrete vs. \
                   integrated). To fix this issue, ensure that you are using the MSVC \
                   version of Rust and invoke the `init_env!()` macro at the root of \
                   your crate."
            );
            warn!(
                "surfman: Could not find the NVIDIA and/or AMD GPU selection symbols. \
                   Your application may end up using the wrong GPU (discrete vs. \
                   integrated). To fix this issue, ensure that you are using the MSVC \
                   version of Rust and invoke the `init_env!()` macro at the root of \
                   your crate."
            );
            return;
        }
        let value = match preference {
            PowerPreference::HighPerformance => 1,
            PowerPreference::LowPower => 0,
        };
        *nvidia_gpu_select_variable = value;
        *amd_gpu_select_variable = value;
    }
}

pub fn get_proc_address(symbol_name: &str) -> *const c_void {
    unsafe {
        // https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
        let symbol_name: CString = CString::new(symbol_name).unwrap();
        let symbol_ptr = symbol_name.as_ptr() as *const u8 as LPCSTR;
        let addr = wglGetProcAddress(symbol_ptr) as *const c_void;
        if !addr.is_null() {
            return addr;
        }
        let opengl_library = (*OPENGL_LIBRARY) as HMODULE;
        libloaderapi::GetProcAddress(opengl_library, symbol_ptr) as *const c_void
    }
}
