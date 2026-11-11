//! Named test parameters.

use wb_core::{PeerId, Rgba};

/// Author of every test stroke.
pub const AUTHOR: PeerId = PeerId(0);
/// Colour of test strokes; no cache looks at it.
pub const COLOR: Rgba = Rgba([0, 0, 0, 255]);
/// Width of test strokes; no cache looks at it.
pub const WIDTH: u32 = 16;

/// The real piece budget in bytes: about one datagram.
pub const BUDGET: usize = 1200;

/// At most this many strokes in a generated board, so gets often hit the same stroke.
pub const POOL: usize = 16;
/// At most this many points in a generated stroke, few enough for one piece.
pub const MAX_POINTS: usize = 32;
/// Generated coordinates lie in `-SPAN..=SPAN`.
pub const SPAN: i64 = 1000;
/// Exclusive upper bound on the length of a generated malformed piece.
pub const JUNK_LEN: usize = 32;
/// At most this many steps in one generated session.
pub const MAX_STEPS: usize = 128;

/// Zoom of every test camera that does not test zoom.
pub const OPENING_ZOOM: f32 = 1.0;
/// One pixel at zoom 1, in world units.
pub const ONE_WORLD_PIXEL: u32 = 16;
/// Size of the off-screen canvas, in screen pixels.
pub const VIEWPORT: egui::Vec2 = egui::Vec2::new(800.0, 600.0);
/// A point far outside a `VIEWPORT` centred on the world origin.
pub const FAR_OFF_SCREEN: wb_core::WorldPos = wb_core::WorldPos::new(1_000_000, 0);
/// Two stroke colours told apart in paint order.
pub const RED: Rgba = Rgba([220, 40, 40, 255]);
/// See `RED`.
pub const BLUE: Rgba = Rgba([30, 90, 220, 255]);

/// Eraser reach in world units, divisible by 5 for a 3-4-5 triangle.
pub const REACH: u32 = 100;
/// `REACH` as a coordinate offset.
pub const REACH_OFFSET: i64 = REACH as i64;

/// Points in a stroke too long for one piece of `PIECE_BUDGET_BYTES`.
pub const LONG_STROKE_POINTS: i64 = 2_000;
/// Distance between neighbouring points of the long stroke: a multi-byte varint offset.
pub const LONG_STROKE_STEP: i64 = 1_000;

/// Generated camera centres lie in this range on both axes.
pub const WORLD_CENTRE: std::ops::Range<i64> = -(1 << 40)..(1 << 40);
/// Generated points lie this far from the camera centre, in world units.
pub const OFFSET_FROM_CENTRE: std::ops::Range<i64> = -1_000_000..1_000_000;
/// Generated screen positions lie in this range on both axes.
pub const SCREEN_COORDINATE: std::ops::Range<f32> = 0.0..2_000.0;
/// Generated viewport sides, in screen pixels.
pub const VIEWPORT_SIDE: std::ops::Range<f32> = 1.0..4_000.0;
/// Generated zoom factors, far past both ends of the zoom range.
pub const ZOOM_FACTOR: std::ops::Range<f32> = 0.001..1_000.0;
/// Exclusive upper bound on the zoom steps in one generated case.
pub const MAX_ZOOM_STEPS: usize = 8;
/// How far the world point under the anchor may drift while zooming, in screen pixels.
pub const ANCHOR_TOLERANCE_PX: f32 = 0.1;
