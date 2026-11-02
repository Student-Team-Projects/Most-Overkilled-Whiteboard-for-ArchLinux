use eframe::egui::{Pos2, Vec2};

use crate::stand_in::{Aabb, WORLD_UNITS_PER_PIXEL, WorldPos};

pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 2.0;
const SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub world_centre: WorldPos,
    pub screen_centre: Pos2,
    zoom: f32,
}

impl Camera {
    pub fn new(world_centre: WorldPos, screen_centre: Pos2, zoom: f32) -> Self {
        let mut camera = Self {
            world_centre,
            screen_centre,
            zoom: MIN_ZOOM,
        };
        camera.set_zoom_clamped(zoom);
        camera
    }

    pub fn zoom(self) -> f32 {
        self.zoom
    }

    pub fn world_to_screen(self, point: WorldPos) -> Pos2 {
        let offset_in_world_units = Vec2::new(
            world_units_to_f32(point.x.saturating_sub(self.world_centre.x)),
            world_units_to_f32(point.y.saturating_sub(self.world_centre.y)),
        );
        self.screen_centre + self.world_units_to_screen(offset_in_world_units)
    }

    pub fn screen_to_world(self, pos: Pos2) -> WorldPos {
        let offset = self.screen_to_world_units(pos - self.screen_centre);
        WorldPos {
            x: self
                .world_centre
                .x
                .saturating_add(round_to_world_units(offset.x)),
            y: self
                .world_centre
                .y
                .saturating_add(round_to_world_units(offset.y)),
        }
    }

    pub fn world_length_to_screen(self, length_in_world_units: u32) -> f32 {
        let length = world_units_to_f32(i64::from(length_in_world_units));
        self.world_units_to_screen(Vec2::new(length, 0.0)).x
    }

    pub fn pan(&mut self, screen_delta: Vec2) {
        let delta = self.screen_to_world_units(screen_delta);
        self.world_centre = WorldPos {
            x: self
                .world_centre
                .x
                .saturating_sub(round_to_world_units(delta.x)),
            y: self
                .world_centre
                .y
                .saturating_sub(round_to_world_units(delta.y)),
        };
    }

    pub fn zoom_at(&mut self, anchor: Pos2, factor: f32) {
        let world_under_anchor = self.screen_to_world(anchor);
        self.set_zoom_clamped(self.zoom * factor);
        self.pan(anchor - self.world_to_screen(world_under_anchor));
    }

    pub fn visible_area(self, viewport_size: Vec2) -> Aabb {
        let half = self.screen_to_world_units(viewport_size / 2.0);
        let half_x = round_to_world_units(half.x.ceil()) + SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS;
        let half_y = round_to_world_units(half.y.ceil()) + SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS;
        Aabb {
            min: WorldPos {
                x: self.world_centre.x.saturating_sub(half_x),
                y: self.world_centre.y.saturating_sub(half_y),
            },
            max: WorldPos {
                x: self.world_centre.x.saturating_add(half_x),
                y: self.world_centre.y.saturating_add(half_y),
            },
        }
    }

    fn set_zoom_clamped(&mut self, zoom: f32) {
        if zoom.is_finite() {
            self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        }
    }

    fn world_units_to_screen(self, units: Vec2) -> Vec2 {
        units / f32::from(WORLD_UNITS_PER_PIXEL) * self.zoom
    }

    fn screen_to_world_units(self, screen: Vec2) -> Vec2 {
        screen / self.zoom * f32::from(WORLD_UNITS_PER_PIXEL)
    }
}

#[allow(clippy::cast_possible_truncation)] // `as` saturates; screen-sized offsets are far inside i64
fn round_to_world_units(units: f32) -> i64 {
    units.round() as i64
}

#[allow(clippy::cast_precision_loss)] // offsets from the camera centre that reach the screen are exact
fn world_units_to_f32(units: i64) -> f32 {
    units as f32
}

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;
    use proptest::prelude::*;

    const WORLD_CENTRE_RANGE: std::ops::Range<i64> = -(1 << 40)..(1 << 40);
    const OFFSET_FROM_CENTRE_RANGE: std::ops::Range<i64> = -1_000_000..1_000_000;
    const SCREEN_COORDINATE_RANGE: std::ops::Range<f32> = 0.0..2_000.0;
    const VIEWPORT_SIDE_RANGE: std::ops::Range<f32> = 1.0..4_000.0;
    const ZOOM_FACTOR_RANGE: std::ops::Range<f32> = 0.001..1_000.0;
    const ANCHOR_TOLERANCE_PX: f32 = 0.1;
    const STARTING_ZOOM: f32 = 1.0;

    fn area_contains(area: Aabb, point: WorldPos) -> bool {
        (area.min.x..=area.max.x).contains(&point.x) && (area.min.y..=area.max.y).contains(&point.y)
    }

    fn any_camera() -> impl Strategy<Value = Camera> {
        (
            WORLD_CENTRE_RANGE,
            WORLD_CENTRE_RANGE,
            SCREEN_COORDINATE_RANGE,
            SCREEN_COORDINATE_RANGE,
            MIN_ZOOM..=MAX_ZOOM,
        )
            .prop_map(|(x, y, screen_x, screen_y, zoom)| {
                Camera::new(WorldPos { x, y }, Pos2::new(screen_x, screen_y), zoom)
            })
    }

    proptest! {
        #[test]
        fn inverse_round_trip(
            camera in any_camera(),
            dx in OFFSET_FROM_CENTRE_RANGE,
            dy in OFFSET_FROM_CENTRE_RANGE,
        ) {
            let point = WorldPos {
                x: camera.world_centre.x + dx,
                y: camera.world_centre.y + dy,
            };
            prop_assert_eq!(camera.screen_to_world(camera.world_to_screen(point)), point);
        }

        #[test]
        fn zoom_anchor_fixed(
            mut camera in any_camera(),
            anchor_x in SCREEN_COORDINATE_RANGE,
            anchor_y in SCREEN_COORDINATE_RANGE,
            factor in ZOOM_FACTOR_RANGE,
        ) {
            let anchor = Pos2::new(anchor_x, anchor_y);
            let world_under_anchor = camera.screen_to_world(anchor);
            camera.zoom_at(anchor, factor);
            let distance = camera.world_to_screen(world_under_anchor).distance(anchor);
            prop_assert!(distance < ANCHOR_TOLERANCE_PX, "moved by {distance} px");
        }

        #[test]
        fn zoom_clamped(mut camera in any_camera(), factors in prop::collection::vec(ZOOM_FACTOR_RANGE, 1..8)) {
            for factor in factors {
                camera.zoom_at(camera.screen_centre, factor);
                prop_assert!((MIN_ZOOM..=MAX_ZOOM).contains(&camera.zoom()));
            }
        }

        #[test]
        fn visible_area_covers_corners(
            camera in any_camera(),
            width in VIEWPORT_SIDE_RANGE,
            height in VIEWPORT_SIDE_RANGE,
            fraction_x in 0.0f32..=1.0,
            fraction_y in 0.0f32..=1.0,
        ) {
            let size = Vec2::new(width, height);
            let area = camera.visible_area(size);
            let top_left = camera.screen_centre - size / 2.0;
            for corner in [Vec2::ZERO, Vec2::new(width, 0.0), Vec2::new(0.0, height), size] {
                prop_assert!(area_contains(area, camera.screen_to_world(top_left + corner)));
            }
            let inside = top_left + Vec2::new(width * fraction_x, height * fraction_y);
            prop_assert!(area_contains(area, camera.screen_to_world(inside)));
        }
    }

    #[test]
    fn non_finite_zoom_factor_leaves_zoom_unchanged() {
        let mut camera = Camera::new(WorldPos { x: 0, y: 0 }, Pos2::ZERO, STARTING_ZOOM);
        camera.zoom_at(Pos2::ZERO, f32::NAN);
        assert!((camera.zoom() - STARTING_ZOOM).abs() < f32::EPSILON);
    }
}
