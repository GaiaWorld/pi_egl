use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    // println!("cargo:rerun-if-changed=build.rs");

    // // TODO 填上 DXT-压缩纹理，ASTC-压缩纹理
    // let extensions = [];

    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     let mut file = File::create(dest.join("gles3_bindings.rs")).unwrap();

    //     gl_generator::Registry::new(
    //         gl_generator::Api::Gles2,
    //         (3, 0),
    //         gl_generator::Profile::Core,
    //         gl_generator::Fallbacks::None,
    //         extensions,
    //     )
    //     .write_bindings(gl_generator::StaticStructGenerator, &mut file)
    //     .unwrap();
    // }
    
    // // TODO WebGL2.0 接口；
    // #[cfg(target_arch = "wasm32")]
    // {
    //     let mut file = File::create(dest.join("webgl2_bindings.rs")).unwrap();
        
    //     webgl_generator::Registry::new(
    //         webgl_generator::Api::Gles2,
    //         (3, 0),
    //         webgl_generator::Profile::Core,
    //         webgl_generator::Fallbacks::None,
    //         extensions,
    //     )
    //     .write_bindings(webgl_generator::StaticStructGenerator, &mut file)
    //     .unwrap();
    // }
}
