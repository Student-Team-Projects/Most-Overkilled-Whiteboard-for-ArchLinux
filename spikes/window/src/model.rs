use crate::config::{DEFAULT_STROKE_COLOR, DEFAULT_STROKE_WIDTH_PX, WORLD_UNITS_PER_PIXEL};
use crate::page_layout::{distance_between_page_tops_in_world_units, page_top_in_world_units};
use crate::stand_in::{Aabb, Rgba, StandInBoard, StandInStroke, WorldPos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
}

#[derive(Debug)]
pub struct ToolState {
    pub tool: Tool,
    pub color: Rgba,
    pub stroke_width_px: u16,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            tool: Tool::Pen,
            color: DEFAULT_STROKE_COLOR,
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
        }
    }
}

impl ToolState {
    pub fn stroke_width_in_world_units(&self) -> u32 {
        u32::from(self.stroke_width_px) * u32::from(WORLD_UNITS_PER_PIXEL)
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub tools: ToolState,
    board: StandInBoard,
    stroke_in_progress: Vec<WorldPos>,
    scroll_offset_in_world_units: i64,
}

impl Model {
    pub fn board(&self) -> &StandInBoard {
        &self.board
    }

    pub fn stroke_in_progress(&self) -> &[WorldPos] {
        &self.stroke_in_progress
    }

    pub fn scroll_offset_in_world_units(&self) -> i64 {
        self.scroll_offset_in_world_units
    }

    pub fn is_printable(&self, point: WorldPos) -> bool {
        (0..=self.board.last_page_index())
            .any(|page_index| Aabb::printable_part_of_page(page_index).contains(point))
    }

    pub fn page_nearest_to_top_of_view(&self) -> u16 {
        let distance = distance_between_page_tops_in_world_units();
        let nearest = (self.scroll_offset_in_world_units + distance / 2) / distance;
        u16::try_from(nearest)
            .unwrap_or(u16::MAX)
            .min(self.board.last_page_index())
    }

    pub fn scroll_by(&mut self, world_units: i64) {
        let last_page_top = self.scroll_offset_showing_top_of(self.board.last_page_index());
        self.scroll_offset_in_world_units = self
            .scroll_offset_in_world_units
            .saturating_add(world_units)
            .clamp(0, last_page_top);
    }

    pub fn scroll_to_top_of(&mut self, page_index: u16) {
        self.scroll_offset_in_world_units = self.scroll_offset_showing_top_of(page_index);
    }

    pub fn add_page_at_end(&mut self) {
        let new_last_page_index = self.board.last_page_index().saturating_add(1);
        self.board.apply_add_page_idempotently(new_last_page_index);
        self.scroll_to_top_of(new_last_page_index);
    }

    pub fn extend_stroke_in_progress(&mut self, point: WorldPos) {
        if self.stroke_in_progress.last() != Some(&point) {
            self.stroke_in_progress.push(point);
        }
    }

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

    pub fn erase_strokes_where(&mut self, is_erased: impl Fn(&StandInStroke) -> bool) {
        self.board.strokes.retain(|stroke| !is_erased(stroke));
    }

    fn scroll_offset_showing_top_of(&self, page_index: u16) -> i64 {
        page_top_in_world_units(page_index.min(self.board.last_page_index()))
    }
}
