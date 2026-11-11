//! The board canvas, in two steps per frame so the controller can act in between.

use egui::{Painter, Response, Sense, StrokeKind};

use crate::camera::Camera;
use crate::config::{
    BACKGROUND_BEHIND_PAGE, ERASER_OUTLINE, ERASER_RADIUS_ON_SCREEN_PX, PAGE_COLOR,
    PRINTABLE_AREA_OUTLINE,
};
use crate::controller::{CanvasInput, SinglePageController};
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
    /// Reserves the rest of `ui` for the canvas and reads this frame's pointer input on it.
    pub fn new(ui: &mut egui::Ui) -> (CanvasInput, Self) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let is_pointer_held = response.is_pointer_button_down_on();
        let pointer_pressed_at = if is_pointer_held || response.clicked() {
            response.interact_pointer_pos()
        } else {
            None
        };
        let input = CanvasInput {
            area: response.rect,
            pointer_pressed_at,
            is_pointer_held,
        };
        (input, Self { response, painter })
    }

    /// Paints the page, its strokes and the stroke in progress clipped to the printable part,
    /// and the eraser's reach while the pointer is over the printable part.
    pub fn paint(&self, model: &mut Model) {
        let printable = model
            .camera
            .world_area_to_screen(SinglePageController::PRINTABLE_AREA);
        let page = model
            .camera
            .world_area_to_screen(SinglePageController::PAGE);
        self.painter
            .rect_filled(self.response.rect, 0.0, BACKGROUND_BEHIND_PAGE);
        self.painter.rect_filled(page, 0.0, PAGE_COLOR);
        self.painter
            .rect_stroke(printable, 0.0, PRINTABLE_AREA_OUTLINE, StrokeKind::Outside);
        let printable_painter = self.painter.with_clip_rect(printable);
        model.board.paint(&printable_painter, &model.camera);
        Naive::paint_round_line(
            &printable_painter,
            &model.camera,
            model.stroke_in_progress(),
            model.tools.stroke_width_in_world_units(),
            model.tools.color,
        );
        if model.tools.tool == Tool::Eraser
            && let Some(pos) = self.response.hover_pos()
            && SinglePageController::is_printable(model.camera.screen_to_world(pos))
        {
            self.painter
                .circle_stroke(pos, ERASER_RADIUS_ON_SCREEN_PX, ERASER_OUTLINE);
        }
    }
}
