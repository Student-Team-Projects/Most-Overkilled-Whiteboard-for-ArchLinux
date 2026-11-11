//! The camera that pans and zooms: plain `i64` world coordinates, one affine transform.

use egui::{Pos2, Rect, Vec2};
use wb_core::{Aabb, WorldPos};

use super::Camera;
use crate::config::{MAX_ZOOM, MIN_ZOOM, WORLD_UNITS_PER_PIXEL};

/// Shows the world point `world_centre` at the screen point `screen_centre`, scaled by a zoom
/// kept in `[MIN_ZOOM, MAX_ZOOM]`. At zoom 1 one screen pixel is `WORLD_UNITS_PER_PIXEL` world
/// units.
///
/// World points are taken relative to `world_centre` in `i64` before any float math, so
/// precision on screen is the same everywhere in the world.
///
/// See also: `docs/contracts/camera.md`, ADR 0013, ADR 0014.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineCamera {
    /// World point shown at `screen_centre`.
    pub world_centre: WorldPos,
    /// Screen point where `world_centre` appears, normally the centre of the canvas.
    pub screen_centre: Pos2,
    zoom: f32,
}

/// Room for float rounding when `visible_area` turns the viewport into world units.
const SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS: i64 = 1;

impl AffineCamera {
    /// A camera showing `world_centre` at `screen_centre`, with `zoom` clamped to the allowed
    /// range; a non-finite `zoom` gives `MIN_ZOOM`.
    #[must_use]
    pub fn new(world_centre: WorldPos, screen_centre: Pos2, zoom: f32) -> Self {
        let mut camera = Self {
            world_centre,
            screen_centre,
            zoom: MIN_ZOOM,
        };
        camera.set_zoom_clamped(zoom);
        camera
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

impl Camera for AffineCamera {
    fn zoom(&self) -> f32 {
        self.zoom
    }

    fn world_to_screen(&self, point: WorldPos) -> Pos2 {
        let offset_in_world_units = Vec2::new(
            world_units_to_f32(point.x.saturating_sub(self.world_centre.x)),
            world_units_to_f32(point.y.saturating_sub(self.world_centre.y)),
        );
        self.screen_centre + self.world_units_to_screen(offset_in_world_units)
    }

    fn screen_to_world(&self, pos: Pos2) -> WorldPos {
        let offset = self.screen_to_world_units(pos - self.screen_centre);
        WorldPos::new(
            self.world_centre
                .x
                .saturating_add(round_to_world_units(offset.x)),
            self.world_centre
                .y
                .saturating_add(round_to_world_units(offset.y)),
        )
    }

    fn world_length_to_screen(&self, length_in_world_units: u32) -> f32 {
        let length = world_units_to_f32(i64::from(length_in_world_units));
        self.world_units_to_screen(Vec2::new(length, 0.0)).x
    }

    fn screen_length_to_world(&self, length_on_screen: f32) -> u32 {
        let length = round_to_world_units(
            self.screen_to_world_units(Vec2::new(length_on_screen, 0.0))
                .x,
        );
        u32::try_from(length.max(0)).unwrap_or(u32::MAX)
    }

    fn pan(&mut self, screen_delta: Vec2) {
        let delta = self.screen_to_world_units(screen_delta);
        self.world_centre = WorldPos::new(
            self.world_centre
                .x
                .saturating_sub(round_to_world_units(delta.x)),
            self.world_centre
                .y
                .saturating_sub(round_to_world_units(delta.y)),
        );
    }

    fn zoom_at(&mut self, anchor: Pos2, factor: f32) {
        let world_under_anchor = self.screen_to_world(anchor);
        self.set_zoom_clamped(self.zoom * factor);
        self.pan(anchor - self.world_to_screen(world_under_anchor));
    }

    fn visible_area(&self, viewport_size: Vec2) -> Aabb {
        let half = self.screen_to_world_units(viewport_size / 2.0);
        let half_x = round_to_world_units(half.x.ceil()) + SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS;
        let half_y = round_to_world_units(half.y.ceil()) + SLACK_FOR_FLOAT_ROUNDING_IN_WORLD_UNITS;
        Aabb::new(
            WorldPos::new(
                self.world_centre.x.saturating_sub(half_x),
                self.world_centre.y.saturating_sub(half_y),
            ),
            WorldPos::new(
                self.world_centre.x.saturating_add(half_x),
                self.world_centre.y.saturating_add(half_y),
            ),
        )
    }

    fn fitting(area: Aabb, viewport: Rect, space_around_px: f32) -> Self {
        let world_size = Vec2::new(
            world_units_to_f32(area.max.x.saturating_sub(area.min.x)),
            world_units_to_f32(area.max.y.saturating_sub(area.min.y)),
        ) / f32::from(WORLD_UNITS_PER_PIXEL);
        let room = viewport.size() - Vec2::splat(2.0 * space_around_px);
        let zoom = (room.x / world_size.x).min(room.y / world_size.y);
        let centre = WorldPos::new(
            area.min.x.midpoint(area.max.x),
            area.min.y.midpoint(area.max.y),
        );
        Self::new(centre, viewport.center(), zoom)
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
