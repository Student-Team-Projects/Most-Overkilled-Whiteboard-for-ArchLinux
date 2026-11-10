//! The controller: pointer input on the page, toolbar commands.

use egui::{Pos2, Rect, Vec2};
use wb_app::camera::{AffineCamera, Camera};
use wb_app::config::{PAGE, PRINTABLE_AREA, SPACE_AROUND_PAGE_PX};
use wb_app::controller::{CanvasInput, Command, Controller, SinglePageController};
use wb_app::local_board::LocalBoard;
use wb_app::model::{AppModel, Model};
use wb_app::tool::Tool;
use wb_core::{Board, WorldPos};

use crate::support::config::{AUTHOR, VIEWPORT};

fn canvas() -> Rect {
    Rect::from_min_size(Pos2::ZERO, VIEWPORT)
}

fn input(pointer_pressed_at: Option<Pos2>, is_pointer_held: bool) -> CanvasInput {
    CanvasInput {
        area: canvas(),
        pointer_pressed_at,
        is_pointer_held,
    }
}

fn on_screen(point: WorldPos) -> Pos2 {
    AffineCamera::fitting(PAGE, canvas(), SPACE_AROUND_PAGE_PX).world_to_screen(point)
}

fn strokes(model: &AppModel) -> usize {
    model.board().board().visible().count()
}

#[test]
fn pen_held_across_two_points_then_released_draws_one_stroke() {
    let mut model = AppModel::new(AUTHOR);
    let centre = canvas().center();
    SinglePageController.handle_canvas_input(&mut model, &input(Some(centre), true));
    SinglePageController.handle_canvas_input(
        &mut model,
        &input(Some(centre + Vec2::new(10.0, 0.0)), true),
    );
    assert_eq!(model.stroke_in_progress().len(), 2);
    assert_eq!(strokes(&model), 0);
    SinglePageController.handle_canvas_input(&mut model, &input(None, false));
    assert!(model.stroke_in_progress().is_empty());
    assert_eq!(strokes(&model), 1);
}

#[test]
fn a_held_pointer_that_does_not_move_adds_one_point() {
    let mut model = AppModel::new(AUTHOR);
    SinglePageController.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    SinglePageController.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    assert_eq!(model.stroke_in_progress().len(), 1);
}

#[test]
fn a_click_without_moving_draws_a_dot() {
    let mut model = AppModel::new(AUTHOR);
    SinglePageController.handle_canvas_input(&mut model, &input(Some(canvas().center()), false));
    assert_eq!(strokes(&model), 1);
}

#[test]
fn the_pen_draws_nothing_in_the_margin() {
    let mut model = AppModel::new(AUTHOR);
    let in_margin = on_screen(WorldPos::new(PAGE.min.x + 1, PRINTABLE_AREA.min.y + 1));
    SinglePageController.handle_canvas_input(&mut model, &input(Some(in_margin), false));
    assert_eq!(strokes(&model), 0);
}

#[test]
fn leaving_the_printable_part_ends_the_stroke_and_reentering_starts_a_new_one() {
    let mut model = AppModel::new(AUTHOR);
    let inside = on_screen(PRINTABLE_AREA.min) + Vec2::new(5.0, 5.0);
    let outside = on_screen(PAGE.min) + Vec2::new(1.0, 1.0);
    for pos in [inside, inside + Vec2::new(5.0, 0.0), outside, inside] {
        SinglePageController.handle_canvas_input(&mut model, &input(Some(pos), true));
    }
    SinglePageController.handle_canvas_input(&mut model, &input(None, false));
    assert_eq!(strokes(&model), 2);
}

#[test]
fn the_eraser_erases_the_stroke_under_the_pointer() {
    let mut model = AppModel::new(AUTHOR);
    SinglePageController.handle_canvas_input(&mut model, &input(Some(canvas().center()), false));
    SinglePageController.apply(&mut model, Command::SelectTool(Tool::Eraser));
    SinglePageController.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    assert_eq!(strokes(&model), 0);
}
