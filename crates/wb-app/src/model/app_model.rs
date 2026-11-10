//! The window's model: one A4 page seen through an affine camera.

use egui::{Painter, Rect};
use wb_core::{PeerId, WorldPos};

use super::Model;
use crate::camera::{AffineCamera, Camera};
use crate::config::{PAGE, SPACE_AROUND_PAGE_PX};
use crate::local_board::{LocalBoard, SimpleLocalBoard};
use crate::tool::ToolState;

/// The model the window runs on: an affine camera over a `SimpleLocalBoard`.
#[derive(Debug)]
pub struct AppModel {
    tools: ToolState,
    camera: AffineCamera,
    board: SimpleLocalBoard,
    stroke_in_progress: Vec<WorldPos>,
}

impl AppModel {
    /// An empty board authored by `me`, with the default pen. The camera is fitted to the canvas
    /// on the first frame.
    #[must_use]
    pub fn new(me: PeerId) -> Self {
        Self {
            tools: ToolState::default(),
            camera: AffineCamera::fitting(PAGE, Rect::ZERO, SPACE_AROUND_PAGE_PX),
            board: SimpleLocalBoard::new(me),
            stroke_in_progress: Vec::new(),
        }
    }
}

impl Model for AppModel {
    type Camera = AffineCamera;
    type Board = SimpleLocalBoard;

    fn tools(&self) -> &ToolState {
        &self.tools
    }

    fn tools_mut(&mut self) -> &mut ToolState {
        &mut self.tools
    }

    fn camera(&self) -> &Self::Camera {
        &self.camera
    }

    fn camera_mut(&mut self) -> &mut Self::Camera {
        &mut self.camera
    }

    fn board(&self) -> &Self::Board {
        &self.board
    }

    fn board_mut(&mut self) -> &mut Self::Board {
        &mut self.board
    }

    fn paint_board(&mut self, painter: &Painter) {
        self.board.paint(painter, &self.camera);
    }

    fn stroke_in_progress(&self) -> &[WorldPos] {
        &self.stroke_in_progress
    }

    fn extend_stroke_in_progress(&mut self, point: WorldPos) {
        if self.stroke_in_progress.last() != Some(&point) {
            self.stroke_in_progress.push(point);
        }
    }

    fn finish_stroke_in_progress(&mut self) {
        let points = std::mem::take(&mut self.stroke_in_progress);
        let width = self.tools.stroke_width_in_world_units();
        if let Err(error) = self.board.add_stroke(&points, self.tools.color, width) {
            eprintln!("stroke dropped: {error}");
        }
    }
}
