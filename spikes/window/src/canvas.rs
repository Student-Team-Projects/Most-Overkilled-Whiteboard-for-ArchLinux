use eframe::egui::{
    self, Color32, Painter, PointerButton, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2,
};

use crate::camera::Camera;
use crate::stand_in::{
    A4_HEIGHT_PX, A4_WIDTH_PX, Aabb, GAP_BETWEEN_PAGES_PX, StandInBoard, StandInStroke,
    UNPRINTABLE_MARGIN_PX, WORLD_UNITS_PER_PIXEL, WorldPos,
};
use crate::tool::{Tool, ToolState};

const ERASER_RADIUS_ON_SCREEN_PX: f32 = 8.0;
const SPACE_AROUND_PAGE_PX: f32 = 16.0;

const BACKGROUND_BEHIND_PAGES: Color32 = Color32::from_gray(40);
const UNPRINTABLE_MARGIN_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_gray(220),
};
const ERASER_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};

fn px_to_world_units(px: u16) -> i64 {
    i64::from(px) * i64::from(WORLD_UNITS_PER_PIXEL)
}

fn distance_between_page_tops_in_world_units() -> i64 {
    px_to_world_units(A4_HEIGHT_PX) + px_to_world_units(GAP_BETWEEN_PAGES_PX)
}

impl Aabb {
    fn of_stacked_page(page_index: u16) -> Self {
        let top = i64::from(page_index) * distance_between_page_tops_in_world_units();
        Self {
            min: WorldPos { x: 0, y: top },
            max: WorldPos {
                x: px_to_world_units(A4_WIDTH_PX),
                y: top + px_to_world_units(A4_HEIGHT_PX),
            },
        }
    }

    fn printable_part(self) -> Self {
        let margin = px_to_world_units(UNPRINTABLE_MARGIN_PX);
        Self {
            min: WorldPos {
                x: self.min.x + margin,
                y: self.min.y + margin,
            },
            max: WorldPos {
                x: self.max.x - margin,
                y: self.max.y - margin,
            },
        }
    }
}

impl Camera {
    fn fitting_first_stacked_page(viewport: Rect) -> Self {
        let room = (viewport.size() - Vec2::splat(2.0 * SPACE_AROUND_PAGE_PX)).max(Vec2::ZERO);
        let zoom = (room.x / f32::from(A4_WIDTH_PX)).min(room.y / f32::from(A4_HEIGHT_PX));
        let first_page = Aabb::of_stacked_page(0);
        let page_centre = WorldPos {
            x: first_page.max.x / 2,
            y: 0,
        };
        let mut camera = Self::new(page_centre, viewport.center(), zoom);
        let first_page_top_on_screen = camera.world_to_screen(first_page.min).y;
        camera.pan(Vec2::new(
            0.0,
            viewport.top() + SPACE_AROUND_PAGE_PX - first_page_top_on_screen,
        ));
        camera
    }

    fn rect_on_screen(self, area: Aabb) -> Rect {
        Rect::from_min_max(
            self.world_to_screen(area.min),
            self.world_to_screen(area.max),
        )
    }
}

#[derive(Debug, Default)]
pub struct StackedPageCanvas {
    board: StandInBoard,
    stroke_in_progress: Vec<WorldPos>,
    camera: Option<Camera>,
    viewport_top: f32,
}

impl StackedPageCanvas {
    pub fn ui(&mut self, ui: &mut egui::Ui, tools: &ToolState) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.viewport_top = response.rect.top();
        let mut camera = self
            .camera
            .unwrap_or_else(|| Camera::fitting_first_stacked_page(response.rect));
        camera.screen_centre = response.rect.center();
        pan_and_zoom_with_input(ui, &response, &mut camera);
        let camera = self.kept_over_page_stack(camera);
        self.camera = Some(camera);

        let drawable_areas = self.paint_visible_pages(&painter, camera, response.rect);
        let is_drawable = |pos: &Pos2| drawable_areas.iter().any(|area| area.contains(*pos));
        let primary_held =
            response.is_pointer_button_down_on() && ui.input(|input| input.pointer.primary_down());
        let held_or_clicked_at = if primary_held || response.clicked() {
            response.interact_pointer_pos().filter(is_drawable)
        } else {
            None
        };

        match tools.tool {
            Tool::Pen => {
                if let Some(pos) = held_or_clicked_at {
                    self.extend_stroke_in_progress(camera.screen_to_world(pos));
                }
                if !primary_held {
                    self.finish_stroke_in_progress(tools);
                }
            }
            Tool::Eraser => {
                if let Some(pos) = held_or_clicked_at {
                    self.erase_strokes_touching(camera, pos);
                }
            }
        }

        self.paint_strokes(&painter, camera, &drawable_areas, tools);
        if tools.tool == Tool::Eraser
            && let Some(pos) = response.hover_pos().filter(is_drawable)
        {
            painter.circle_stroke(pos, ERASER_RADIUS_ON_SCREEN_PX, ERASER_OUTLINE);
        }
    }

    pub fn page_bar(&mut self, ui: &mut egui::Ui) {
        let current_page = self.page_nearest_to_top_of_view();
        let page_number = |page_index: u16| u32::from(page_index) + 1;
        ui.horizontal(|ui| {
            let previous = ui.add_enabled(current_page > 0, egui::Button::new("◀"));
            if previous
                .on_hover_text("Scroll to the previous page")
                .clicked()
            {
                self.scroll_to_top_of(current_page - 1);
            }
            ui.label(format!(
                "Page {} / {}",
                page_number(current_page),
                page_number(self.board.last_page_index())
            ));
            let next = ui.add_enabled(
                current_page < self.board.last_page_index(),
                egui::Button::new("▶"),
            );
            if next.on_hover_text("Scroll to the next page").clicked() {
                self.scroll_to_top_of(current_page + 1);
            }
            if ui
                .button("➕")
                .on_hover_text("Add a page at the end")
                .clicked()
            {
                self.add_page_at_end();
            }
            if let Some(camera) = self.camera {
                ui.label(format!("{:.0} %", camera.zoom() * 100.0))
                    .on_hover_text("Zoom: Ctrl + scroll or pinch; pan: scroll, or drag with the middle or right button");
            }
        });
    }

    fn kept_over_page_stack(&self, mut camera: Camera) -> Camera {
        let first_page = Aabb::of_stacked_page(0);
        let last_page = Aabb::of_stacked_page(self.board.last_page_index());
        camera.world_centre = WorldPos {
            x: camera
                .world_centre
                .x
                .clamp(first_page.min.x, first_page.max.x),
            y: camera
                .world_centre
                .y
                .clamp(first_page.min.y, last_page.max.y),
        };
        camera
    }

    fn clamped_page_index_at(&self, world_y: i64) -> u16 {
        let last = self.board.last_page_index();
        let index = world_y
            .div_euclid(distance_between_page_tops_in_world_units())
            .clamp(0, i64::from(last));
        u16::try_from(index).unwrap_or(last)
    }

    fn paint_visible_pages(&self, painter: &Painter, camera: Camera, area: Rect) -> Vec<Rect> {
        painter.rect_filled(area, 0.0, BACKGROUND_BEHIND_PAGES);
        let visible = camera.visible_area(area.size());
        let first = self.clamped_page_index_at(visible.min.y);
        let last = self.clamped_page_index_at(visible.max.y);
        (first..=last)
            .map(Aabb::of_stacked_page)
            .filter(|page| camera.rect_on_screen(*page).intersects(area))
            .map(|page| {
                let drawable = camera.rect_on_screen(page.printable_part());
                painter.rect_filled(camera.rect_on_screen(page), 0.0, Color32::WHITE);
                painter.rect_stroke(
                    drawable,
                    0.0,
                    UNPRINTABLE_MARGIN_OUTLINE,
                    StrokeKind::Outside,
                );
                drawable
            })
            .collect()
    }

    fn paint_strokes(
        &self,
        painter: &Painter,
        camera: Camera,
        drawable_areas: &[Rect],
        tools: &ToolState,
    ) {
        for &drawable in drawable_areas {
            let painter = painter.with_clip_rect(drawable);
            for stroke in &self.board.strokes {
                let [r, g, b, a] = stroke.unmultiplied_rgba;
                let color = Color32::from_rgba_unmultiplied(r, g, b, a);
                paint_rounded_polyline(
                    &painter,
                    camera,
                    &stroke.points,
                    stroke.width_in_world_units,
                    color,
                );
            }
            paint_rounded_polyline(
                &painter,
                camera,
                &self.stroke_in_progress,
                tools.stroke_width_in_world_units(),
                tools.color,
            );
        }
    }

    fn extend_stroke_in_progress(&mut self, point: WorldPos) {
        if self.stroke_in_progress.last() != Some(&point) {
            self.stroke_in_progress.push(point);
        }
    }

    fn finish_stroke_in_progress(&mut self, tools: &ToolState) {
        let points = std::mem::take(&mut self.stroke_in_progress);
        if !points.is_empty() {
            self.board.strokes.push(StandInStroke {
                unmultiplied_rgba: tools.color.to_srgba_unmultiplied(),
                width_in_world_units: tools.stroke_width_in_world_units(),
                points,
            });
        }
    }

    fn erase_strokes_touching(&mut self, camera: Camera, eraser_center: Pos2) {
        self.board
            .strokes
            .retain(|stroke| !eraser_touches(camera, stroke, eraser_center));
    }

    fn add_page_at_end(&mut self) {
        let new_last_page_index = self.board.last_page_index().saturating_add(1);
        self.board.apply_add_page_idempotently(new_last_page_index);
        self.scroll_to_top_of(new_last_page_index);
    }

    fn scroll_to_top_of(&mut self, page_index: u16) {
        let page = Aabb::of_stacked_page(page_index.min(self.board.last_page_index()));
        if let Some(camera) = &mut self.camera {
            let page_top_on_screen = camera.world_to_screen(page.min).y;
            camera.pan(Vec2::new(
                0.0,
                self.viewport_top + SPACE_AROUND_PAGE_PX - page_top_on_screen,
            ));
        }
    }

    fn page_nearest_to_top_of_view(&self) -> u16 {
        let Some(camera) = self.camera else {
            return 0;
        };
        let top_of_view = camera
            .screen_to_world(Pos2::new(camera.screen_centre.x, self.viewport_top))
            .y;
        self.clamped_page_index_at(top_of_view + distance_between_page_tops_in_world_units() / 2)
    }
}

fn pan_and_zoom_with_input(ui: &egui::Ui, response: &Response, camera: &mut Camera) {
    if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
        camera.pan(response.drag_delta());
    }
    if let Some(pointer) = response.hover_pos() {
        let (scroll, zoom_factor) =
            ui.input(|input| (input.smooth_scroll_delta, input.zoom_delta()));
        camera.pan(scroll);
        if (zoom_factor - 1.0).abs() > f32::EPSILON {
            camera.zoom_at(pointer, zoom_factor);
        }
    }
}

fn eraser_touches(camera: Camera, stroke: &StandInStroke, eraser_center: Pos2) -> bool {
    let half_stroke_width = camera.world_length_to_screen(stroke.width_in_world_units) / 2.0;
    let reach = ERASER_RADIUS_ON_SCREEN_PX + half_stroke_width;
    if let [dot] = stroke.points.as_slice() {
        return camera.world_to_screen(*dot).distance(eraser_center) <= reach;
    }
    stroke.points.windows(2).any(|segment| {
        let start = camera.world_to_screen(segment[0]);
        let end = camera.world_to_screen(segment[1]);
        distance_from_point_to_segment(eraser_center, start, end) <= reach
    })
}

fn distance_from_point_to_segment(point: Pos2, start: Pos2, end: Pos2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_sq();
    let closest_fraction_along_segment = if length_squared > 0.0 {
        ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    point.distance(start + segment * closest_fraction_along_segment)
}

fn paint_rounded_polyline(
    painter: &Painter,
    camera: Camera,
    points: &[WorldPos],
    width_in_world_units: u32,
    color: Color32,
) {
    let width = camera.world_length_to_screen(width_in_world_units);
    let on_screen: Vec<Pos2> = points.iter().map(|&p| camera.world_to_screen(p)).collect();
    for segment in on_screen.windows(2) {
        painter.line_segment([segment[0], segment[1]], Stroke::new(width, color));
    }
    for &point in &on_screen {
        painter.circle_filled(point, width / 2.0, color);
    }
}

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;
    use crate::camera::{MAX_ZOOM, MIN_ZOOM};

    const FLOAT_TOLERANCE: f32 = 1e-6;
    const START: Pos2 = Pos2::new(0.0, 0.0);
    const END: Pos2 = Pos2::new(10.0, 0.0);
    const VIEWPORT: Rect = Rect::from_min_max(Pos2::new(56.0, 40.0), Pos2::new(1280.0, 770.0));
    const VERY_TALL_VIEWPORT: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(4_000.0, 4_000.0));
    const DOT_POSITION: WorldPos = WorldPos {
        x: 4_000,
        y: 20_000,
    };
    const DOT_WIDTH_IN_WORLD_UNITS: u32 = 64;
    const DISTANCE_PAST_REACH_EDGE_PX: f32 = 0.5;
    const FAR_OUTSIDE_THE_PAGE_STACK: WorldPos = WorldPos {
        x: -1_000_000,
        y: 1_000_000,
    };

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < FLOAT_TOLERANCE,
            "{actual} != {expected}"
        );
    }

    fn dot_and_eraser_reach_on_screen(camera: Camera) -> (StandInStroke, Pos2, f32) {
        let dot = StandInStroke {
            unmultiplied_rgba: Color32::BLACK.to_srgba_unmultiplied(),
            width_in_world_units: DOT_WIDTH_IN_WORLD_UNITS,
            points: vec![DOT_POSITION],
        };
        let reach = ERASER_RADIUS_ON_SCREEN_PX
            + camera.world_length_to_screen(DOT_WIDTH_IN_WORLD_UNITS) / 2.0;
        (dot, camera.world_to_screen(DOT_POSITION), reach)
    }

    #[test]
    fn opening_view_shows_the_first_page_top_centred_below_the_space() {
        let camera = Camera::fitting_first_stacked_page(VIEWPORT);
        let first_page = camera.rect_on_screen(Aabb::of_stacked_page(0));
        assert!((first_page.top() - (VIEWPORT.top() + SPACE_AROUND_PAGE_PX)).abs() < 0.5);
        assert!((first_page.center().x - VIEWPORT.center().x).abs() < 0.5);
        assert!((MIN_ZOOM..=MAX_ZOOM).contains(&camera.zoom()));
    }

    #[test]
    fn opening_view_in_a_very_tall_window_stops_at_max_zoom() {
        let camera = Camera::fitting_first_stacked_page(VERY_TALL_VIEWPORT);
        assert_close(camera.zoom(), MAX_ZOOM);
    }

    #[test]
    fn camera_far_outside_is_pulled_back_over_the_page_stack() {
        let canvas = StackedPageCanvas::default();
        let camera = Camera::new(FAR_OUTSIDE_THE_PAGE_STACK, VIEWPORT.center(), 1.0);
        let kept = canvas.kept_over_page_stack(camera);
        let only_page = Aabb::of_stacked_page(0);
        assert_eq!(
            kept.world_centre,
            WorldPos {
                x: only_page.min.x,
                y: only_page.max.y
            }
        );
    }

    #[test]
    fn eraser_touches_a_dot_just_inside_its_reach() {
        let camera = Camera::fitting_first_stacked_page(VIEWPORT);
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(camera);
        let eraser_center = dot_on_screen + Vec2::new(reach - DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(eraser_touches(camera, &dot, eraser_center));
    }

    #[test]
    fn eraser_misses_a_dot_just_outside_its_reach() {
        let camera = Camera::fitting_first_stacked_page(VIEWPORT);
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(camera);
        let eraser_center = dot_on_screen + Vec2::new(reach + DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(!eraser_touches(camera, &dot, eraser_center));
    }

    #[test]
    fn point_beside_the_segment_measures_straight_across() {
        let distance = distance_from_point_to_segment(Pos2::new(5.0, 3.0), START, END);
        assert_close(distance, 3.0);
    }

    #[test]
    fn point_past_the_end_measures_to_the_end() {
        let distance = distance_from_point_to_segment(Pos2::new(13.0, 4.0), START, END);
        assert_close(distance, 5.0);
    }

    #[test]
    fn zero_length_segment_measures_to_its_only_point() {
        let distance = distance_from_point_to_segment(Pos2::new(3.0, 4.0), START, START);
        assert_close(distance, 5.0);
    }
}
