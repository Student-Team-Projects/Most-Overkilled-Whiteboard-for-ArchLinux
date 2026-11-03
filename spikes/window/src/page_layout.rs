use crate::config::{
    A4_HEIGHT_PX, A4_WIDTH_PX, GAP_BETWEEN_PAGES_PX, UNPRINTABLE_MARGIN_PX, WORLD_UNITS_PER_PIXEL,
};
use crate::stand_in::{Aabb, WorldPos};

pub fn pixels_to_world_units(pixels: u16) -> i64 {
    i64::from(pixels) * i64::from(WORLD_UNITS_PER_PIXEL)
}

pub fn distance_between_page_tops_in_world_units() -> i64 {
    pixels_to_world_units(A4_HEIGHT_PX) + pixels_to_world_units(GAP_BETWEEN_PAGES_PX)
}

pub fn page_top_in_world_units(page_index: u16) -> i64 {
    i64::from(page_index) * distance_between_page_tops_in_world_units()
}

impl Aabb {
    pub fn page(page_index: u16) -> Self {
        let top = page_top_in_world_units(page_index);
        Self {
            min: WorldPos { x: 0, y: top },
            max: WorldPos {
                x: pixels_to_world_units(A4_WIDTH_PX),
                y: top + pixels_to_world_units(A4_HEIGHT_PX),
            },
        }
    }

    pub fn printable_part_of_page(page_index: u16) -> Self {
        let page = Self::page(page_index);
        let margin = pixels_to_world_units(UNPRINTABLE_MARGIN_PX);
        Self {
            min: WorldPos {
                x: page.min.x + margin,
                y: page.min.y + margin,
            },
            max: WorldPos {
                x: page.max.x - margin,
                y: page.max.y - margin,
            },
        }
    }
}
