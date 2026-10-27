//! Named test parameters.

use wb_core::{Aabb, PeerId, Rgba, WorldPos};

/// Colour of test strokes; no `Board` test looks at it.
pub const COLOR: Rgba = Rgba([0, 0, 0, 255]);
/// Width of test strokes; no `Board` test looks at it.
pub const WIDTH: u32 = 16;
/// Bounds of test strokes, which have no points.
pub const BBOX: Aabb = Aabb::new(WorldPos::new(0, 0), WorldPos::new(0, 0));

/// At most this many ops in one generated universe.
pub const UNIVERSE: usize = 40;
/// Generated ops are spread over this many peers.
pub const PEERS: u64 = 3;
/// Generated Lamports range over `0..LAMPORTS`, few enough to collide often.
pub const LAMPORTS: u64 = 4;
/// Erase targets range over ids `0..TARGETS`: a full universe plus ids that never exist.
pub const TARGETS: u64 = 48;
/// The local author in tests that draw or erase; generated ops use it too, so its new ids
/// must skip theirs.
pub const ME: PeerId = PeerId(1);
/// Author of the erases in a canonical op list; no generated op uses it.
pub const CANONICAL_ERASER: PeerId = PeerId(u64::MAX);
