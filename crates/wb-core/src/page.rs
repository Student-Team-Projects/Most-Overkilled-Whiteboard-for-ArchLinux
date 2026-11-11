//! The page stack: A4 pages one under another, the first with its top-left corner at the origin.

use crate::config::{PAGE_GAP, PAGE_HEIGHT, PAGE_WIDTH, PRINT_MARGIN};
use crate::{Aabb, Intersects, WorldPos};

/// Distance from the top of one page to the top of the next.
const PITCH: i64 = PAGE_HEIGHT + PAGE_GAP;

/// A page of the stack, numbered from 0 at the top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page(
    /// Position in the stack.
    pub u32,
);

impl Page {
    /// The page's rectangle.
    #[must_use]
    pub fn bounds(self) -> Aabb {
        let top = i64::from(self.0) * PITCH;
        Aabb::new(
            WorldPos::new(0, top),
            WorldPos::new(PAGE_WIDTH, top + PAGE_HEIGHT),
        )
    }

    /// The page without its print margin: the only part the pen draws on.
    #[must_use]
    pub fn printable_area(self) -> Aabb {
        let page = self.bounds();
        Aabb::new(
            WorldPos::new(page.min.x + PRINT_MARGIN, page.min.y + PRINT_MARGIN),
            WorldPos::new(page.max.x - PRINT_MARGIN, page.max.y - PRINT_MARGIN),
        )
    }

    /// The page whose printable area holds `point`, whether or not the board has that many pages;
    /// `None` in a margin, a gap or beside the stack.
    #[must_use]
    pub fn printable_at(point: WorldPos) -> Option<Self> {
        let page = Self(u32::try_from(point.y.div_euclid(PITCH)).ok()?);
        page.printable_area()
            .intersects(&Aabb::new(point, point))
            .then_some(page)
    }
}

/// How many pages a stroke whose lowest point is at `y` needs: one past the page whose span, with
/// the gap below it, holds `y`. At least 1, saturating at `u32::MAX`.
pub fn pages_to_hold(y: i64) -> u32 {
    let page = y.div_euclid(PITCH).max(0);
    u32::try_from(page).map_or(u32::MAX, |page| page.saturating_add(1))
}
