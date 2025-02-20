use std::{
    ffi::CString,
    marker::PhantomData,
    mem,
    os::raw::{c_int, c_void},
    ptr,
    sync::mpsc::{self, Sender},
    thread,
};

use std::thread::JoinHandle;
use winapi::{
    shared::{
        minwindef::{
            self, BOOL, FALSE, FLOAT, HMODULE, LPARAM, LPVOID, LRESULT, UINT, WORD, WPARAM,
        },
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
            self, COLOR_BACKGROUND, CREATESTRUCTA, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, MSG, WM_CLOSE,
            WM_CREATE, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

use crate::PowerPreference;
/// HIDDEN_WINDOW_SIZE是用于创建隐藏窗口时的尺寸参数。
/// 值为16，即窗口的尺寸将被设置为16x16像素。
pub(crate) const HIDDEN_WINDOW_SIZE: c_int = 16;

/// NVIDIA_GPU_SELECT_SYMBOL是用于启用NVIDIA显卡的符号常量。
/// 用于调用NVIDIA的特殊函数来启用高性能显卡。
static NVIDIA_GPU_SELECT_SYMBOL: &[u8] = b"NvOptimusEnablement\0";
/// AMD_GPU_SELECT_SYMBOL是用于启用AMD显卡的符号常量。
/// 用于调用AMD的特殊函数来启用高性能显卡。
static AMD_GPU_SELECT_SYMBOL: &[u8] = b"AmdPowerXpressRequestHighPerformance\0";

lazy_static! {
    /// WGL_EXTENSION_FUNCTIONS是与WGL扩展函数相关的结构体的实例。
    /// 这个实例包含了一些与OpenGL扩展相关的函数指针。
    pub(crate) static ref WGL_EXTENSION_FUNCTIONS: WGLExtensionFunctions =
        thread::spawn(extension_loader_thread).join().unwrap();

    /// OPENGL_LIBRARY存储了opengl32.dll的模块句柄。
    /// 该句柄用于获取OpenGL相关的函数。
    pub(crate) static ref OPENGL_LIBRARY: u64 =
        { unsafe { libloaderapi::LoadLibraryA(&b"opengl32.dll\0"[0] as *const u8 as LPCSTR) } }
            as u64;
}

/// WGLExtensionFunctions结构体包含了WGL扩展函数的函数指针。
/// 这些函数指针在与Windows OpenGL扩展相关的操作中使用。
#[allow(non_snake_case)]
#[derive(Default)]
pub(crate) struct WGLExtensionFunctions {
    /// wglCreateContextAttribsARB函数用于创建带有属性列表的OpenGL上下文。
    pub wglCreateContextAttribsARB: Option<unsafe extern "C" fn(
        hDC: HDC,
        shareContext: HGLRC,
        attribList: *const c_int,
    ) -> HGLRC>,

    /// wglChoosePixelFormatARB函数用于为设备上下文选择合适的像素格式。
    pub wglChoosePixelFormatARB: Option<unsafe extern "C" fn(
            hdc: HDC,
            piAttribIList: *const c_int,
            pfAttribFList: *const FLOAT,
            nMaxFormats: UINT,
            piFormats: *mut c_int,
            nNumFormats: *mut UINT,
    ) -> BOOL>,

    /// wglSwapIntervalEXT函数用于控制交换区间，以实现垂直同步。
    pub wglSwapIntervalEXT: Option<unsafe extern "C" fn(interval: c_int) -> BOOL>,
}

/// extension_loader_thread函数负责加载WGL扩展函数。
/// 通过创建一个临时窗口并初始化OpenGL上下文来获取这些扩展函数的指针。
fn extension_loader_thread() -> WGLExtensionFunctions {
    unsafe {
        // 获取当前模块的句柄。
        let instance = libloaderapi::GetModuleHandleA(ptr::null_mut());

        // 定义一个特定的窗口类名，用于创建一个隐藏窗口。
        let window_class_name = &b"SurfmanFalseWindow\0"[0] as *const u8 as LPCSTR;

        // 定义窗口类的属性。
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

        // 注册窗口类。
        let window_class_atom = winuser::RegisterClassA(&window_class);
        assert_ne!(window_class_atom, 0, "注册窗口类失败");

        // 初始化WGL_extension_functions结构体。
        let mut extension_functions = WGLExtensionFunctions::default();

        // 创建一个窗口实例，用于获取WGL扩展函数。
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

        // 销毁创建的窗口。
        winuser::DestroyWindow(window);

        extension_functions
    }
}

/// extension_loader_window_proc是窗口处理过程函数，用于处理窗口消息。
/// 特别是在WM_CREATE消息时，初始化OpenGL上下文并获取扩展函数指针。
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
                // 设定像素格式描述符，以配置 OpenGL 渲染上下文。
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

                // 获取设备上下文。
                let dc = winuser::GetDC(hwnd);

                // 根据描述符选择像素格式。
                let pixel_format = wingdi::ChoosePixelFormat(dc, &pixel_format_descriptor);
                assert_ne!(pixel_format, 0, "选择像素格式失败");

                // 设置像素格式到设备上下文。
                let ok = wingdi::SetPixelFormat(dc, pixel_format, &pixel_format_descriptor);
                assert_ne!(ok, FALSE, "设置像素格式失败");

                // 创建 OpenGL 渲染上下文。
                let gl_context = wglCreateContext(dc);
                assert!(!gl_context.is_null(), "创建 OpenGL 上下文失败");

                // 将上下文设置为当前上下文。
                let ok = wglMakeCurrent(dc, gl_context);
                assert_ne!(ok, FALSE, "无法设置 OpenGL 上下文为当前");

                // 获取 WGL 扩展函数指针。
                let create_struct = lParam as *mut CREATESTRUCTA;
                let wgl_extension_functions = (*create_struct).lpCreateParams as *mut WGLExtensionFunctions;

                (*wgl_extension_functions).wglCreateContextAttribsARB = Some(mem::transmute(
                    wglGetProcAddress(&b"wglCreateContextAttribsARB\0"[0] as *const u8 as LPCSTR),
                ));

                (*wgl_extension_functions).wglChoosePixelFormatARB = Some(mem::transmute(
                    wglGetProcAddress(&b"wglChoosePixelFormatARB\0"[0] as *const u8 as LPCSTR),
                ));

                let func = wglGetProcAddress(&b"wglSwapIntervalEXT\0"[0] as *const u8 as LPCSTR);
                if !func.is_null() {
                    (*wgl_extension_functions).wglSwapIntervalEXT = Some(mem::transmute(func));
                }

                // 清理上下文。
                wglMakeCurrent(dc, std::ptr::null_mut());
                wglDeleteContext(gl_context);
                0
            }
            _ => {
                // 对于其他消息，调用默认窗口处理过程。
                winuser::DefWindowProcA(hwnd, uMsg, wParam, lParam)
            }
        }
    }
}
/// 设置指定的环境变量以选择 GPU。
/// 这些环境变量主要用于指定使用集成显卡还是独立显卡。
/// 1. `NVIDIA_GPU_SELECT_SYMBOL`：针对 NVIDIA 显卡的选择。
/// 2. `AMD_GPU_SELECT_SYMBOL`：针对 AMD 显卡的选择。
/// 这些变量可以在程序启动前设置，以优化OpenGL性能。
/// 目前支持的选项有两种：
/// - `PowerPreference::HighPerformance`：优先使用独立显卡，提供更高的性能。
/// - `PowerPreference::LowPower`：优先使用集成显卡，提供更低的功耗。
pub(crate) fn set_exported_variables(preference: PowerPreference) {
    unsafe {
        let current_module = libloaderapi::GetModuleHandleA(ptr::null());
        assert!(!current_module.is_null());

        // 获取 NVIDIA 和 AMD 的 GPU 选择变量地址。
        let nvidia_gpu_select_variable: *mut i32 = libloaderapi::GetProcAddress(
            current_module,
            NVIDIA_GPU_SELECT_SYMBOL.as_ptr() as LPCSTR,
        ) as *mut i32;
        let amd_gpu_select_variable: *mut i32 =
            libloaderapi::GetProcAddress(current_module, AMD_GPU_SELECT_SYMBOL.as_ptr() as LPCSTR)
                as *mut i32;

        // 检查是否获取到了两个变量的指针。
        if nvidia_gpu_select_variable.is_null() || amd_gpu_select_variable.is_null() {
            log::warn!(
                "pi_egl: 无法找到 NVIDIA 和/或 AMD 的 GPU 选择符号。 \
                 您的应用程序可能会使用错误的 GPU（独立显卡 vs. 集成显卡）。 \
                 要解决此问题，请确保使用 MSVC 版本的 Rust 并在 crate 根目录调用 `init_env!()` 宏。"
            );
            return;
        }

        // 将值设置到指定的变量中。
        let value = match preference {
            PowerPreference::HighPerformance => 1,
            PowerPreference::LowPower => 0,
        };
        *nvidia_gpu_select_variable = value;
        *amd_gpu_select_variable = value;
    }
}

/// 获取 OpenGL 过程地址。
/// 在 Windows 中，某些 OpenGL 函数需要通过 `wglGetProcAddress` 来获取。
/// 如果函数不存在，会尝试从显卡驱动中获取地址。
pub fn get_proc_address(symbol_name: &str) -> *const c_void {
    unsafe {
        // 将符号名称转换为 C 字符串。
        let symbol_name: CString = CString::new(symbol_name).unwrap();
        let symbol_ptr = symbol_name.as_ptr() as *const u8 as LPCSTR;
        let addr = wglGetProcAddress(symbol_ptr) as *const c_void;

        // 如果通过 `wglGetProcAddress` 获取到地址，直接返回。
        if !addr.is_null() {
            return addr;
        }

        // 如果未找到，尝试从显卡驱动中获取。
        // 这里从 `OPENGL_LIBRARY` 变量中获取 OpenGL 库的句柄。
        // 必须提前初始化该句柄。
        let opengl_library = (*OPENGL_LIBRARY) as HMODULE;
        libloaderapi::GetProcAddress(opengl_library, symbol_ptr) as *const c_void
    }
}

// 保护用于隐藏窗口的句柄，防止窗口被提前销毁。
/// HiddenWindow 结构体用于管理一个隐藏窗口。
/// 它包含窗口句柄和一个线程句柄，用于确保窗口的安全关闭。
pub(crate) struct HiddenWindow {
    window: HWND,
    join_handle: Option<JoinHandle<()>>,
}

// DCGuard 结构体用于确保在 scope 之后释放设备上下文。
/// DCGuard 是一种 RAII wrapper，用于管理设备上下文 (DC) 的释放。
/// 它会在 drop 时确保正确释放设备上下文，避免资源泄漏。
pub(crate) struct DCGuard<'a> {
    pub(crate) dc: HDC,
    window: Option<HWND>,
    phantom: PhantomData<&'a HWND>,
}

// SendableHWND 包裝 HWND，保证其 Send 特性安全。
/// SendableHWND 是一个将 HWND 包装的结构，用于允许 HWND 在多线程环境中安全传递。
/// 尽管 HWND 本身是不可 Send 的，但一些特定用例下可能需要在线程间传递它。
/// 这里通过 unsafe 实现 Send 特性来满足需求。
struct SendableHWND(HWND);

unsafe impl Send for SendableHWND {}

// HiddenWindow 实现 Drop 特性，以确保窗口在销毁时正确清理资源。
/// 当 HiddenWindow 被 drop 时，会异步发送 WM_CLOSE 消息关闭窗口。
/// 如果使用线程句柄，它会等待线程结束，确保资源被正确释放。
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            winuser::PostMessageA(self.window, WM_CLOSE, 0, 0);
            if let Some(join_handle) = self.join_handle.take() {
                drop(join_handle.join());
            }
        }
    }
}

impl<'a> Drop for DCGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if let Some(window) = self.window {
                winuser::ReleaseDC(window, self.dc);
            }
        }
    }
}

/// HiddenWindow 实现的方法，提供创建、读取和管理隐藏窗口的功能。
impl HiddenWindow {
    /// 创建一个新的 HiddenWindow 实例。
    /// 这个方法会在后台开启一个线程来创建一个隐藏窗口，并运行消息循环。
    /// 返回一个新的 HiddenWindow 实例，包含窗口句柄和线程句柄。
    pub(crate) fn _new() -> HiddenWindow {
        let (sender, receiver) = mpsc::channel();
        let join_handle = thread::spawn(|| HiddenWindow::_thread(sender));
        let window = receiver.recv().unwrap().0;
        HiddenWindow {
            window,
            join_handle: Some(join_handle),
        }
    }

    /// 获取与窗口相关联的设备上下文 (DC)。
    /// 这个方法会返回一个 DCGuard 实例，确保在 scope 结束时释放设备上下文。
    #[inline]
    pub(crate) fn _get_dc(&self) -> DCGuard {
        unsafe { DCGuard::_new(winuser::GetDC(self.window), Some(self.window)) }
    }

    /// 这是 HiddenWindow 在后台运行的线程函数。
    /// 它会创建一个新的窗口，运行消息循环，直到收到 WM_CLOSE 消息为止。
    fn _thread(sender: Sender<SendableHWND>) {
        unsafe {
            let instance = libloaderapi::GetModuleHandleA(ptr::null_mut());
            let window_class_name = &b"SurfmanHiddenWindow\0"[0] as *const u8 as LPCSTR;
            let mut window_class = mem::zeroed();
            if winuser::GetClassInfoA(instance, window_class_name, &mut window_class) == FALSE {
                window_class = WNDCLASSA {
                    style: CS_OWNDC,
                    lpfnWndProc: Some(winuser::DefWindowProcA),
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
            }

            let window = winuser::CreateWindowExA(
                0,
                window_class_name,
                window_class_name,
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                HIDDEN_WINDOW_SIZE,
                HIDDEN_WINDOW_SIZE,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );

            sender.send(SendableHWND(window)).unwrap();

            let mut msg: MSG = mem::zeroed();
            while winuser::GetMessageA(&mut msg, window, 0, 0) != FALSE {
                println!("msg: {}", minwindef::LOWORD(msg.message));
                winuser::TranslateMessage(&msg);
                winuser::DispatchMessageA(&msg);
                if minwindef::LOWORD(msg.message) as UINT == WM_CLOSE {
                    break;
                }
            }
        }
    }

    /// 创建一个新的窗口并返回其句柄。
    /// 这个方法直接创建窗口而不启动消息循环，适用于需要自定义窗口管理的场景。
    pub fn create() -> HWND {
        unsafe {
            let instance = libloaderapi::GetModuleHandleA(ptr::null_mut());
            let window_class_name = &b"SurfmanHiddenWindow\0"[0] as *const u8 as LPCSTR;
            let mut window_class = mem::zeroed();
            if winuser::GetClassInfoA(instance, window_class_name, &mut window_class) == FALSE {
                window_class = WNDCLASSA {
                    style: CS_OWNDC,
                    lpfnWndProc: Some(winuser::DefWindowProcA),
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
            }

            let window = winuser::CreateWindowExA(
                0,
                window_class_name,
                window_class_name,
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                HIDDEN_WINDOW_SIZE,
                HIDDEN_WINDOW_SIZE,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut(),
            );

            window
        }
    }
}

/// DCGuard 实现的方法，确保设备上下文在作用域结束时正确释放。
impl<'a> DCGuard<'a> {
    /// 创建一个新的 DCGuard 实例，用于管理设备上下文。
    /// 需要提供设备上下文句柄 (DC) 和关联的窗口句柄 (HWND)。
    pub(crate) fn _new(dc: HDC, window: Option<HWND>) -> DCGuard<'a> {
        DCGuard {
            dc,
            window,
            phantom: PhantomData,
        }
    }
}

/// 设置设备上下文的像素格式.
/// 需要提供设备上下文句柄 (DC) 和所需的像素格式。
/// 此函数会先查询当前设备上下文是否支持指定的像素格式，如果支持则设置之。
pub(crate) fn set_dc_pixel_format(dc: HDC, pixel_format: c_int) {
    unsafe {
        let mut pixel_format_descriptor = mem::zeroed();
        let pixel_format_count = wingdi::DescribePixelFormat(
            dc,
            pixel_format,
            mem::size_of::<PIXELFORMATDESCRIPTOR>() as UINT,
            &mut pixel_format_descriptor,
        );
        assert_ne!(pixel_format_count, 0, "无法描述像素格式"); // 确认描述成功
        let ok = wingdi::SetPixelFormat(dc, pixel_format, &mut pixel_format_descriptor);
        assert_ne!(ok, FALSE, "设置像素格式失败"); // 确保设置成功
    }
}
