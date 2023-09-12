#[macro_use]
extern crate lazy_static;

mod gl;
mod instance;
mod surface;
mod context;
pub mod macros;

pub mod platform;
pub use gl::*;
pub use instance::*;
pub use surface::*;
pub use context::*;

/// Power Preference when choosing a physical adapter.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PowerPreference {
    /// Windows下: 集显
    LowPower = 0,
    /// Windows下: 独显
    HighPerformance = 1,
}