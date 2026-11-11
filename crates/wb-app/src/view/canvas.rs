//! The board canvas, in two steps per frame so the controller can act in between.

use egui::{Painter, Response, Sense, StrokeKind, Vec2};
use wb_core::{Aabb, Board, Intersects, Page};

use crate::camera::Camera;
use crate::config::{
    BACKGROUND_BEHIND_PAGE, ERASER_OUTLINE, ERASER_RADIUS_ON_SCREEN_PX, PAGE_COLOR,
    PRINTABLE_AREA_OUTLINE,
};
use crate::controller::{CanvasInput, PageStackController};
use crate::local_board::LocalBoard;
use crate::model::Model;
use crate::stroke_renderer::Naive;
use crate::tool::Tool;

/// The canvas area reserved for this frame, between reading its input and painting it.
pub struct AllocatedCanvas {
    response: Response,
    painter: Painter,
}

impl AllocatedCanvas {
    /// Reserves the rest of `ui` for the canvas and reads this frame's pointer and scroll input on
    /// it.
    pub fn new(ui: &mut egui::Ui) -> (CanvasInput, Self) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let is_pointer_held = response.is_pointer_button_down_on();
        let pointer_pressed_at = if is_pointer_held || response.clicked() {
            response.interact_pointer_pos()
        } else {
            None
        };
        let scroll_delta = if response.hovered() {
            ui.input(|input| input.smooth_scroll_delta)
        } else {
            Vec2::ZERO
        };
        let input = CanvasInput {
            area: response.rect,
            pointer_pressed_at,
            is_pointer_held,
            scroll_delta,
        };
        (input, Self { response, painter })
    }

    /// Paints every page in view with its strokes, each clipped to its page; the stroke in
    /// progress, clipped to the page it is on; and the eraser's reach while the pointer is over a
    /// printable area.
    pub fn paint(&self, model: &mut Model) {
        self.painter
            .rect_filled(self.response.rect, 0.0, BACKGROUND_BEHIND_PAGE);
        let in_view = Aabb::new(
            model.camera.screen_to_world(self.response.rect.min),
            model.camera.screen_to_world(self.response.rect.max),
        );
        let page_count = model.board.board().page_count();
        for page in (0..page_count).map(Page) {
            if !page.bounds().intersects(&in_view) {
                continue;
            }
            let printable = model.camera.world_area_to_screen(page.printable_area());
            let page = model.camera.world_area_to_screen(page.bounds());
            self.painter.rect_filled(page, 0.0, PAGE_COLOR);
            self.painter
                .rect_stroke(printable, 0.0, PRINTABLE_AREA_OUTLINE, StrokeKind::Outside);
            model
                .board
                .paint(&self.painter.with_clip_rect(page), &model.camera);
        }
        if let Some(page) = model
            .stroke_in_progress()
            .first()
            .and_then(|&first| Page::printable_at(first))
        {
            let page = model.camera.world_area_to_screen(page.bounds());
            Naive::paint_round_line(
                &self.painter.with_clip_rect(page),
                &model.camera,
                model.stroke_in_progress(),
                model.tools.stroke_width_in_world_units(),
                model.tools.color,
            );
        }
        if model.tools.tool == Tool::Eraser
            && let Some(pos) = self.response.hover_pos()
            && PageStackController::printable_area_at(page_count, model.camera.screen_to_world(pos))
                .is_some()
        {
            self.painter
                .circle_stroke(pos, ERASER_RADIUS_ON_SCREEN_PX, ERASER_OUTLINE);
        }
    }
}
