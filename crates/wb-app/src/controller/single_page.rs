//! The controller for a board of one A4 page.

use wb_core::{Aabb, Intersects, WorldPos};

use super::{CanvasInput, Controller};
use crate::camera::{AffineCamera, Camera};
use crate::config::{
    A4_HEIGHT_PX, A4_WIDTH_PX, ERASER_RADIUS_ON_SCREEN_PX, PRINT_MARGIN_PX, SPACE_AROUND_PAGE_PX,
    WORLD_UNITS_PER_PIXEL,
};
use crate::local_board::LocalBoard;
use crate::model::Model;
use crate::tool::Tool;

/// Keeps the one page fitted to the canvas and lets the tools work only on its printable part.
#[derive(Debug, Default)]
pub struct SinglePageController {
    pointer_held_outside: Option<WorldPos>,
}

const UNITS_PER_PX: i64 = WORLD_UNITS_PER_PIXEL as i64;
const PRINT_MARGIN: i64 = PRINT_MARGIN_PX * UNITS_PER_PX;

impl SinglePageController {
    /// The one A4 page, its top-left corner at the world origin.
    pub const PAGE: Aabb = Aabb::new(
        WorldPos::new(0, 0),
        WorldPos::new(A4_WIDTH_PX * UNITS_PER_PX, A4_HEIGHT_PX * UNITS_PER_PX),
    );

    /// The part of `PAGE` the pen draws on: the page without its print margin.
    pub const PRINTABLE_AREA: Aabb = Aabb::new(
        WorldPos::new(
            Self::PAGE.min.x + PRINT_MARGIN,
            Self::PAGE.min.y + PRINT_MARGIN,
        ),
        WorldPos::new(
            Self::PAGE.max.x - PRINT_MARGIN,
            Self::PAGE.max.y - PRINT_MARGIN,
        ),
    );

    /// Whether `point` lies on the printable part of the page, the only place the pen draws and
    /// the eraser works.
    #[must_use]
    pub fn is_printable(point: WorldPos) -> bool {
        Self::PRINTABLE_AREA.intersects(&Aabb::new(point, point))
    }
}

/// Where the segment from the printable point `inside` towards `outside` leaves the printable
/// part, rounded to whole world units and kept on it.
fn printable_edge_towards(inside: WorldPos, outside: WorldPos) -> WorldPos {
    let area = SinglePageController::PRINTABLE_AREA;
    let fraction = fraction_within(inside.x, outside.x, area.min.x, area.max.x)
        .min(fraction_within(inside.y, outside.y, area.min.y, area.max.y));
    WorldPos::new(
        step_towards(inside.x, outside.x, fraction).clamp(area.min.x, area.max.x),
        step_towards(inside.y, outside.y, fraction).clamp(area.min.y, area.max.y),
    )
}

/// How much of the way from `from` to `to`, as a fraction in `[0, 1]`, stays within
/// `[min, max]`, given that `from` lies within it.
#[allow(clippy::cast_precision_loss)] // pointer moves within one frame are far below 2^53 units
fn fraction_within(from: i64, to: i64, min: i64, max: i64) -> f64 {
    let bound = if to > max {
        max
    } else if to < min {
        min
    } else {
        return 1.0;
    };
    bound.saturating_sub(from) as f64 / to.saturating_sub(from) as f64
}

/// The coordinate `fraction` of the way from `from` to `to`, rounded.
#[allow(clippy::cast_possible_truncation)] // the result lies between `from` and `to`
#[allow(clippy::cast_precision_loss)] // pointer moves within one frame are far below 2^53 units
fn step_towards(from: i64, to: i64, fraction: f64) -> i64 {
    from.saturating_add((to.saturating_sub(from) as f64 * fraction).round() as i64)
}

impl Controller for SinglePageController {
    /// Fits the camera to the page on the canvas, then lets the current tool act on the
    /// printable part: the pen extends the stroke in progress and finishes it once the button is
    /// released or the pointer leaves the printable part, ending it on the printable edge where
    /// the pointer crossed it; a held pointer coming back onto the printable part starts a new
    /// stroke on the edge where it crossed in;
    /// the eraser erases every stroke within its radius of the pointer.
    fn handle_canvas_input(&mut self, model: &mut Model, input: &CanvasInput) {
        model.camera = AffineCamera::fitting(Self::PAGE, input.area, SPACE_AROUND_PAGE_PX);
        let pointer = input
            .pointer_pressed_at
            .map(|pos| model.camera.screen_to_world(pos));
        let printable_point = pointer.filter(|&point| Self::is_printable(point));
        match model.tools.tool {
            Tool::Pen => {
                if let Some(point) = printable_point {
                    if let Some(outside) = self.pointer_held_outside {
                        model.extend_stroke_in_progress(printable_edge_towards(point, outside));
                    }
                    model.extend_stroke_in_progress(point);
                } else if let (Some(outside), Some(&last)) =
                    (pointer, model.stroke_in_progress().last())
                {
                    model.extend_stroke_in_progress(printable_edge_towards(last, outside));
                }
                if !input.is_pointer_held || printable_point.is_none() {
                    model.finish_stroke_in_progress();
                }
            }
            Tool::Eraser => {
                if let Some(centre) = printable_point {
                    let reach = model
                        .camera
                        .screen_length_to_world(ERASER_RADIUS_ON_SCREEN_PX);
                    model.board.erase_touched(centre, reach);
                }
            }
        }
        self.pointer_held_outside =
            pointer.filter(|&point| input.is_pointer_held && !Self::is_printable(point));
    }
}
