//! The `Model` trait and its models.

mod app_model;

pub use app_model::AppModel;

use egui::Painter;
use wb_core::WorldPos;

use crate::camera::Camera;
use crate::local_board::LocalBoard;
use crate::tool::ToolState;

/// Everything the window shows: the pen settings, the camera, the board and the stroke being
/// drawn. Views only read it; a controller is its only writer.
pub trait Model {
    /// The view onto the board.
    type Camera: Camera;
    /// The strokes and everything kept in step with them.
    type Board: LocalBoard;

    /// The pen settings; any combination is valid.
    fn tools(&self) -> &ToolState;
    /// The pen settings, to change.
    fn tools_mut(&mut self) -> &mut ToolState;

    /// The view onto the board.
    fn camera(&self) -> &Self::Camera;
    /// The view onto the board, to pan, zoom or fit; any camera is valid.
    fn camera_mut(&mut self) -> &mut Self::Camera;

    /// The strokes.
    fn board(&self) -> &Self::Board;
    /// The strokes, to add or erase.
    fn board_mut(&mut self) -> &mut Self::Board;

    /// Paints the board into `painter` as the camera shows it.
    fn paint_board(&mut self, painter: &Painter);

    /// Points of the stroke being drawn, not on the board yet; empty between strokes.
    fn stroke_in_progress(&self) -> &[WorldPos];

    /// Adds `point` to the stroke being drawn, unless it repeats the last point.
    fn extend_stroke_in_progress(&mut self, point: WorldPos);

    /// Moves the stroke being drawn onto the board with the current colour and width. Does
    /// nothing when no stroke is in progress. A stroke the codec cannot encode is dropped and
    /// reported on stderr.
    fn finish_stroke_in_progress(&mut self);
}
