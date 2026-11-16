//! In-process Sense sources for Linux. See `sense/spec/platform-linux.md`.
//!
//! Implemented: `linux.atspi` (focused element and exact selection through the accessibility bus).

#[cfg(target_os = "linux")]
pub mod atspi;
