//! Every runtime tunable of the app: world scale, zoom, tools, encoding and window styling.

use egui::{Color32, Stroke, Vec2};
use wb_core::Rgba;

/// World units in one screen pixel at zoom 1 (ADR 0014).
pub const WORLD_UNITS_PER_PIXEL: u16 = 16;

/// Space kept between the fitted page and the canvas edges, in screen pixels.
pub const SPACE_AROUND_PAGE_PX: f32 = 16.0;

/// Smallest zoom the camera allows (ADR 0013).
pub const MIN_ZOOM: f32 = 0.5;
/// Largest zoom the camera allows (ADR 0013).
pub const MAX_ZOOM: f32 = 2.0;

/// Largest encoded piece of a stroke, in bytes. Below the 1200-byte datagram so an Add packet
/// with its metadata still fits; the exact budget comes with the packet format (T26 #50).
pub const PIECE_BUDGET_BYTES: usize = 1000;

/// Eraser radius in screen pixels, the same at every zoom (ADR 0013).
pub const ERASER_RADIUS_ON_SCREEN_PX: f32 = 8.0;

/// Stroke width at start-up, in pixels at zoom 1.
pub const DEFAULT_STROKE_WIDTH_PX: u16 = 4;
/// Widths offered as one-click buttons, in pixels at zoom 1.
pub const PRESET_STROKE_WIDTHS_PX: [u16; 3] = [2, 4, 8];
/// Narrowest width the toolbar accepts, in pixels at zoom 1.
pub const MIN_STROKE_WIDTH_PX: u16 = 1;
/// Widest width the toolbar accepts, in pixels at zoom 1.
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

/// Canvas colour around the page.
pub const BACKGROUND_BEHIND_PAGE: Color32 = Color32::from_gray(40);
/// Colour of the page.
pub const PAGE_COLOR: Color32 = Color32::WHITE;
/// Line around the printable part of the page, outside it, in the margin.
pub const PRINTABLE_AREA_OUTLINE: Stroke = Stroke {
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
