//! Engine of the Sense context service.
//!
//! This crate contains no OS call and no `cfg(target_os)`: everything platform-specific is a
//! [`Source`] implemented elsewhere. See `sense/README.md` for the invariants.

pub mod card;
pub mod engine;
pub mod privacy;
pub mod profiles;
pub mod time;

pub use engine::{CaptureRequest, Config, Engine, Source};
