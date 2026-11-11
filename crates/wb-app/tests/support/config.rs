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
