//! `Camera` behaviour beyond the contract suite.

use egui::{Pos2, Rect};
use wb_app::camera::{AffineCamera, Camera};
use wb_app::config::SPACE_AROUND_PAGE_PX;
use wb_core::Page;
use wb_core::WorldPos;

use crate::support::config::{ANCHOR_TOLERANCE_PX, ONE_WORLD_PIXEL, OPENING_ZOOM, VIEWPORT};

#[test]
fn non_finite_zoom_factor_leaves_zoom_unchanged() {
    let mut camera = AffineCamera::new(WorldPos::new(0, 0), Pos2::ZERO, OPENING_ZOOM);
    camera.zoom_at(Pos2::ZERO, f32::NAN);
    camera.zoom_at(Pos2::ZERO, f32::INFINITY);
    assert!((camera.zoom() - OPENING_ZOOM).abs() < f32::EPSILON);
}

#[test]
fn one_world_pixel_is_zoom_screen_pixels_and_back() {
    let camera = AffineCamera::new(WorldPos::new(0, 0), Pos2::ZERO, 2.0);
    assert!((camera.world_length_to_screen(ONE_WORLD_PIXEL) - 2.0).abs() < f32::EPSILON);
    assert_eq!(camera.screen_length_to_world(2.0), ONE_WORLD_PIXEL);
}

#[test]
fn round_trip_is_exact_at_the_edge_of_the_world() {
    let far = WorldPos::new(i64::MAX - 1_000, i64::MIN + 1_000);
    let camera = AffineCamera::new(far, Pos2::new(400.0, 300.0), OPENING_ZOOM);
    let nearby = WorldPos::new(far.x - 160, far.y + 320);
    assert_eq!(
        camera.screen_to_world(camera.world_to_screen(nearby)),
        nearby
    );
}

#[test]
fn fitting_shows_the_whole_area_centred_with_space_around() {
    let viewport = Rect::from_min_size(Pos2::ZERO, VIEWPORT);
    let camera = AffineCamera::fitting(Page(0).bounds(), viewport, SPACE_AROUND_PAGE_PX);
    let page = camera.world_area_to_screen(Page(0).bounds());
    assert!(
        viewport
            .shrink(SPACE_AROUND_PAGE_PX - 0.01)
            .contains_rect(page)
    );
    assert!((page.center() - viewport.center()).length() < ANCHOR_TOLERANCE_PX);
    let tight = (page.height() - (VIEWPORT.y - 2.0 * SPACE_AROUND_PAGE_PX)).abs();
    assert!(
        tight < ANCHOR_TOLERANCE_PX,
        "the page should fill the height"
    );
}
