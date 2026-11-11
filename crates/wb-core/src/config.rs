//! Page geometry. Every peer must use the same values, so changing one is a protocol change.

/// World units in one point, the page unit: 1/72 inch, one pixel at zoom 1.
const UNITS_PER_POINT: i64 = 16;

/// Width of an A4 page.
pub const PAGE_WIDTH: i64 = 595 * UNITS_PER_POINT;
/// Height of an A4 page.
pub const PAGE_HEIGHT: i64 = 842 * UNITS_PER_POINT;
/// Space between one page and the next.
pub const PAGE_GAP: i64 = 16 * UNITS_PER_POINT;
/// Margin on every side of a page where the pen does not draw.
pub const PRINT_MARGIN: i64 = 14 * UNITS_PER_POINT;
