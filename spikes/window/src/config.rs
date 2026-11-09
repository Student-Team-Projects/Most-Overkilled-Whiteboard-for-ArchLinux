//! Every runtime tunable of the spike: page geometry, world scale, tools and window.

use eframe::egui::{Color32, Stroke, Vec2};

use crate::stand_in::Rgba;

/// World units in one page pixel at zoom 1 (ADR 0014).
pub const WORLD_UNITS_PER_PIXEL: u16 = 16;

/// Page width: A4 at 72 dpi, in page pixels.
pub const A4_WIDTH_PX: u16 = 595;
/// Page height: A4 at 72 dpi, in page pixels.
pub const A4_HEIGHT_PX: u16 = 842;
/// Margin on every side of a page where the pen does not draw, in page pixels.
pub const UNPRINTABLE_MARGIN_PX: u16 = 14;
/// Gap between two pages of the stack, in page pixels.
pub const GAP_BETWEEN_PAGES_PX: u16 = 16;
/// Space kept between the fitted page and the canvas edges, in screen pixels.
pub const SPACE_AROUND_PAGE_PX: f32 = 16.0;

/// Eraser radius in screen pixels, the same at every zoom (ADR 0013).
pub const ERASER_RADIUS_ON_SCREEN_PX: f32 = 8.0;

/// Stroke width at start-up, in page pixels.
pub const DEFAULT_STROKE_WIDTH_PX: u16 = 4;
/// Widths offered as one-click buttons, in page pixels.
pub const PRESET_STROKE_WIDTHS_PX: [u16; 3] = [2, 4, 8];
/// Narrowest width the toolbar accepts, in page pixels.
pub const MIN_STROKE_WIDTH_PX: u16 = 1;
/// Widest width the toolbar accepts, in page pixels.
pub const MAX_STROKE_WIDTH_PX: u16 = 32;
/// Stroke colour at start-up: opaque black.
pub const DEFAULT_STROKE_COLOR: Rgba = Rgba([0, 0, 0, 255]);
/// Colours offered as one-click buttons, with their tooltip names.
pub const NAMED_PRESET_COLORS: [(Rgba, &str); 4] = [
    (DEFAULT_STROKE_COLOR, "Black"),
    (Rgba([220, 40, 40, 255]), "Red"),
    (Rgba([30, 90, 220, 255]), "Blue"),
    (Rgba([30, 150, 60, 255]), "Green"),
];

/// Window size at start-up, in screen pixels.
pub const INITIAL_WINDOW_SIZE: [f32; 2] = [1280.0, 800.0];
/// Width of the left toolbar, in screen pixels.
pub const TOOLBAR_WIDTH_PX: f32 = 56.0;
/// Delay before a tooltip appears.
pub const TOOLTIP_DELAY_SECONDS: f32 = 0.2;

/// Canvas colour around and between the pages.
pub const BACKGROUND_BEHIND_PAGES: Color32 = Color32::from_gray(40);
/// Line around the printable part of each page, outside it, in the margin.
pub const UNPRINTABLE_MARGIN_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_gray(220),
};
/// Circle showing the eraser's reach under the pointer.
pub const ERASER_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};

/// Font size of the save, open, connect and PDF icons in the top bar.
pub const ACTION_ICON_SIZE: f32 = 20.0;
/// Font size of the Draw and Erase icons in the toolbar.
pub const TOOL_ICON_SIZE: f32 = 24.0;
/// Size of each colour and width preset button, in screen pixels.
pub const PRESET_BUTTON_SIZE: Vec2 = Vec2::new(40.0, 28.0);
/// Radius of the colour dot on a colour preset button, in screen pixels.
pub const COLOR_SWATCH_RADIUS_PX: f32 = 9.0;
/// Outline of the colour dot, so white and light colours stay visible.
pub const COLOR_SWATCH_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};
/// Space left and right of the line drawn on a width preset button, in screen pixels.
pub const STROKE_WIDTH_PREVIEW_INSET_PX: f32 = 8.0;
