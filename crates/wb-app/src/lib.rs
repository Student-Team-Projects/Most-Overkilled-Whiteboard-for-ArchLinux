//! The parts of the whiteboard app that do not need a window.

pub mod camera;
pub mod config;
mod stroke_cache;

pub use stroke_cache::{DecodeAll, StrokeCache};
