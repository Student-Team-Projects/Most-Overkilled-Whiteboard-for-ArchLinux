//! Every runtime tunable of the app.

/// World units in one screen pixel at zoom 1 (ADR 0014).
pub const WORLD_UNITS_PER_PIXEL: u16 = 16;

/// Smallest zoom the camera allows (ADR 0013).
pub const MIN_ZOOM: f32 = 0.5;
/// Largest zoom the camera allows (ADR 0013).
pub const MAX_ZOOM: f32 = 2.0;
