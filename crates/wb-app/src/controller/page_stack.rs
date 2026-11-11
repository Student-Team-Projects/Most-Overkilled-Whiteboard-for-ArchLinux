//! The controller for a stack of A4 pages.

use egui::Vec2;
use wb_core::{Aabb, Board, Intersects, Page, WorldPos};

use super::{CanvasInput, Controller};
use crate::camera::{AffineCamera, Camera};
use crate::config::{ERASER_RADIUS_ON_SCREEN_PX, SPACE_AROUND_PAGE_PX};
use crate::local_board::LocalBoard;
use crate::model::Model;
use crate::tool::Tool;

/// Shows one whole page at a time, scrolls along the stack, and lets the tools work only on the
/// printable areas of existing pages.
#[derive(Debug, Default)]
pub struct PageStackController {
    last_held_point: Option<WorldPos>,
}

impl PageStackController {
    /// The printable area holding `point`, edges included, if it lies on one of the first
    /// `page_count` pages.
    #[must_use]
    pub fn printable_area_at(page_count: u32, point: WorldPos) -> Option<Aabb> {
        Page::printable_at(point)
            .filter(|page| page.0 < page_count)
            .map(Page::printable_area)
    }

    /// The centre of `page`.
    #[must_use]
    pub fn page_centre(page: Page) -> WorldPos {
        let bounds = page.bounds();
        WorldPos::new(
            bounds.min.x.midpoint(bounds.max.x),
            bounds.min.y.midpoint(bounds.max.y),
        )
    }

    /// The camera for this frame: a page fitted to the canvas, at the scroll position of
    /// `previous` moved by `scroll`, kept between the centres of the first and the last page.
    fn scrolled_camera(
        page_count: u32,
        previous: AffineCamera,
        input: &CanvasInput,
    ) -> AffineCamera {
        let fitted = AffineCamera::fitting(Page(0).bounds(), input.area, SPACE_AROUND_PAGE_PX);
        let mut camera = AffineCamera::new(
            WorldPos::new(fitted.world_centre.x, previous.world_centre.y),
            fitted.screen_centre,
            fitted.zoom(),
        );
        camera.pan(Vec2::new(0.0, input.scroll_delta.y));
        camera.world_centre.y = camera.world_centre.y.clamp(
            Self::page_centre(Page(0)).y,
            Self::page_centre(Page(page_count.saturating_sub(1))).y,
        );
        camera
    }

    /// The pen at `pointer`: extends the stroke in progress while it stays in the printable area
    /// it started in, and ends it on that area's edge once the pointer leaves. A pointer on a
    /// printable area with no stroke in progress starts one, on the edge where it crossed in if
    /// it was held outside the area a frame before. Releasing the button finishes the stroke.
    fn pen(&self, model: &mut Model, pointer: Option<WorldPos>, is_pointer_held: bool) {
        let page_count = model.board.board().page_count();
        if let Some(point) = pointer {
            let stroke_area = model
                .stroke_in_progress()
                .first()
                .and_then(|&first| Self::printable_area_at(page_count, first));
            if let (Some(area), Some(&last)) = (stroke_area, model.stroke_in_progress().last())
                && !contains(area, point)
            {
                model.extend_stroke_in_progress(edge_towards(area, last, point));
                model.finish_stroke_in_progress();
            }
            if let Some(area) = Self::printable_area_at(page_count, point) {
                if model.stroke_in_progress().is_empty()
                    && let Some(previous) = self.last_held_point
                    && !contains(area, previous)
                {
                    model.extend_stroke_in_progress(edge_towards(area, point, previous));
                }
                model.extend_stroke_in_progress(point);
            }
        }
        if !is_pointer_held {
            model.finish_stroke_in_progress();
        }
    }
}

fn contains(area: Aabb, point: WorldPos) -> bool {
    area.intersects(&Aabb::new(point, point))
}

/// Where the segment from `inside`, a point of `area`, towards `outside` leaves `area`, rounded
/// to whole world units and kept on it.
fn edge_towards(area: Aabb, inside: WorldPos, outside: WorldPos) -> WorldPos {
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

impl Controller for PageStackController {
    /// Fits a page to the canvas and scrolls the stack by the wheel, then lets the current tool
    /// act on the printable areas: the pen draws strokes that each stay on one page, ending and
    /// starting on a printable edge where the pointer crossed it; the eraser erases every stroke
    /// within its radius of the pointer.
    fn handle_canvas_input(&mut self, model: &mut Model, input: &CanvasInput) {
        let page_count = model.board.board().page_count();
        model.camera = Self::scrolled_camera(page_count, model.camera, input);
        let pointer = input
            .pointer_pressed_at
            .map(|pos| model.camera.screen_to_world(pos));
        match model.tools.tool {
            Tool::Pen => self.pen(model, pointer, input.is_pointer_held),
            Tool::Eraser => {
                if let Some(centre) =
                    pointer.filter(|&point| Self::printable_area_at(page_count, point).is_some())
                {
                    let reach = model
                        .camera
                        .screen_length_to_world(ERASER_RADIUS_ON_SCREEN_PX);
                    model.board.erase_touched(centre, reach);
                }
            }
        }
        self.last_held_point = pointer.filter(|_| input.is_pointer_held);
    }
}
