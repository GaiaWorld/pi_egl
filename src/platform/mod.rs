#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "linux")]
pub mod android;

#[cfg(target_arch = "wasm32")]
pub mod web;