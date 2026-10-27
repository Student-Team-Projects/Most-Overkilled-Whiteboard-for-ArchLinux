use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2};

use crate::stand_in::{
    A4_HEIGHT_PX, A4_WIDTH_PX, GAP_BETWEEN_PAGES_PX, StandInStroke, UNPRINTABLE_MARGIN_PX,
    WORLD_UNITS_PER_PIXEL, WorldPos,
};
use crate::tool::{Tool, ToolState};

const ERASER_RADIUS_ON_SCREEN_PX: f32 = 8.0;
const SPACE_AROUND_PAGE_PX: f32 = 16.0;
const MIN_POINTS_IN_STROKE: usize = 2;

const BACKGROUND_BEHIND_PAGES: Color32 = Color32::from_gray(40);
const UNPRINTABLE_MARGIN_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_gray(220),
};
const ERASER_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};

fn page_size_px() -> Vec2 {
    Vec2::new(f32::from(A4_WIDTH_PX), f32::from(A4_HEIGHT_PX))
}

fn distance_between_page_tops_px() -> f32 {
    f32::from(A4_HEIGHT_PX) + f32::from(GAP_BETWEEN_PAGES_PX)
}

#[derive(Clone, Copy, Debug)]
struct PageView {
    first_page_top_left: Pos2,
    zoom: f32,
}

impl PageView {
    fn fitting_one_whole_page(area: Rect, scroll_offset_px: f32) -> Self {
        let room = (area.size() - Vec2::splat(2.0 * SPACE_AROUND_PAGE_PX)).max(Vec2::ZERO);
        let page = page_size_px();
        let zoom = (room.x / page.x).min(room.y / page.y);
        let left = area.center().x - page.x * zoom / 2.0;
        let top = area.top() + SPACE_AROUND_PAGE_PX - scroll_offset_px * zoom;
        Self {
            first_page_top_left: Pos2::new(left, top),
            zoom,
        }
    }

    fn page_on_screen(self, page_index: u16) -> Rect {
        let page_top = Vec2::new(0.0, f32::from(page_index) * distance_between_page_tops_px());
        Rect::from_min_size(
            self.first_page_top_left + page_top * self.zoom,
            page_size_px() * self.zoom,
        )
    }

    fn drawable_part_of(self, page_on_screen: Rect) -> Rect {
        page_on_screen.shrink(f32::from(UNPRINTABLE_MARGIN_PX) * self.zoom)
    }

    #[allow(clippy::cast_possible_truncation)] // on-screen positions × 16 always fit in i64
    fn screen_to_world(self, pos: Pos2) -> WorldPos {
        let world_units =
            (pos - self.first_page_top_left) / self.zoom * f32::from(WORLD_UNITS_PER_PIXEL);
        WorldPos {
            x: world_units.x.round() as i64,
            y: world_units.y.round() as i64,
        }
    }

    #[allow(clippy::cast_precision_loss)] // page-stack coordinates stay far below f32's exact range
    fn world_to_screen(self, point: WorldPos) -> Pos2 {
        let pixels = Vec2::new(point.x as f32, point.y as f32) / f32::from(WORLD_UNITS_PER_PIXEL);
        self.first_page_top_left + pixels * self.zoom
    }

    fn stroke_width_on_screen(self, width_in_world_units: u16) -> f32 {
        f32::from(width_in_world_units) / f32::from(WORLD_UNITS_PER_PIXEL) * self.zoom
    }
}

#[derive(Debug, Default)]
pub struct Canvas {
    strokes: Vec<StandInStroke>,
    stroke_in_progress: Vec<WorldPos>,
    last_page_index: u16,
    scroll_offset_px: f32,
}

impl Canvas {
    pub fn ui(&mut self, ui: &mut egui::Ui, tools: &ToolState) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());
        self.scroll_with_wheel(ui, &response);
        let view = PageView::fitting_one_whole_page(response.rect, self.scroll_offset_px);
        let drawable_areas = self.paint_visible_pages(&painter, view, response.rect);
        let is_drawable = |pos: &Pos2| drawable_areas.iter().any(|area| area.contains(*pos));
        let dragged_to = if response.dragged() {
            response.interact_pointer_pos().filter(is_drawable)
        } else {
            None
        };

        match tools.tool {
            Tool::Pen => {
                if let Some(pos) = dragged_to {
                    self.extend_stroke_in_progress(view.screen_to_world(pos));
                }
                if response.drag_stopped() {
                    self.finish_stroke_in_progress(tools);
                }
            }
            Tool::Eraser => {
                if let Some(pos) = dragged_to {
                    self.erase_strokes_touching(view, pos);
                }
            }
        }

        self.paint_strokes(&painter, view, &drawable_areas, tools);
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
                page_number(self.last_page_index)
            ));
            let next = ui.add_enabled(current_page < self.last_page_index, egui::Button::new("▶"));
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
        });
    }

    fn scroll_with_wheel(&mut self, ui: &egui::Ui, response: &Response) {
        let zoom = PageView::fitting_one_whole_page(response.rect, self.scroll_offset_px).zoom;
        if response.contains_pointer() && zoom > 0.0 {
            self.scroll_offset_px -= ui.input(|input| input.smooth_scroll_delta.y) / zoom;
        }
        let last_page_top = self.scroll_offset_showing_top_of(self.last_page_index);
        self.scroll_offset_px = self.scroll_offset_px.clamp(0.0, last_page_top);
    }

    fn paint_visible_pages(&self, painter: &Painter, view: PageView, area: Rect) -> Vec<Rect> {
        painter.rect_filled(area, 0.0, BACKGROUND_BEHIND_PAGES);
        (0..=self.last_page_index)
            .map(|page_index| view.page_on_screen(page_index))
            .filter(|page| page.intersects(area))
            .map(|page| {
                let drawable = view.drawable_part_of(page);
                painter.rect_filled(page, 0.0, Color32::WHITE);
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
        view: PageView,
        drawable_areas: &[Rect],
        tools: &ToolState,
    ) {
        for &drawable in drawable_areas {
            let painter = painter.with_clip_rect(drawable);
            for stroke in &self.strokes {
                let [r, g, b, a] = stroke.unmultiplied_rgba;
                let color = Color32::from_rgba_unmultiplied(r, g, b, a);
                paint_polyline(
                    &painter,
                    view,
                    &stroke.points,
                    stroke.width_in_world_units,
                    color,
                );
            }
            paint_polyline(
                &painter,
                view,
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
        if points.len() >= MIN_POINTS_IN_STROKE {
            self.strokes.push(StandInStroke {
                unmultiplied_rgba: tools.color.to_srgba_unmultiplied(),
                width_in_world_units: tools.stroke_width_in_world_units(),
                points,
            });
        }
    }

    fn erase_strokes_touching(&mut self, view: PageView, eraser_center: Pos2) {
        self.strokes
            .retain(|stroke| !eraser_touches(view, stroke, eraser_center));
    }

    fn add_page_at_end(&mut self) {
        self.last_page_index = self.last_page_index.saturating_add(1);
        self.scroll_to_top_of(self.last_page_index);
    }

    fn scroll_to_top_of(&mut self, page_index: u16) {
        self.scroll_offset_px = self.scroll_offset_showing_top_of(page_index);
    }

    fn scroll_offset_showing_top_of(&self, page_index: u16) -> f32 {
        f32::from(page_index.min(self.last_page_index)) * distance_between_page_tops_px()
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=last page
    fn page_nearest_to_top_of_view(&self) -> u16 {
        let page_index = (self.scroll_offset_px / distance_between_page_tops_px()).round();
        page_index.clamp(0.0, f32::from(self.last_page_index)) as u16
    }
}

fn eraser_touches(view: PageView, stroke: &StandInStroke, eraser_center: Pos2) -> bool {
    let half_stroke_width = view.stroke_width_on_screen(stroke.width_in_world_units) / 2.0;
    let reach = ERASER_RADIUS_ON_SCREEN_PX + half_stroke_width;
    stroke.points.windows(2).any(|segment| {
        let start = view.world_to_screen(segment[0]);
        let end = view.world_to_screen(segment[1]);
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

fn paint_polyline(
    painter: &Painter,
    view: PageView,
    points: &[WorldPos],
    width_in_world_units: u16,
    color: Color32,
) {
    let on_screen = points.iter().map(|&p| view.world_to_screen(p)).collect();
    let width = view.stroke_width_on_screen(width_in_world_units);
    painter.line(on_screen, Stroke::new(width, color));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
    }

    const START: Pos2 = Pos2::new(0.0, 0.0);
    const END: Pos2 = Pos2::new(10.0, 0.0);

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
