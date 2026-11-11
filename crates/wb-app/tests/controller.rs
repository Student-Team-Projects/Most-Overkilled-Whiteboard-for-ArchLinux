//! The controller: pointer and scroll input on the page stack, toolbar commands.

use egui::{Pos2, Rect, Vec2};
use wb_app::camera::{AffineCamera, Camera};
use wb_app::config::SPACE_AROUND_PAGE_PX;
use wb_app::controller::{CanvasInput, Command, Controller, PageStackController};
use wb_app::local_board::LocalBoard;
use wb_app::model::Model;
use wb_app::pages::PageStack;
use wb_app::tool::Tool;
use wb_core::{Aabb, Board, Raw, StrokeCodec, WorldPos};

use crate::support::config::{AUTHOR, VIEWPORT};

fn page() -> Aabb {
    PageStack::page(0)
}

fn canvas() -> Rect {
    Rect::from_min_size(Pos2::ZERO, VIEWPORT)
}

fn input(pointer_pressed_at: Option<Pos2>, is_pointer_held: bool) -> CanvasInput {
    CanvasInput {
        area: canvas(),
        pointer_pressed_at,
        is_pointer_held,
        scroll_delta: Vec2::ZERO,
    }
}

fn on_screen(point: WorldPos) -> Pos2 {
    AffineCamera::fitting(page(), canvas(), SPACE_AROUND_PAGE_PX).world_to_screen(point)
}

fn strokes(model: &Model) -> usize {
    model.board.board().visible().count()
}

#[test]
fn pen_held_across_two_points_then_released_draws_one_stroke() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    let centre = canvas().center();
    controller.handle_canvas_input(&mut model, &input(Some(centre), true));
    controller.handle_canvas_input(
        &mut model,
        &input(Some(centre + Vec2::new(10.0, 0.0)), true),
    );
    assert_eq!(model.stroke_in_progress().len(), 2);
    assert_eq!(strokes(&model), 0);
    controller.handle_canvas_input(&mut model, &input(None, false));
    assert!(model.stroke_in_progress().is_empty());
    assert_eq!(strokes(&model), 1);
}

#[test]
fn a_held_pointer_that_does_not_move_adds_one_point() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    assert_eq!(model.stroke_in_progress().len(), 1);
}

#[test]
fn a_click_without_moving_draws_a_dot() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), false));
    assert_eq!(strokes(&model), 1);
}

#[test]
fn the_pen_draws_nothing_in_the_margin() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    let in_margin = on_screen(WorldPos::new(
        page().min.x + 1,
        PageStack::printable_area(0).min.y + 1,
    ));
    controller.handle_canvas_input(&mut model, &input(Some(in_margin), false));
    assert_eq!(strokes(&model), 0);
}

#[test]
fn leaving_the_printable_part_ends_the_stroke_and_reentering_starts_a_new_one() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    let inside = on_screen(PageStack::printable_area(0).min) + Vec2::new(5.0, 5.0);
    let outside = on_screen(page().min) + Vec2::new(1.0, 1.0);
    for pos in [inside, inside + Vec2::new(5.0, 0.0), outside, inside] {
        controller.handle_canvas_input(&mut model, &input(Some(pos), true));
    }
    controller.handle_canvas_input(&mut model, &input(None, false));
    assert_eq!(strokes(&model), 2);
}

#[test]
fn the_eraser_erases_the_stroke_under_the_pointer() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), false));
    controller.apply(&mut model, Command::SelectTool(Tool::Eraser));
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), true));
    assert_eq!(strokes(&model), 0);
}

#[test]
fn leaving_the_printable_part_ends_the_stroke_on_its_edge() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    let row = PageStack::printable_area(0)
        .min
        .y
        .midpoint(PageStack::printable_area(0).max.y);
    let inside = on_screen(WorldPos::new(
        PageStack::printable_area(0)
            .min
            .x
            .midpoint(PageStack::printable_area(0).max.x),
        row,
    ));
    let outside = on_screen(WorldPos::new(page().max.x, row));
    for pos in [inside, outside] {
        controller.handle_canvas_input(&mut model, &input(Some(pos), true));
    }
    let stroke = model.board.board().visible().next().unwrap();
    let last = *Vec::from(&Raw.decode(&stroke.points).unwrap())
        .last()
        .unwrap();
    assert_eq!(last.x, PageStack::printable_area(0).max.x);
}

#[test]
fn coming_back_onto_the_printable_part_starts_the_stroke_on_its_edge() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    let row = PageStack::printable_area(0)
        .min
        .y
        .midpoint(PageStack::printable_area(0).max.y);
    let outside = on_screen(WorldPos::new(page().max.x, row));
    let inside = on_screen(WorldPos::new(
        PageStack::printable_area(0)
            .min
            .x
            .midpoint(PageStack::printable_area(0).max.x),
        row,
    ));
    for pos in [outside, inside] {
        controller.handle_canvas_input(&mut model, &input(Some(pos), true));
    }
    controller.handle_canvas_input(&mut model, &input(None, false));
    let stroke = model.board.board().visible().next().unwrap();
    let first = Vec::from(&Raw.decode(&stroke.points).unwrap())[0];
    assert_eq!(first.x, PageStack::printable_area(0).max.x);
}

fn first_and_last_points(model: &Model) -> Vec<(WorldPos, WorldPos)> {
    model
        .board
        .board()
        .visible()
        .filter_map(|stroke| {
            let points = Vec::from(&Raw.decode(&stroke.points).ok()?);
            Some((*points.first()?, *points.last()?))
        })
        .collect()
}

fn scroll(delta_y: f32) -> CanvasInput {
    CanvasInput {
        scroll_delta: Vec2::new(0.0, delta_y),
        ..input(None, false)
    }
}

#[test]
fn adding_a_page_appends_it_and_keeps_the_view() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.apply(&mut model, Command::AddPage);
    controller.handle_canvas_input(&mut model, &input(None, false));
    assert_eq!(model.pages.count(), 2);
    assert_eq!(model.camera.world_centre, PageStack::centre(0));
}

#[test]
fn scrolling_stays_between_the_first_and_the_last_page() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.handle_canvas_input(&mut model, &scroll(-VIEWPORT.y));
    assert_eq!(model.camera.world_centre, PageStack::centre(0));
    controller.apply(&mut model, Command::AddPage);
    controller.handle_canvas_input(&mut model, &scroll(-VIEWPORT.y));
    assert!(model.camera.world_centre.y > PageStack::centre(0).y);
    for _ in 0..model.pages.count() * 4 {
        controller.handle_canvas_input(&mut model, &scroll(-VIEWPORT.y));
    }
    assert_eq!(model.camera.world_centre, PageStack::centre(1));
    for _ in 0..model.pages.count() * 4 {
        controller.handle_canvas_input(&mut model, &scroll(VIEWPORT.y));
    }
    assert_eq!(model.camera.world_centre, PageStack::centre(0));
}

#[test]
fn the_pen_draws_on_an_added_page() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.apply(&mut model, Command::AddPage);
    for _ in 0..4 {
        controller.handle_canvas_input(&mut model, &scroll(-VIEWPORT.y));
    }
    controller.handle_canvas_input(&mut model, &input(Some(canvas().center()), false));
    let [(dot, _)] = first_and_last_points(&model)[..] else {
        panic!("expected one stroke");
    };
    assert_eq!(model.pages.page_at(dot), Some(1));
}

#[test]
fn a_stroke_dragged_across_the_gap_becomes_one_stroke_per_page() {
    let mut model = Model::new(AUTHOR);
    let mut controller = PageStackController::default();
    controller.apply(&mut model, Command::AddPage);
    controller.handle_canvas_input(&mut model, &input(None, false));
    let column = PageStack::centre(0).x;
    let near_bottom_of_first = WorldPos::new(column, PageStack::printable_area(0).max.y - 1);
    let near_top_of_second = WorldPos::new(column, PageStack::printable_area(1).min.y + 1);
    for point in [near_bottom_of_first, near_top_of_second] {
        let pos = model.camera.world_to_screen(point);
        controller.handle_canvas_input(&mut model, &input(Some(pos), true));
    }
    controller.handle_canvas_input(&mut model, &input(None, false));
    let strokes = first_and_last_points(&model);
    assert_eq!(strokes.len(), 2);
    assert_eq!(strokes[0].1.y, PageStack::printable_area(0).max.y);
    assert_eq!(strokes[1].0.y, PageStack::printable_area(1).min.y);
}
