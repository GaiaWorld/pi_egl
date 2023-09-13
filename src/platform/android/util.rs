use crate::platform::android::egl;
use crate::platform::android::egl::types::{EGLConfig, EGLDisplay, EGLint};
use crate::platform::android::egl::Egl;
use libc::{dlopen, dlsym, RTLD_LAZY};
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
};
pub struct EGLLibraryWrapper(*mut c_void);

unsafe impl Send for EGLLibraryWrapper {}
unsafe impl Sync for EGLLibraryWrapper {}

pub struct EGLFuncWrapper(pub Egl);

unsafe impl Send for EGLFuncWrapper {}
unsafe impl Sync for EGLFuncWrapper {}

lazy_static! {
    pub static ref EGL_LIBRARY: EGLLibraryWrapper = {
        unsafe {
            EGLLibraryWrapper(dlopen(
                &b"libEGL.so\0"[0] as *const u8 as *const _,
                RTLD_LAZY,
            ))
        }
    };
    pub static ref EGL_FUNCTIONS: EGLFuncWrapper = EGLFuncWrapper(Egl::load_with(get_egl_address));
}

fn get_egl_address(symbol_name: &str) -> *const c_void {
    unsafe {
        let symbol_name: CString = CString::new(symbol_name).unwrap();
        let symbol_ptr = symbol_name.as_ptr() as *const u8 as *const c_char;
        let v = dlsym(EGL_LIBRARY.0, symbol_ptr) as *const c_void;
        // println!("egl symbol_name {:?} ptr is {:?}!!", symbol_name, v);
        v
    }
}

// pub(crate) unsafe fn lookup_egl_extension(name: &'static [u8]) -> *mut c_void {
//     EGL_FUNCTIONS
//         .with(|egl| mem::transmute(egl.GetProcAddress(&name[0] as *const u8 as *const c_char)))
// }

pub(crate) unsafe fn egl_config_from_display(egl_display: EGLDisplay) -> EGLConfig {
    let config_attributes = [
        egl::RENDERABLE_TYPE as EGLint,
        egl::OPENGL_ES2_BIT as EGLint, // Request opengl ES2.0
        egl::SURFACE_TYPE as EGLint,
        egl::WINDOW_BIT as EGLint,
        egl::BLUE_SIZE as EGLint,
        8,
        egl::GREEN_SIZE as EGLint,
        8,
        egl::RED_SIZE as EGLint,
        8,
        egl::DEPTH_SIZE as EGLint,
        24,
        egl::NONE as EGLint,
        0,
        0,
        0,
    ]
    .to_vec();

    let egl = &EGL_FUNCTIONS.0;

    let (mut config, mut config_count) = (std::ptr::null(), 0);
    let result = egl.ChooseConfig(
        egl_display,
        config_attributes.as_ptr(),
        &mut config,
        1,
        &mut config_count,
    );
    assert_ne!(result, egl::FALSE);
    assert!(config_count > 0);
    config
}
