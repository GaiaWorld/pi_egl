#[macro_export]
macro_rules! init_env {
    () => {
        #[cfg(target_os = "windows")]
        #[link_section = ".drectve"]
        #[no_mangle]
        pub static _SURFMAN_LINK_ARGS: [u8; 74] =
            *b" /export:NvOptimusEnablement /export:AmdPowerXpressRequestHighPerformance ";
        #[cfg(target_os = "windows")]
        #[no_mangle]
        pub static mut NvOptimusEnablement: i32 = 1;
        #[cfg(target_os = "windows")]
        #[no_mangle]
        pub static mut AmdPowerXpressRequestHighPerformance: i32 = 1;
    };
}