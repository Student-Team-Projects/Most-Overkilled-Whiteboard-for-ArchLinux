//! Page geometry: where each page of the stack lies and where the pen may draw.

use wb_core::{Page, WorldPos};

use crate::support::config::{FIRST_PAGE, FIRST_PRINTABLE_AREA, PAGE_PITCH};
use crate::support::page::AabbExt;

#[test]
fn pages_are_a4_one_under_another() {
    assert_eq!(Page(0).bounds(), FIRST_PAGE);
    assert_eq!(Page(1).bounds(), FIRST_PAGE.moved_down(PAGE_PITCH));
    let last = i64::from(u32::MAX) * PAGE_PITCH;
    assert_eq!(Page(u32::MAX).bounds(), FIRST_PAGE.moved_down(last));
}

#[test]
fn the_printable_area_leaves_a_margin_on_every_side() {
    assert_eq!(Page(0).printable_area(), FIRST_PRINTABLE_AREA);
    assert_eq!(
        Page(2).printable_area(),
        FIRST_PRINTABLE_AREA.moved_down(2 * PAGE_PITCH)
    );
}

#[test]
fn printable_at_finds_the_page_under_the_pen() {
    let area = FIRST_PRINTABLE_AREA.moved_down(7 * PAGE_PITCH);
    for corner in [area.min, area.max, WorldPos::new(area.min.x, area.max.y)] {
        assert_eq!(Page::printable_at(corner), Some(Page(7)), "{corner:?}");
    }
    let last = FIRST_PRINTABLE_AREA.moved_down(i64::from(u32::MAX) * PAGE_PITCH);
    assert_eq!(Page::printable_at(last.min), Some(Page(u32::MAX)));
}

#[test]
fn printable_at_is_none_off_the_printable_areas() {
    let area = FIRST_PRINTABLE_AREA;
    let page_below = FIRST_PAGE.max.y + 1;
    let off = [
        WorldPos::new(area.min.x - 1, area.min.y),
        WorldPos::new(area.max.x + 1, area.max.y),
        WorldPos::new(area.min.x, area.min.y - 1),
        WorldPos::new(area.min.x, area.max.y + 1),
        WorldPos::new(area.min.x, page_below),
        WorldPos::new(area.min.x, PAGE_PITCH - 1),
        WorldPos::new(area.min.x, PAGE_PITCH),
        WorldPos::new(-1, area.min.y),
        WorldPos::new(FIRST_PAGE.max.x + 1, area.min.y),
        WorldPos::new(i64::MIN, area.min.y),
        WorldPos::new(area.min.x, -1),
        WorldPos::new(i64::MIN, i64::MIN),
        WorldPos::new(i64::MAX, i64::MAX),
        WorldPos::new(
            area.min.x,
            (i64::from(u32::MAX) + 1) * PAGE_PITCH + area.min.y,
        ),
    ];
    for point in off {
        assert_eq!(Page::printable_at(point), None, "{point:?}");
    }
}
