//! The whiteboard app: camera, tools, rendering and the window's views. `main.rs` only opens
//! the window.

pub mod camera;
pub mod config;
pub mod controller;
pub mod eraser_utils;
pub mod local_board;
pub mod model;
pub mod pages;
mod stroke_cache;
pub mod stroke_renderer;
pub mod tool;
pub mod view;

pub use stroke_cache::{DecodeAll, StrokeCache};
