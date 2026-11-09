//! The page canvas, in two steps per frame so the controller can act in between.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind};

use crate::camera::StackedPageCamera;
use crate::config::{
    BACKGROUND_BEHIND_PAGES, ERASER_OUTLINE, ERASER_RADIUS_ON_SCREEN_PX, UNPRINTABLE_MARGIN_OUTLINE,
};
use crate::controller::CanvasInput;
use crate::model::Model;
use crate::stand_in::{Aabb, Rgba, WorldPos};
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
        let scroll_on_screen = if response.contains_pointer() {
            ui.input(|input| input.smooth_scroll_delta.y)
        } else {
            0.0
        };
        let is_pointer_held = response.is_pointer_button_down_on();
        let pointer_pressed_at = if is_pointer_held || response.clicked() {
            response.interact_pointer_pos()
        } else {
            None
        };
        let input = CanvasInput {
            area: response.rect,
            scroll_on_screen,
            pointer_pressed_at,
            is_pointer_held,
        };
        (input, Self { response, painter })
    }
}

/// Paints the visible pages, their strokes and the stroke in progress,
/// clipped to the printable parts, plus the eraser outline under the pointer.
pub fn paint(canvas: &AllocatedCanvas, model: &Model) {
    let area = canvas.response.rect;
    let camera =
        StackedPageCamera::fitting_one_whole_page(area, model.scroll_offset_in_world_units());
    let printable_areas = paint_visible_pages(&canvas.painter, camera, area, model);
    paint_strokes(&canvas.painter, camera, &printable_areas, model);
    if model.tools.tool == Tool::Eraser
        && let Some(pos) = canvas.response.hover_pos()
        && model.is_printable(camera.screen_to_world(pos))
    {
        canvas
            .painter
            .circle_stroke(pos, ERASER_RADIUS_ON_SCREEN_PX, ERASER_OUTLINE);
    }
}

fn paint_visible_pages(
    painter: &Painter,
    camera: StackedPageCamera,
    area: Rect,
    model: &Model,
) -> Vec<Rect> {
    painter.rect_filled(area, 0.0, BACKGROUND_BEHIND_PAGES);
    (0..=model.board().last_page_index())
        .map(|page_index| {
            let page = camera.world_area_to_screen(Aabb::page(page_index));
            let printable = camera.world_area_to_screen(Aabb::printable_part_of_page(page_index));
            (page, printable)
        })
        .filter(|(page, _)| page.intersects(area))
        .map(|(page, printable)| {
            painter.rect_filled(page, 0.0, Color32::WHITE);
            painter.rect_stroke(
                printable,
                0.0,
                UNPRINTABLE_MARGIN_OUTLINE,
                StrokeKind::Outside,
            );
            printable
        })
        .collect()
}

fn paint_strokes(
    painter: &Painter,
    camera: StackedPageCamera,
    printable_areas: &[Rect],
    model: &Model,
) {
    for &printable in printable_areas {
        let painter = painter.with_clip_rect(printable);
        for stroke in &model.board().strokes {
            paint_rounded_polyline(
                &painter,
                camera,
                &stroke.points,
                stroke.width_in_world_units,
                stroke.color,
            );
        }
        paint_rounded_polyline(
            &painter,
            camera,
            model.stroke_in_progress(),
            model.tools.stroke_width_in_world_units(),
            model.tools.color,
        );
    }
}

fn paint_rounded_polyline(
    painter: &Painter,
    camera: StackedPageCamera,
    points: &[WorldPos],
    width_in_world_units: u32,
    color: Rgba,
) {
    let width = camera.world_length_to_screen(width_in_world_units);
    let color = Color32::from(color);
    let on_screen: Vec<Pos2> = points.iter().map(|&p| camera.world_to_screen(p)).collect();
    for segment in on_screen.windows(2) {
        painter.line_segment([segment[0], segment[1]], Stroke::new(width, color));
    }
    for &point in &on_screen {
        painter.circle_filled(point, width / 2.0, color);
    }
}
