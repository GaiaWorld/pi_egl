use std::env;
use std::fs::File;
use std::path::PathBuf;

use gl_generator::{Registry, Profile, Fallbacks, Api, StructGenerator};


fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    if target_os == "android" {
        println!("cargo:rustc-link-search=native={}", "libs/");
        println!("cargo:rustc-link-lib=static={}", "swappy");
        let mut file = File::create(&dest.join("egl_bindings.rs")).unwrap();
        let registry = Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, []);
        registry.write_bindings(StructGenerator, &mut file).unwrap();
    }
}
