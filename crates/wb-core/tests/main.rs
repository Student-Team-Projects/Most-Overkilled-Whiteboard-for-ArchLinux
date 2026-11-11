//! Every `wb-core` test, in one binary; this file only lists the modules.

mod aabb;
mod board;
mod codec;
mod conformance;
mod intersects;
mod page;
#[cfg(feature = "ecolor")]
mod rgba;
mod stroke_points;
mod support;
