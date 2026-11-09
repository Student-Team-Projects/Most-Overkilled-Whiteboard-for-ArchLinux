//! Screen ↔ world conversion.

use eframe::egui::{Pos2, Rect, Vec2};

use crate::config::{SPACE_AROUND_PAGE_PX, WORLD_UNITS_PER_PIXEL};
use crate::stand_in::{Aabb, WorldPos};

/// Converts between screen positions (egui points) and world positions
/// for a vertical stack of pages scrolled up and down.
///
/// The only place that knows the zoom and the 16 world units per pixel.
/// Positions are taken relative to the view top in `i64` before any float
/// math, so they stay exact on every page of the stack.
/// Stands in for `Camera` (T11), which adds pan, zoom and its [0.5, 2] clamp.
///
/// See also: ADR 0014.
#[derive(Clone, Copy, Debug)]
pub struct StackedPageCamera {
    view_top_left_on_screen: Pos2,
    scroll_offset_in_world_units: i64,
    zoom: f32,
}

impl StackedPageCamera {
    /// Zooms so one whole page fits inside `area` with a margin around it,
    /// centres the stack horizontally and scrolls it up by `scroll_offset_in_world_units`.
    ///
    /// The zoom is 0 when `area` is no larger than the margins; the screen-to-world
    /// conversions then return meaningless values.
    pub fn fitting_one_whole_page(area: Rect, scroll_offset_in_world_units: i64) -> Self {
        let page_at_zoom_one = world_size_at_zoom_one(Aabb::page(0));
        let room = (area.size() - Vec2::splat(2.0 * SPACE_AROUND_PAGE_PX)).max(Vec2::ZERO);
        let zoom = (room.x / page_at_zoom_one.x).min(room.y / page_at_zoom_one.y);
        let left = area.center().x - page_at_zoom_one.x * zoom / 2.0;
        let top = area.top() + SPACE_AROUND_PAGE_PX;
        Self {
            view_top_left_on_screen: Pos2::new(left, top),
            scroll_offset_in_world_units,
            zoom,
        }
    }

    /// Screen pixels per page pixel.
    pub fn zoom(self) -> f32 {
        self.zoom
    }

    /// Screen position of a world point.
    pub fn world_to_screen(self, point: WorldPos) -> Pos2 {
        let below_view_top = point.y.saturating_sub(self.scroll_offset_in_world_units);
        let world_units = Vec2::new(
            world_units_to_f32(point.x),
            world_units_to_f32(below_view_top),
        );
        self.view_top_left_on_screen + self.world_units_to_screen(world_units)
    }

    /// World point under a screen position, rounded to the nearest whole unit.
    /// Requires a zoom above 0.
    pub fn screen_to_world(self, pos: Pos2) -> WorldPos {
        let world_units = self.screen_to_world_units(pos - self.view_top_left_on_screen);
        WorldPos {
            x: round_to_world_units(world_units.x),
            y: round_to_world_units(world_units.y)
                .saturating_add(self.scroll_offset_in_world_units),
        }
    }

    /// On-screen length, in screen pixels, of a world length such as a stroke width.
    pub fn world_length_to_screen(self, length_in_world_units: u32) -> f32 {
        let length = world_units_to_f32(i64::from(length_in_world_units));
        self.world_units_to_screen(Vec2::new(length, 0.0)).x
    }

    /// World length of an on-screen length, rounded to the nearest whole unit.
    /// Keeps the sign. Requires a zoom above 0.
    pub fn screen_length_to_world(self, length_on_screen: f32) -> i64 {
        round_to_world_units(
            self.screen_to_world_units(Vec2::new(length_on_screen, 0.0))
                .x,
        )
    }

    /// Screen rectangle covering a world rectangle.
    pub fn world_area_to_screen(self, area: Aabb) -> Rect {
        Rect::from_min_max(
            self.world_to_screen(area.min),
            self.world_to_screen(area.max),
        )
    }

    fn world_units_to_screen(self, units: Vec2) -> Vec2 {
        units / f32::from(WORLD_UNITS_PER_PIXEL) * self.zoom
    }

    fn screen_to_world_units(self, screen: Vec2) -> Vec2 {
        screen / self.zoom * f32::from(WORLD_UNITS_PER_PIXEL)
    }
}

fn world_size_at_zoom_one(area: Aabb) -> Vec2 {
    let size_in_world_units = Vec2::new(
        world_units_to_f32(area.max.x - area.min.x),
        world_units_to_f32(area.max.y - area.min.y),
    );
    size_in_world_units / f32::from(WORLD_UNITS_PER_PIXEL)
}

#[allow(clippy::cast_possible_truncation)] // `as` saturates; on-screen positions × 16 fit in i64
fn round_to_world_units(units: f32) -> i64 {
    units.round() as i64
}

#[allow(clippy::cast_precision_loss)] // view-relative lengths on screen are far below 2^24; far-off ones need not be exact
fn world_units_to_f32(units: i64) -> f32 {
    units as f32
}

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;
    use crate::page_layout::page_top_in_world_units;

    const CANVAS_AREA: Rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 900.0));
    const SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS: i64 = 16_000;
    const WORLD_POINTS_ACROSS_SEVERAL_PAGES: [WorldPos; 3] = [
        WorldPos { x: 0, y: 0 },
        WorldPos {
            x: 9_520,
            y: 13_728,
        },
        WorldPos {
            x: 4_000,
            y: 60_000,
        },
    ];
    const ONE_PIXEL_IN_WORLD_UNITS: u32 = 16;
    const FLOAT_TOLERANCE_PX: f32 = 1e-3;
    const LAST_POSSIBLE_PAGE_INDEX: u16 = u16::MAX;
    const POINTS_FROM_PAGE_TOP_LEFT: [WorldPos; 3] = [
        WorldPos { x: 0, y: 0 },
        WorldPos { x: 4_001, y: 5_003 },
        WorldPos {
            x: 9_520,
            y: 13_472,
        },
    ];

    fn camera_scrolled_into_second_page() -> StackedPageCamera {
        StackedPageCamera::fitting_one_whole_page(
            CANVAS_AREA,
            SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS,
        )
    }

    #[test]
    fn world_point_survives_a_trip_to_the_screen_and_back() {
        let camera = camera_scrolled_into_second_page();
        for point in WORLD_POINTS_ACROSS_SEVERAL_PAGES {
            assert_eq!(camera.screen_to_world(camera.world_to_screen(point)), point);
        }
    }

    #[test]
    fn world_point_on_the_last_possible_page_survives_a_trip_to_the_screen_and_back() {
        let page_top = page_top_in_world_units(LAST_POSSIBLE_PAGE_INDEX);
        let camera = StackedPageCamera::fitting_one_whole_page(CANVAS_AREA, page_top);
        for offset in POINTS_FROM_PAGE_TOP_LEFT {
            let point = WorldPos {
                x: offset.x,
                y: page_top + offset.y,
            };
            assert_eq!(camera.screen_to_world(camera.world_to_screen(point)), point);
        }
    }

    #[test]
    fn one_pixel_of_world_is_zoom_pixels_on_screen_and_back() {
        let camera = camera_scrolled_into_second_page();
        let on_screen = camera.world_length_to_screen(ONE_PIXEL_IN_WORLD_UNITS);
        assert!((on_screen - camera.zoom()).abs() < FLOAT_TOLERANCE_PX);
        assert_eq!(
            camera.screen_length_to_world(on_screen),
            i64::from(ONE_PIXEL_IN_WORLD_UNITS)
        );
    }

    #[test]
    fn page_area_on_screen_has_the_page_shape_at_the_camera_zoom() {
        let camera = camera_scrolled_into_second_page();
        let page_on_screen = camera.world_area_to_screen(Aabb::page(1));
        let expected = world_size_at_zoom_one(Aabb::page(1)) * camera.zoom();
        assert!((page_on_screen.size() - expected).length() < FLOAT_TOLERANCE_PX);
    }
}
