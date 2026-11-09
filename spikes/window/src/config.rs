//! Every runtime tunable of the spike: page geometry, world scale, tools and window.

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
