//! The page stack: where pages lie and which point is on which page.

use wb_app::pages::PageStack;
use wb_core::WorldPos;

#[test]
fn a_new_stack_has_one_page_and_each_added_page_comes_last() {
    let mut pages = PageStack::default();
    assert_eq!((pages.count(), pages.last()), (1, 0));
    assert_eq!(pages.add_page(), 1);
    assert_eq!(pages.add_page(), 2);
    assert_eq!(pages.count(), 3);
}

#[test]
fn each_page_lies_a_gap_below_the_one_before_with_the_same_size() {
    let (first, second) = (PageStack::page(0), PageStack::page(1));
    assert_eq!(first.min, WorldPos::new(0, 0));
    assert!(second.min.y > first.max.y);
    assert_eq!(second.max.x - second.min.x, first.max.x - first.min.x);
    assert_eq!(second.max.y - second.min.y, first.max.y - first.min.y);
}

#[test]
fn a_point_lies_only_on_an_existing_page() {
    let mut pages = PageStack::default();
    let on_second = PageStack::centre(1);
    let in_gap = WorldPos::new(on_second.x, PageStack::page(0).max.y + 1);
    assert_eq!(pages.page_at(PageStack::centre(0)), Some(0));
    assert_eq!(pages.page_at(on_second), None);
    pages.add_page();
    assert_eq!(pages.page_at(on_second), Some(1));
    assert_eq!(pages.page_at(in_gap), None);
    assert_eq!(pages.page_at(WorldPos::new(on_second.x, -1)), None);
}

#[test]
fn the_margin_is_on_the_page_but_not_printable() {
    let pages = PageStack::default();
    let page = PageStack::page(0);
    let in_margin = WorldPos::new(page.min.x + 1, page.min.y + 1);
    assert_eq!(pages.page_at(in_margin), Some(0));
    assert_eq!(pages.printable_area_at(in_margin), None);
    assert_eq!(
        pages.printable_area_at(PageStack::centre(0)),
        Some(PageStack::printable_area(0))
    );
}
