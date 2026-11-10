//! The controller for a board of one A4 page.

use wb_core::{Aabb, Intersects, WorldPos};

use super::{CanvasInput, Controller};
use crate::camera::Camera;
use crate::config::{ERASER_RADIUS_ON_SCREEN_PX, PAGE, PRINTABLE_AREA, SPACE_AROUND_PAGE_PX};
use crate::local_board::LocalBoard;
use crate::model::Model;
use crate::tool::Tool;

/// Keeps the one page fitted to the canvas and lets the tools work only on its printable part.
#[derive(Debug, Default)]
pub struct SinglePageController;

/// Whether `point` lies on the printable part of the page, the only place the pen draws and
/// the eraser works.
#[must_use]
pub fn is_printable(point: WorldPos) -> bool {
    PRINTABLE_AREA.intersects(&Aabb::new(point, point))
}

impl Controller for SinglePageController {
    /// Fits the camera to the page on the canvas, then lets the current tool act on the
    /// printable part: the pen extends the stroke in progress and finishes it once the button is
    /// released or the pointer leaves the printable part, so re-entering starts a new stroke;
    /// the eraser erases every stroke within its radius of the pointer.
    fn handle_canvas_input<M: Model>(&mut self, model: &mut M, input: &CanvasInput) {
        *model.camera_mut() = M::Camera::fitting(PAGE, input.area, SPACE_AROUND_PAGE_PX);
        let printable_point = input
            .pointer_pressed_at
            .map(|pos| model.camera().screen_to_world(pos))
            .filter(|&point| is_printable(point));
        match model.tools().tool {
            Tool::Pen => {
                if let Some(point) = printable_point {
                    model.extend_stroke_in_progress(point);
                }
                if !input.is_pointer_held || printable_point.is_none() {
                    model.finish_stroke_in_progress();
                }
            }
            Tool::Eraser => {
                if let Some(centre) = printable_point {
                    let reach = model
                        .camera()
                        .screen_length_to_world(ERASER_RADIUS_ON_SCREEN_PX);
                    model.board_mut().erase_touched(centre, reach);
                }
            }
        }
    }
}
