//! Turns what the user did into changes of the model. The only writer of the model.

use eframe::egui::{Pos2, Rect};

use crate::camera::StackedPageCamera;
use crate::config::ERASER_RADIUS_ON_SCREEN_PX;
use crate::model::Model;
use crate::stand_in::{Rgba, StandInStroke};
use crate::tool::Tool;

/// What the user asked for with a toolbar or page bar button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Switch to this tool.
    SelectTool(Tool),
    /// Draw the next strokes in this colour.
    SelectColor(Rgba),
    /// Draw the next strokes this wide, in page pixels.
    SelectStrokeWidth(u16),
    /// Scroll to the top of this page.
    ScrollToTopOf(u32),
    /// Add a page after the last one.
    AddPageAtEnd,
}

/// What the pointer did on the canvas during one frame, in screen coordinates.
#[derive(Clone, Copy, Debug)]
pub struct CanvasInput {
    /// The canvas rectangle on screen.
    pub area: Rect,
    /// Wheel movement in screen pixels while the pointer is over the canvas;
    /// positive scrolls towards the first page.
    pub scroll_on_screen: f32,
    /// Where the primary button is held or was just clicked; `None` otherwise.
    pub pointer_pressed_at: Option<Pos2>,
    /// Whether the primary button is still held on the canvas.
    pub is_pointer_held: bool,
}

/// Changes `model` as `command` asks.
pub fn apply(model: &mut Model, command: Command) {
    match command {
        Command::SelectTool(tool) => model.tools.tool = tool,
        Command::SelectColor(color) => model.tools.color = color,
        Command::SelectStrokeWidth(width_px) => model.tools.stroke_width_px = width_px,
        Command::ScrollToTopOf(page_index) => model.scroll_to_top_of(page_index),
        Command::AddPageAtEnd => model.add_page_at_end(),
    }
}

/// Scrolls, then lets the current tool act on the printable part of the pages:
/// the pen extends the stroke in progress and finishes it once the button
/// is released or the pointer leaves the printable part, so re-entering starts
/// a new stroke; the eraser removes every stroke within its radius.
pub fn handle_canvas_input(model: &mut Model, input: &CanvasInput) {
    scroll(model, input);
    let camera =
        StackedPageCamera::fitting_one_whole_page(input.area, model.scroll_offset_in_world_units());
    let pressed_on_printable_part = input
        .pointer_pressed_at
        .filter(|&pos| model.is_printable(camera.screen_to_world(pos)));

    match model.tools.tool {
        Tool::Pen => {
            let printable_point = pressed_on_printable_part.map(|pos| camera.screen_to_world(pos));
            if let Some(point) = printable_point {
                model.extend_stroke_in_progress(point);
            }
            if !input.is_pointer_held || printable_point.is_none() {
                model.finish_stroke_in_progress();
            }
        }
        Tool::Eraser => {
            if let Some(eraser_center) = pressed_on_printable_part {
                model.erase_strokes_where(|stroke| eraser_touches(camera, stroke, eraser_center));
            }
        }
    }
}

fn scroll(model: &mut Model, input: &CanvasInput) {
    let camera =
        StackedPageCamera::fitting_one_whole_page(input.area, model.scroll_offset_in_world_units());
    let scroll_in_world_units = if camera.zoom() > 0.0 {
        camera.screen_length_to_world(input.scroll_on_screen)
    } else {
        0
    };
    model.scroll_by(-scroll_in_world_units);
}

fn eraser_touches(camera: StackedPageCamera, stroke: &StandInStroke, eraser_center: Pos2) -> bool {
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

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;
    use crate::config::DEFAULT_STROKE_COLOR;
    use crate::page_layout::page_top_in_world_units;
    use crate::stand_in::WorldPos;
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
    const PRINTABLE_POINTS_ON_FIRST_PAGE: [WorldPos; 2] = [
        WorldPos { x: 1_000, y: 1_000 },
        WorldPos { x: 2_000, y: 1_500 },
    ];
    const POINT_IN_LEFT_MARGIN_OF_FIRST_PAGE: WorldPos = WorldPos { x: 100, y: 1_200 };
    const POINT_THREE_ABOVE_THE_SEGMENT_MIDDLE: Pos2 = Pos2::new(5.0, 3.0);
    const POINT_THREE_RIGHT_FOUR_ABOVE_THE_END: Pos2 = Pos2::new(13.0, 4.0);
    const POINT_THREE_RIGHT_FOUR_ABOVE_THE_START: Pos2 = Pos2::new(3.0, 4.0);
    const THREE_STRAIGHT_ACROSS: f32 = 3.0;
    const HYPOTENUSE_OF_THREE_AND_FOUR: f32 = 5.0;
    const AREA_TOO_SMALL_FOR_A_PAGE: Rect =
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(20.0, 20.0));
    const WHEEL_SCROLL_TOWARDS_FIRST_PAGE_PX: f32 = 50.0;
    const CENTER_OF_TOO_SMALL_AREA: Pos2 = Pos2::new(10.0, 10.0);

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < FLOAT_TOLERANCE,
            "{actual} != {expected}"
        );
    }

    fn camera_scrolled_into_second_page() -> StackedPageCamera {
        StackedPageCamera::fitting_one_whole_page(
            CANVAS_AREA,
            SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS,
        )
    }

    fn dot_and_eraser_reach_on_screen(camera: StackedPageCamera) -> (StandInStroke, Pos2, f32) {
        let dot = StandInStroke {
            color: DEFAULT_STROKE_COLOR,
            width_in_world_units: DOT_WIDTH_IN_WORLD_UNITS,
            points: vec![DOT_POSITION],
        };
        let reach = ERASER_RADIUS_ON_SCREEN_PX
            + camera.world_length_to_screen(DOT_WIDTH_IN_WORLD_UNITS) / 2.0;
        (dot, camera.world_to_screen(DOT_POSITION), reach)
    }

    fn pointer_input(pointer_pressed_at: Option<Pos2>, is_pointer_held: bool) -> CanvasInput {
        CanvasInput {
            area: CANVAS_AREA,
            scroll_on_screen: 0.0,
            pointer_pressed_at,
            is_pointer_held,
        }
    }

    #[test]
    fn pen_held_across_two_points_then_released_draws_one_stroke() {
        let mut model = Model::default();
        let camera = StackedPageCamera::fitting_one_whole_page(CANVAS_AREA, 0);
        for point in PRINTABLE_POINTS_ON_FIRST_PAGE {
            let held_at = Some(camera.world_to_screen(point));
            handle_canvas_input(&mut model, &pointer_input(held_at, true));
        }
        handle_canvas_input(&mut model, &pointer_input(None, false));

        assert_eq!(model.board().strokes.len(), 1);
        assert_eq!(
            model.board().strokes[0].points,
            PRINTABLE_POINTS_ON_FIRST_PAGE
        );
        assert!(model.stroke_in_progress().is_empty());
    }

    #[test]
    fn pen_leaving_the_printable_part_ends_the_stroke_and_reentering_starts_a_new_one() {
        let mut model = Model::default();
        let camera = StackedPageCamera::fitting_one_whole_page(CANVAS_AREA, 0);
        let [inside_before, inside_after] = PRINTABLE_POINTS_ON_FIRST_PAGE;
        for point in [
            inside_before,
            POINT_IN_LEFT_MARGIN_OF_FIRST_PAGE,
            inside_after,
        ] {
            let held_at = Some(camera.world_to_screen(point));
            handle_canvas_input(&mut model, &pointer_input(held_at, true));
        }
        handle_canvas_input(&mut model, &pointer_input(None, false));

        let strokes: Vec<Vec<WorldPos>> = model
            .board()
            .strokes
            .iter()
            .map(|stroke| stroke.points.clone())
            .collect();
        assert_eq!(strokes, [vec![inside_before], vec![inside_after]]);
    }

    #[test]
    fn eraser_touches_a_dot_just_inside_its_reach() {
        let camera = camera_scrolled_into_second_page();
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(camera);
        let eraser_center = dot_on_screen + Vec2::new(reach - DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(eraser_touches(camera, &dot, eraser_center));
    }

    #[test]
    fn eraser_misses_a_dot_just_outside_its_reach() {
        let camera = camera_scrolled_into_second_page();
        let (dot, dot_on_screen, reach) = dot_and_eraser_reach_on_screen(camera);
        let eraser_center = dot_on_screen + Vec2::new(reach + DISTANCE_PAST_REACH_EDGE_PX, 0.0);
        assert!(!eraser_touches(camera, &dot, eraser_center));
    }

    #[test]
    fn point_beside_the_segment_measures_straight_across() {
        let distance =
            distance_from_point_to_segment(POINT_THREE_ABOVE_THE_SEGMENT_MIDDLE, START, END);
        assert_close(distance, THREE_STRAIGHT_ACROSS);
    }

    #[test]
    fn point_past_the_end_measures_to_the_end() {
        let distance =
            distance_from_point_to_segment(POINT_THREE_RIGHT_FOUR_ABOVE_THE_END, START, END);
        assert_close(distance, HYPOTENUSE_OF_THREE_AND_FOUR);
    }

    #[test]
    fn zero_length_segment_measures_to_its_only_point() {
        let distance =
            distance_from_point_to_segment(POINT_THREE_RIGHT_FOUR_ABOVE_THE_START, START, START);
        assert_close(distance, HYPOTENUSE_OF_THREE_AND_FOUR);
    }

    #[test]
    fn pen_in_an_area_too_small_for_a_page_draws_nothing() {
        let mut model = Model::default();
        let pressed = CanvasInput {
            area: AREA_TOO_SMALL_FOR_A_PAGE,
            scroll_on_screen: 0.0,
            pointer_pressed_at: Some(CENTER_OF_TOO_SMALL_AREA),
            is_pointer_held: true,
        };
        handle_canvas_input(&mut model, &pressed);
        assert!(model.stroke_in_progress().is_empty());
        handle_canvas_input(
            &mut model,
            &CanvasInput {
                pointer_pressed_at: None,
                is_pointer_held: false,
                ..pressed
            },
        );
        assert!(model.board().strokes.is_empty());
    }

    #[test]
    fn wheel_in_an_area_too_small_for_a_page_does_not_scroll() {
        let mut model = Model::default();
        model.add_page_at_end();
        let wheel = CanvasInput {
            area: AREA_TOO_SMALL_FOR_A_PAGE,
            scroll_on_screen: WHEEL_SCROLL_TOWARDS_FIRST_PAGE_PX,
            pointer_pressed_at: None,
            is_pointer_held: false,
        };
        handle_canvas_input(&mut model, &wheel);
        assert_eq!(
            model.scroll_offset_in_world_units(),
            page_top_in_world_units(1)
        );
    }
}
