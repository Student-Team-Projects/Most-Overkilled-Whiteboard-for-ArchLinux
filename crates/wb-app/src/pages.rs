//! The stack of A4 pages the board is drawn on (ADR 0027).

use wb_core::{Aabb, Intersects, WorldPos};

use crate::config::{
    A4_HEIGHT_PX, A4_WIDTH_PX, PAGE_GAP_PX, PRINT_MARGIN_PX, WORLD_UNITS_PER_PIXEL,
};

const UNITS_PER_PX: i64 = WORLD_UNITS_PER_PIXEL as i64;
const PAGE_WIDTH: i64 = A4_WIDTH_PX * UNITS_PER_PX;
const PAGE_HEIGHT: i64 = A4_HEIGHT_PX * UNITS_PER_PX;
const PAGE_PITCH: i64 = (A4_HEIGHT_PX + PAGE_GAP_PX) * UNITS_PER_PX;
const PRINT_MARGIN: i64 = PRINT_MARGIN_PX * UNITS_PER_PX;

/// A vertical stack of A4 pages: page 0 on top with its top-left corner at the world origin, and
/// each next page a gap below the one before (ADR 0027). Holds only how many pages exist, at
/// least one; the board takes that over once pages are board state (T30 #66).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageStack {
    count: u32,
}

impl Default for PageStack {
    fn default() -> Self {
        Self { count: 1 }
    }
}

impl PageStack {
    /// How many pages exist; at least one.
    #[must_use]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// The index of the last page.
    #[must_use]
    pub fn last(&self) -> u32 {
        self.count - 1
    }

    /// Appends a page below the last one and returns its index. Stays at `u32::MAX` pages.
    pub fn add_page(&mut self) -> u32 {
        self.count = self.count.saturating_add(1);
        self.last()
    }

    /// Page `index`, whether or not it exists yet.
    #[must_use]
    pub fn page(index: u32) -> Aabb {
        let top = i64::from(index) * PAGE_PITCH;
        Aabb::new(
            WorldPos::new(0, top),
            WorldPos::new(PAGE_WIDTH, top + PAGE_HEIGHT),
        )
    }

    /// The centre of page `index`.
    #[must_use]
    pub fn centre(index: u32) -> WorldPos {
        let page = Self::page(index);
        WorldPos::new(
            page.min.x.midpoint(page.max.x),
            page.min.y.midpoint(page.max.y),
        )
    }

    /// The part of page `index` the pen draws on: the page without its print margin.
    #[must_use]
    pub fn printable_area(index: u32) -> Aabb {
        let page = Self::page(index);
        Aabb::new(
            WorldPos::new(page.min.x + PRINT_MARGIN, page.min.y + PRINT_MARGIN),
            WorldPos::new(page.max.x - PRINT_MARGIN, page.max.y - PRINT_MARGIN),
        )
    }

    /// The existing page holding `point`, edges included; a point in the gap between pages lies
    /// on none.
    #[must_use]
    pub fn page_at(&self, point: WorldPos) -> Option<u32> {
        if point.y < 0 {
            return None;
        }
        let index = u32::try_from(point.y / PAGE_PITCH)
            .ok()
            .filter(|&index| index < self.count)?;
        Self::page(index)
            .intersects(&Aabb::new(point, point))
            .then_some(index)
    }

    /// The printable area holding `point`, edges included, if it lies on one of an existing page.
    #[must_use]
    pub fn printable_area_at(&self, point: WorldPos) -> Option<Aabb> {
        self.page_at(point)
            .map(Self::printable_area)
            .filter(|area| area.intersects(&Aabb::new(point, point)))
    }
}
