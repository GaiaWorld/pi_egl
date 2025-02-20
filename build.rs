use std::env;
use std::fs::File;
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

/// 主程序入口函数
///
/// 本函数用于根据目标操作系统生成相应的OpenGL ES绑定和相关设置。
/// 当目标操作系统是Android或Linux时，将执行生成绑定的操作。
fn main() {
    // 获取目标操作系统信息
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    // 获取输出目录路径
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    // 如果目标操作系统是Android或Linux，执行绑定生成
    if target_os == "android" || target_os == "linux" {
        #[cfg(feature = "swappy")]
        {
            // 根据共享库或静态库特性添加相应的库
            if cfg!(feature = "shared-stdcxx"){
                _add_lib("c++_shared", false);
            } else {
                _add_lib("c++_static", false);
            }
            // 将swappy库加入链接搜索路径和链接库
            println!("cargo:rustc-link-search=native={}", "libs/");
            println!("cargo:rustc-link-lib=static={}", "swappy");
        }

        // 创建EGL绑定文件
        let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
        // 生成OpenGL ES 1.5 Core API的绑定
        let registry = Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, []);
        registry.write_bindings(StructGenerator, &mut file).unwrap();
    }
}

/// 添加链接库
///
/// 根据传入的库名和是否静态，向编译器发出链接指令。
/// 仅在非测试特性下执行。
fn _add_lib(_name: impl AsRef<str>, _static: bool) {
    #[cfg(not(feature = "test"))]
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        _name.as_ref()
    );
}