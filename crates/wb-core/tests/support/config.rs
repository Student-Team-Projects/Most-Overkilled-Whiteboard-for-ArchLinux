//! Named parameters of the tests.

/// Distinct stroke ids in a generated session: few enough that inserts often replace and removes
/// often hit.
pub const ID_POOL: u64 = 64;

/// Most steps in one generated session.
pub const MAX_STEPS: usize = 256;

/// Generated coordinates fall mostly in `-GRID..=GRID`, so boxes often overlap and touch.
pub const GRID: i64 = 8;

/// A coordinate far from the origin, 2⁶², for the checks that the world has no fixed bounds.
pub const FAR: i64 = 1 << 62;
