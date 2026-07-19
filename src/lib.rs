//! A deliberately non-Vulkan substrate.
//!
//! The crate models a bounded material field. Energy enters at its boundary,
//! propagates by local differences, alters the material it traverses, and can
//! become visible where local spectral coupling permits it. There are no API
//! commands, handles, queues, allocations, or device objects in the model.

mod falsify;
mod field;
mod render;
mod sweep;

pub use falsify::{
    FalsificationReport, FalsificationThresholds, PrimitiveStack, StackOutcome,
    run_standard_falsification,
};
pub use field::{Config, CouplingMode, DisturbanceMode, Measurements, Spectrum, World};
pub use render::write_bmp;
pub use sweep::{CandidateOutcome, PrimitiveCandidate, SweepReport, run_standard_sweep};
