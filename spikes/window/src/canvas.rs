use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind};

use crate::config::ERASER_RADIUS_ON_SCREEN_PX;
use crate::page_layout::{distance_between_page_tops_in_world_units, page_top_in_world_units};
use crate::stand_in::{Aabb, StandInBoard, StandInStroke, WorldPos};
use crate::tool::{Tool, ToolState};
use crate::view::StackedPageView;

const BACKGROUND_BEHIND_PAGES: Color32 = Color32::from_gray(40);
const UNPRINTABLE_MARGIN_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_gray(220),
};
const ERASER_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};

#[derive(Debug, Default)]
pub struct StackedPageCanvas {
    board: StandInBoard,
    stroke_in_progress: Vec<WorldPos>,
    scroll_offset_in_world_units: i64,
}

impl StackedPageCanvas {
    pub fn ui(&mut self, ui: &mut egui::Ui, tools: &ToolState) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.scroll_with_wheel(ui, &response);
        let view = StackedPageView::fitting_one_whole_page(
            response.rect,
            self.scroll_offset_in_world_units,
        );
        let drawable_areas = self.paint_visible_pages(&painter, view, response.rect);
        let is_drawable = |pos: &Pos2| drawable_areas.iter().any(|area| area.contains(*pos));
        let held_or_clicked_at = if response.is_pointer_button_down_on() || response.clicked() {
            response.interact_pointer_pos().filter(is_drawable)
        } else {
            None
        };

        match tools.tool {
            Tool::Pen => {
                if let Some(pos) = held_or_clicked_at {
                    self.extend_stroke_in_progress(view.screen_to_world(pos));
                }
                if !response.is_pointer_button_down_on() {
                    self.finish_stroke_in_progress(tools);
                }
            }
            Tool::Eraser => {
                if let Some(pos) = held_or_clicked_at {
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
        });
    }

    fn scroll_with_wheel(&mut self, ui: &egui::Ui, response: &Response) {
        let view = StackedPageView::fitting_one_whole_page(
            response.rect,
            self.scroll_offset_in_world_units,
        );
        if response.contains_pointer() && view.zoom() > 0.0 {
            let scroll_on_screen = ui.input(|input| input.smooth_scroll_delta.y);
            self.scroll_offset_in_world_units -= view.screen_length_to_world(scroll_on_screen);
        }
        let last_page_top = self.scroll_offset_showing_top_of(self.board.last_page_index());
        self.scroll_offset_in_world_units =
            self.scroll_offset_in_world_units.clamp(0, last_page_top);
    }

    fn paint_visible_pages(
        &self,
        painter: &Painter,
        view: StackedPageView,
        area: Rect,
    ) -> Vec<Rect> {
        painter.rect_filled(area, 0.0, BACKGROUND_BEHIND_PAGES);
        (0..=self.board.last_page_index())
            .map(|page_index| {
                let page = view.world_area_to_screen(Aabb::page(page_index));
                let drawable = view.world_area_to_screen(Aabb::printable_part_of_page(page_index));
                (page, drawable)
            })
            .filter(|(page, _)| page.intersects(area))
            .map(|(page, drawable)| {
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
        view: StackedPageView,
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
                    view,
                    &stroke.points,
                    stroke.width_in_world_units,
                    color,
                );
            }
            paint_rounded_polyline(
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
        if !points.is_empty() {
            self.board.strokes.push(StandInStroke {
                unmultiplied_rgba: tools.color.to_srgba_unmultiplied(),
                width_in_world_units: tools.stroke_width_in_world_units(),
                points,
            });
        }
    }

    fn erase_strokes_touching(&mut self, view: StackedPageView, eraser_center: Pos2) {
        self.board
            .strokes
            .retain(|stroke| !eraser_touches(view, stroke, eraser_center));
    }

    fn add_page_at_end(&mut self) {
        let new_last_page_index = self.board.last_page_index().saturating_add(1);
        self.board.apply_add_page_idempotently(new_last_page_index);
        self.scroll_to_top_of(new_last_page_index);
    }

    fn scroll_to_top_of(&mut self, page_index: u16) {
        self.scroll_offset_in_world_units = self.scroll_offset_showing_top_of(page_index);
    }

    fn scroll_offset_showing_top_of(&self, page_index: u16) -> i64 {
        page_top_in_world_units(page_index.min(self.board.last_page_index()))
    }

    fn page_nearest_to_top_of_view(&self) -> u16 {
        let distance = distance_between_page_tops_in_world_units();
        let nearest = (self.scroll_offset_in_world_units + distance / 2) / distance;
        u16::try_from(nearest)
            .unwrap_or(u16::MAX)
            .min(self.board.last_page_index())
    }
}

fn eraser_touches(view: StackedPageView, stroke: &StandInStroke, eraser_center: Pos2) -> bool {
    let half_stroke_width = view.world_length_to_screen(stroke.width_in_world_units) / 2.0;
    let reach = ERASER_RADIUS_ON_SCREEN_PX + half_stroke_width;
    if let [dot] = stroke.points.as_slice() {
        return view.world_to_screen(*dot).distance(eraser_center) <= reach;
    }
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

fn paint_rounded_polyline(
    painter: &Painter,
    view: StackedPageView,
    points: &[WorldPos],
    width_in_world_units: u32,
    color: Color32,
) {
    let width = view.world_length_to_screen(width_in_world_units);
    let on_screen: Vec<Pos2> = points.iter().map(|&p| view.world_to_screen(p)).collect();
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
    use eframe::egui::Vec2;

    const FLOAT_TOLERANCE: f32 = 1e-6;
    const START: Pos2 = Pos2::new(0.0, 0.0);
    const END: Pos2 = Pos2::new(10.0, 0.0);
    const CANVAS_AREA: Rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 900.0));
    const SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS: i64 = 16_000;
    const DOT_POSITION: WorldPos = WorldPos {
        x: 4_000,
        y: 20_000,
    };
    const DOT_WIDTH_IN_WORLD_UNITS: u32 = 64;
    const DISTANCE_PAST_REACH_EDGE_PX: f32 = 0.5;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < FLOAT_TOLERANCE,
            "{actual} != {expected}"
        );
    }

    fn view_scrolled_into_second_page() -> StackedPageView {
        StackedPageView::fitting_one_whole_page(
            CANVAS_AREA,
            SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS,
        )
    }

    fn dot_and_eraser_reach_on_screen(view: StackedPageView) -> (StandInStroke, Pos2, f32) {
        let dot = StandInStroke {
            unmultiplied_rgba: Color32::BLACK.to_srgba_unmultiplied(),
            width_in_world_units: DOT_WIDTH_IN_WORLD_UNITS,
            points: vec![DOT_POSITION],
        };
        let reach = ERASER_RADIUS_ON_SCREEN_PX
            + view.world_length_to_screen(DOT_WIDTH_IN_WORLD_UNITS) / 2.0;
        (dot, view.world_to_screen(DOT_POSITION), reach)
    }

    #[test]
    fn eraser_touches_a_dot_just_inside_its_reach() {
        let view = view_scrolled_into_second_page();
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(view);
        let eraser_center = dot_on_screen + Vec2::new(reach - DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(eraser_touches(view, &dot, eraser_center));
    }

    #[test]
    fn eraser_misses_a_dot_just_outside_its_reach() {
        let view = view_scrolled_into_second_page();
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(view);
        let eraser_center = dot_on_screen + Vec2::new(reach + DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(!eraser_touches(view, &dot, eraser_center));
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
