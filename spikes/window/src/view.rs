use eframe::egui::{Pos2, Rect, Vec2};

use crate::config::{SPACE_AROUND_PAGE_PX, WORLD_UNITS_PER_PIXEL};
use crate::stand_in::{Aabb, WorldPos};

#[derive(Clone, Copy, Debug)]
pub struct StackedPageView {
    first_page_top_left: Pos2,
    zoom: f32,
}

impl StackedPageView {
    pub fn fitting_one_whole_page(area: Rect, scroll_offset_in_world_units: i64) -> Self {
        let page_at_zoom_one = world_size_at_zoom_one(Aabb::page(0));
        let room = (area.size() - Vec2::splat(2.0 * SPACE_AROUND_PAGE_PX)).max(Vec2::ZERO);
        let zoom = (room.x / page_at_zoom_one.x).min(room.y / page_at_zoom_one.y);
        let scroll_offset_at_zoom_one =
            world_units_to_f32(scroll_offset_in_world_units) / f32::from(WORLD_UNITS_PER_PIXEL);
        let left = area.center().x - page_at_zoom_one.x * zoom / 2.0;
        let top = area.top() + SPACE_AROUND_PAGE_PX - scroll_offset_at_zoom_one * zoom;
        Self {
            first_page_top_left: Pos2::new(left, top),
            zoom,
        }
    }

    pub fn zoom(self) -> f32 {
        self.zoom
    }

    pub fn world_to_screen(self, point: WorldPos) -> Pos2 {
        let world_units = Vec2::new(world_units_to_f32(point.x), world_units_to_f32(point.y));
        self.first_page_top_left + self.world_units_to_screen(world_units)
    }

    pub fn screen_to_world(self, pos: Pos2) -> WorldPos {
        let world_units = self.screen_to_world_units(pos - self.first_page_top_left);
        WorldPos {
            x: round_to_world_units(world_units.x),
            y: round_to_world_units(world_units.y),
        }
    }

    pub fn world_length_to_screen(self, length_in_world_units: u32) -> f32 {
        let length = world_units_to_f32(i64::from(length_in_world_units));
        self.world_units_to_screen(Vec2::new(length, 0.0)).x
    }

    pub fn screen_length_to_world(self, length_on_screen: f32) -> i64 {
        round_to_world_units(
            self.screen_to_world_units(Vec2::new(length_on_screen, 0.0))
                .x,
        )
    }

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

#[allow(clippy::cast_precision_loss)] // page-stack coordinates stay far below f32's exact range
fn world_units_to_f32(units: i64) -> f32 {
    units as f32
}

#[cfg(test)]
mod tests_of_private_items_in_binary_crate {
    use super::*;

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

    fn view_scrolled_into_second_page() -> StackedPageView {
        StackedPageView::fitting_one_whole_page(
            CANVAS_AREA,
            SCROLLED_INTO_SECOND_PAGE_IN_WORLD_UNITS,
        )
    }

    #[test]
    fn world_point_survives_a_trip_to_the_screen_and_back() {
        let view = view_scrolled_into_second_page();
        for point in WORLD_POINTS_ACROSS_SEVERAL_PAGES {
            assert_eq!(view.screen_to_world(view.world_to_screen(point)), point);
        }
    }

    #[test]
    fn one_pixel_of_world_is_zoom_pixels_on_screen_and_back() {
        let view = view_scrolled_into_second_page();
        let on_screen = view.world_length_to_screen(ONE_PIXEL_IN_WORLD_UNITS);
        assert!((on_screen - view.zoom()).abs() < FLOAT_TOLERANCE_PX);
        assert_eq!(
            view.screen_length_to_world(on_screen),
            i64::from(ONE_PIXEL_IN_WORLD_UNITS)
        );
    }

    #[test]
    fn page_area_on_screen_has_the_page_shape_at_the_view_zoom() {
        let view = view_scrolled_into_second_page();
        let page_on_screen = view.world_area_to_screen(Aabb::page(1));
        let expected = world_size_at_zoom_one(Aabb::page(1)) * view.zoom();
        assert!((page_on_screen.size() - expected).length() < FLOAT_TOLERANCE_PX);
    }
}
