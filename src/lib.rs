//! Portable CPU primitives for a future resource-pack generator.
//!
//! This is deliberately not a replacement for the missing Python pipeline.
//! The inputs below are linear scalar data, not gamma-encoded image samples.

pub mod connected;
pub mod labpbr;
pub mod manifest;
pub mod material;
pub mod pipeline;
pub mod png_io;
pub mod recipes;
pub mod resample;
pub mod verify;
pub mod zip;
