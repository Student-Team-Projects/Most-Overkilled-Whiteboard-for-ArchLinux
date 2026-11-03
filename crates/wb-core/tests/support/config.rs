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

/// The real piece budget in bytes: about one datagram.
pub const BUDGET: usize = 1200;

/// The largest two-point `Raw` piece in bytes, from which `Raw` never returns `BudgetTooSmall`:
/// a 10-byte varint per origin axis, a 1-byte length, the first point's `[0, 0]` offset (2 bytes)
/// and a 5-byte varint per axis of the second offset.
pub const RAW_MIN_BUDGET: usize = 10 + 10 + 1 + 2 + 5 + 5;

/// Points in a long test stroke: past 127, so postcard needs a 2-byte length prefix.
pub const LONG_STROKE: u16 = 300;

/// Generated coordinates and steps near zero lie in `-NEAR_ZERO..NEAR_ZERO`, so each fits a
/// 1-byte varint.
pub const NEAR_ZERO: i32 = 64;

/// Generated wide offsets lie within `±WIDE_OFFSET`: most need a 5-byte varint, yet the step
/// between two of them still fits `i32`.
pub const WIDE_OFFSET: i32 = 1 << 30;

/// Exclusive upper bound on the length of random bytes fed to `decode`.
pub const FUZZ_LEN: usize = 256;
