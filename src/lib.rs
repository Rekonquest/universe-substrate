//! A deliberately non-Vulkan substrate.
//!
//! The crate models a bounded material field. Energy enters at its boundary,
//! propagates by local differences, alters the material it traverses, and can
//! become visible where local spectral coupling permits it. There are no API
//! commands, handles, queues, allocations, or device objects in the model.

mod field;
mod render;

pub use field::{Config, Measurements, Spectrum, World};
pub use render::write_bmp;
