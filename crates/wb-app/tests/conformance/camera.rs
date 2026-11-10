//! The four guarantees of `docs/contracts/camera.md`, on random cameras.

use egui::{Pos2, Vec2};
use proptest::prelude::*;
use wb_app::camera::Camera;
use wb_app::config::{MAX_ZOOM, MIN_ZOOM};
use wb_core::{Aabb, WorldPos};

use crate::support::config::{
    ANCHOR_TOLERANCE_PX, MAX_ZOOM_STEPS, OFFSET_FROM_CENTRE, SCREEN_COORDINATE, VIEWPORT_SIDE,
    WORLD_CENTRE, ZOOM_FACTOR,
};

fn contains(area: Aabb, point: WorldPos) -> bool {
    (area.min.x..=area.max.x).contains(&point.x) && (area.min.y..=area.max.y).contains(&point.y)
}

fn any_camera() -> impl Strategy<Value = Camera> {
    (
        WORLD_CENTRE,
        WORLD_CENTRE,
        SCREEN_COORDINATE,
        SCREEN_COORDINATE,
        MIN_ZOOM..=MAX_ZOOM,
    )
        .prop_map(|(x, y, screen_x, screen_y, zoom)| {
            Camera::new(WorldPos::new(x, y), Pos2::new(screen_x, screen_y), zoom)
        })
}

proptest! {
    #[test]
    fn inverse_round_trip(
        camera in any_camera(),
        dx in OFFSET_FROM_CENTRE,
        dy in OFFSET_FROM_CENTRE,
    ) {
        let point = WorldPos::new(camera.world_centre.x + dx, camera.world_centre.y + dy);
        prop_assert_eq!(camera.screen_to_world(camera.world_to_screen(point)), point);
    }

    #[test]
    fn zoom_anchor_fixed(
        mut camera in any_camera(),
        anchor_x in SCREEN_COORDINATE,
        anchor_y in SCREEN_COORDINATE,
        factor in ZOOM_FACTOR,
    ) {
        let anchor = Pos2::new(anchor_x, anchor_y);
        let world_under_anchor = camera.screen_to_world(anchor);
        camera.zoom_at(anchor, factor);
        let distance = camera.world_to_screen(world_under_anchor).distance(anchor);
        prop_assert!(distance < ANCHOR_TOLERANCE_PX, "moved by {distance} px");
    }

    #[test]
    fn zoom_clamped(
        mut camera in any_camera(),
        factors in prop::collection::vec(ZOOM_FACTOR, 1..MAX_ZOOM_STEPS),
    ) {
        for factor in factors {
            camera.zoom_at(camera.screen_centre, factor);
            prop_assert!((MIN_ZOOM..=MAX_ZOOM).contains(&camera.zoom()));
        }
    }

    #[test]
    fn visible_area_covers_corners(
        camera in any_camera(),
        width in VIEWPORT_SIDE,
        height in VIEWPORT_SIDE,
        fraction_x in 0.0f32..=1.0,
        fraction_y in 0.0f32..=1.0,
    ) {
        let size = Vec2::new(width, height);
        let area = camera.visible_area(size);
        let top_left = camera.screen_centre - size / 2.0;
        for corner in [Vec2::ZERO, Vec2::new(width, 0.0), Vec2::new(0.0, height), size] {
            prop_assert!(contains(area, camera.screen_to_world(top_left + corner)));
        }
        let inside = top_left + Vec2::new(width * fraction_x, height * fraction_y);
        prop_assert!(contains(area, camera.screen_to_world(inside)));
    }
}
