use eframe::egui::Color32;

pub const WORLD_UNITS_PER_PIXEL: u16 = 16;

pub const A4_WIDTH_PX: u16 = 595;
pub const A4_HEIGHT_PX: u16 = 842;
pub const UNPRINTABLE_MARGIN_PX: u16 = 14;
pub const GAP_BETWEEN_PAGES_PX: u16 = 16;
pub const SPACE_AROUND_PAGE_PX: f32 = 16.0;

pub const ERASER_RADIUS_ON_SCREEN_PX: f32 = 8.0;

pub const DEFAULT_STROKE_WIDTH_PX: u16 = 4;
pub const PRESET_STROKE_WIDTHS_PX: [u16; 3] = [2, 4, 8];
pub const MIN_STROKE_WIDTH_PX: u16 = 1;
pub const MAX_STROKE_WIDTH_PX: u16 = 32;
pub const NAMED_PRESET_COLORS: [(Color32, &str); 4] = [
    (Color32::BLACK, "Black"),
    (Color32::from_rgb(220, 40, 40), "Red"),
    (Color32::from_rgb(30, 90, 220), "Blue"),
    (Color32::from_rgb(30, 150, 60), "Green"),
];

pub const INITIAL_WINDOW_SIZE: [f32; 2] = [1280.0, 800.0];
pub const TOOLBAR_WIDTH_PX: f32 = 56.0;
pub const TOOLTIP_DELAY_SECONDS: f32 = 0.2;
