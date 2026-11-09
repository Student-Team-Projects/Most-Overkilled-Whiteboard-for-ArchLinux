//! The window's state and the operations that keep it valid. No egui types.

use crate::page_layout::{distance_between_page_tops_in_world_units, page_top_in_world_units};
use crate::stand_in::{Aabb, StandInBoard, StandInStroke, WorldPos};
use crate::tool::ToolState;

/// Everything the window shows, and the only operations that change it.
/// Knows nothing about egui; the controller is its only writer.
#[derive(Debug, Default)]
pub struct Model {
    /// Pen settings. Public because any combination of them is valid.
    pub tools: ToolState,
    board: StandInBoard,
    stroke_in_progress: Vec<WorldPos>,
    scroll_offset_in_world_units: i64,
}

impl Model {
    /// The strokes and pages.
    pub fn board(&self) -> &StandInBoard {
        &self.board
    }

    /// Points of the stroke being drawn, not on the board yet; empty between strokes.
    pub fn stroke_in_progress(&self) -> &[WorldPos] {
        &self.stroke_in_progress
    }

    /// How far the page stack is scrolled up; from 0 to the top of the last page.
    pub fn scroll_offset_in_world_units(&self) -> i64 {
        self.scroll_offset_in_world_units
    }

    /// Whether `point` lies inside the printable part of an existing page,
    /// the only place the pen draws and the eraser works.
    pub fn is_printable(&self, point: WorldPos) -> bool {
        (0..=self.board.last_page_index())
            .any(|page_index| Aabb::printable_part_of_page(page_index).contains(point))
    }

    /// Index of the page whose top edge is nearest to the scroll offset.
    pub fn page_nearest_to_top_of_view(&self) -> u32 {
        let distance = distance_between_page_tops_in_world_units();
        let nearest = (self.scroll_offset_in_world_units + distance / 2) / distance;
        u32::try_from(nearest)
            .unwrap_or(u32::MAX)
            .min(self.board.last_page_index())
    }

    /// Scrolls by `world_units` (positive moves down the stack), stopping
    /// at the first page top and at the last page top.
    pub fn scroll_by(&mut self, world_units: i64) {
        let last_page_top = self.scroll_offset_showing_top_of(self.board.last_page_index());
        self.scroll_offset_in_world_units = self
            .scroll_offset_in_world_units
            .saturating_add(world_units)
            .clamp(0, last_page_top);
    }

    /// Scrolls so the top of page `page_index` is at the top of the view;
    /// an index past the last page means the last page.
    pub fn scroll_to_top_of(&mut self, page_index: u32) {
        self.scroll_offset_in_world_units = self.scroll_offset_showing_top_of(page_index);
    }

    /// Adds a page after the last one and scrolls to it.
    pub fn add_page_at_end(&mut self) {
        let new_last_page_index = self.board.last_page_index().saturating_add(1);
        self.board.apply_add_page_idempotently(new_last_page_index);
        self.scroll_to_top_of(new_last_page_index);
    }

    /// Adds `point` to the stroke being drawn, unless it repeats the last point.
    pub fn extend_stroke_in_progress(&mut self, point: WorldPos) {
        if self.stroke_in_progress.last() != Some(&point) {
            self.stroke_in_progress.push(point);
        }
    }

    /// Moves the stroke being drawn onto the board with the current colour and width.
    /// Does nothing when no stroke is in progress.
    pub fn finish_stroke_in_progress(&mut self) {
        let points = std::mem::take(&mut self.stroke_in_progress);
        if !points.is_empty() {
            self.board.strokes.push(StandInStroke {
                color: self.tools.color,
                width_in_world_units: self.tools.stroke_width_in_world_units(),
                points,
            });
        }
    }

    /// Removes every stroke on the board for which `is_erased` returns true.
    pub fn erase_strokes_where(&mut self, is_erased: impl Fn(&StandInStroke) -> bool) {
        self.board.strokes.retain(|stroke| !is_erased(stroke));
    }

    fn scroll_offset_showing_top_of(&self, page_index: u32) -> i64 {
        page_top_in_world_units(page_index.min(self.board.last_page_index()))
    }
}

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;
    use crate::page_layout::page_top_in_world_units;

    const FIRST_PAGE: u32 = 0;
    const SECOND_PAGE: u32 = 1;
    const LAST_POSSIBLE_PAGE_INDEX: u32 = u32::MAX;
    const ONE_WORLD_UNIT: i64 = 1;
    const PRINTABLE_POINT_ON_FIRST_PAGE: WorldPos = WorldPos { x: 1_000, y: 1_000 };
    const POINT_IN_TOP_MARGIN_OF_FIRST_PAGE: WorldPos = WorldPos { x: 1_000, y: 100 };
    const POINT_IN_GAP_BELOW_FIRST_PAGE: WorldPos = WorldPos {
        x: 1_000,
        y: 13_600,
    };
    const POINT_RIGHT_OF_THE_PAGES: WorldPos = WorldPos { x: 9_600, y: 1_000 };
    const PRINTABLE_POINT_ON_SECOND_PAGE: WorldPos = WorldPos {
        x: 1_000,
        y: 15_000,
    };

    fn model_with_two_pages_scrolled_to_the_first() -> Model {
        let mut model = Model::default();
        model.add_page_at_end();
        model.scroll_to_top_of(FIRST_PAGE);
        model
    }

    #[test]
    fn scroll_by_stops_at_the_first_page_top() {
        let mut model = model_with_two_pages_scrolled_to_the_first();
        model.scroll_by(-ONE_WORLD_UNIT);
        assert_eq!(model.scroll_offset_in_world_units(), 0);
        model.scroll_by(i64::MIN);
        assert_eq!(model.scroll_offset_in_world_units(), 0);
    }

    #[test]
    fn scroll_by_stops_at_the_last_page_top() {
        let mut model = model_with_two_pages_scrolled_to_the_first();
        model.scroll_by(page_top_in_world_units(SECOND_PAGE) + ONE_WORLD_UNIT);
        assert_eq!(
            model.scroll_offset_in_world_units(),
            page_top_in_world_units(SECOND_PAGE)
        );
        model.scroll_by(i64::MAX);
        assert_eq!(
            model.scroll_offset_in_world_units(),
            page_top_in_world_units(SECOND_PAGE)
        );
    }

    #[test]
    fn add_page_at_end_on_the_last_possible_page_adds_nothing() {
        let mut model = Model::default();
        model
            .board
            .apply_add_page_idempotently(LAST_POSSIBLE_PAGE_INDEX);
        model.add_page_at_end();
        assert_eq!(model.board().last_page_index(), LAST_POSSIBLE_PAGE_INDEX);
        assert_eq!(
            model.scroll_offset_in_world_units(),
            page_top_in_world_units(LAST_POSSIBLE_PAGE_INDEX)
        );
    }

    #[test]
    fn nearest_page_switches_exactly_halfway_between_page_tops() {
        let mut model = model_with_two_pages_scrolled_to_the_first();
        let halfway = distance_between_page_tops_in_world_units() / 2;
        model.scroll_by(halfway - ONE_WORLD_UNIT);
        assert_eq!(model.page_nearest_to_top_of_view(), FIRST_PAGE);
        model.scroll_by(ONE_WORLD_UNIT);
        assert_eq!(model.page_nearest_to_top_of_view(), SECOND_PAGE);
    }

    #[test]
    fn only_the_printable_part_of_an_existing_page_is_printable() {
        let mut model = Model::default();
        assert!(model.is_printable(PRINTABLE_POINT_ON_FIRST_PAGE));
        assert!(!model.is_printable(POINT_IN_TOP_MARGIN_OF_FIRST_PAGE));
        assert!(!model.is_printable(POINT_IN_GAP_BELOW_FIRST_PAGE));
        assert!(!model.is_printable(POINT_RIGHT_OF_THE_PAGES));
        assert!(!model.is_printable(PRINTABLE_POINT_ON_SECOND_PAGE));
        model.add_page_at_end();
        assert!(model.is_printable(PRINTABLE_POINT_ON_SECOND_PAGE));
        assert!(!model.is_printable(POINT_IN_GAP_BELOW_FIRST_PAGE));
    }
}
