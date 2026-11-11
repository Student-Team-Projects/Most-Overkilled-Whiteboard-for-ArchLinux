//! Everything the window shows.

use egui::Rect;
use wb_core::{PeerId, WorldPos};

use crate::camera::{AffineCamera, Camera};
use crate::config::SPACE_AROUND_PAGE_PX;
use crate::local_board::{LocalBoard, SimpleLocalBoard};
use crate::pages::PageStack;
use crate::tool::ToolState;

/// Everything the window shows: the pen settings, the camera, the board, its pages and the
/// stroke being drawn. Views only read it; a controller is its only writer.
#[derive(Debug)]
pub struct Model {
    /// The pen settings; any combination is valid.
    pub tools: ToolState,
    /// The view onto the board; any camera is valid.
    pub camera: AffineCamera,
    /// The strokes and everything kept in step with them.
    pub board: SimpleLocalBoard,
    /// The pages strokes are drawn on; kept here until pages are board state (T30 #66).
    pub pages: PageStack,
    stroke_in_progress: Vec<WorldPos>,
}

impl Model {
    /// An empty board of one page authored by `me`, with the default pen. The camera is fitted to the canvas
    /// on the first frame.
    #[must_use]
    pub fn new(me: PeerId) -> Self {
        Self {
            tools: ToolState::default(),
            camera: AffineCamera::fitting(PageStack::page(0), Rect::ZERO, SPACE_AROUND_PAGE_PX),
            board: SimpleLocalBoard::new(me),
            pages: PageStack::default(),
            stroke_in_progress: Vec::new(),
        }
    }

    /// Points of the stroke being drawn, not on the board yet; empty between strokes.
    #[must_use]
    pub fn stroke_in_progress(&self) -> &[WorldPos] {
        &self.stroke_in_progress
    }

    /// Adds `point` to the stroke being drawn, unless it repeats the last point.
    pub fn extend_stroke_in_progress(&mut self, point: WorldPos) {
        if self.stroke_in_progress.last() != Some(&point) {
            self.stroke_in_progress.push(point);
        }
    }

    /// Moves the stroke being drawn onto the board with the current colour and width. Does
    /// nothing when no stroke is in progress. A stroke the codec cannot encode is dropped and
    /// reported on stderr.
    pub fn finish_stroke_in_progress(&mut self) {
        let points = std::mem::take(&mut self.stroke_in_progress);
        let width = self.tools.stroke_width_in_world_units();
        if let Err(error) = self.board.add_stroke(&points, self.tools.color, width) {
            eprintln!("stroke dropped: {error}");
        }
    }
}
