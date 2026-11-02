use eframe::egui::color_picker::{self, Alpha};
use eframe::egui::{self, Color32, Rect, Response, RichText, Sense, Stroke, Vec2};

use crate::stand_in::WORLD_UNITS_PER_PIXEL;

const DEFAULT_STROKE_WIDTH_PX: u16 = 4;
const PRESET_STROKE_WIDTHS_PX: [u16; 3] = [2, 4, 8];
const MIN_STROKE_WIDTH_PX: u16 = 1;
const MAX_STROKE_WIDTH_PX: u16 = 32;
const NAMED_PRESET_COLORS: [(Color32, &str); 4] = [
    (Color32::BLACK, "Black"),
    (Color32::from_rgb(220, 40, 40), "Red"),
    (Color32::from_rgb(30, 90, 220), "Blue"),
    (Color32::from_rgb(30, 150, 60), "Green"),
];

const TOOL_ICON_SIZE: f32 = 24.0;
const ACTION_ICON_SIZE: f32 = 20.0;
const PRESET_BUTTON_SIZE: Vec2 = Vec2::new(40.0, 28.0);
const COLOR_SWATCH_RADIUS_PX: f32 = 9.0;
const COLOR_SWATCH_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};
const STROKE_WIDTH_PREVIEW_INSET_PX: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
}

#[derive(Debug)]
pub struct ToolState {
    pub tool: Tool,
    pub color: Color32,
    pub stroke_width_px: u16,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            tool: Tool::Pen,
            color: Color32::BLACK,
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
        }
    }
}

impl ToolState {
    pub fn stroke_width_in_world_units(&self) -> u32 {
        u32::from(self.stroke_width_px) * u32::from(WORLD_UNITS_PER_PIXEL)
    }

    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            self.tool_picker(ui);
            ui.separator();
            self.color_picker(ui);
            ui.separator();
            self.stroke_width_picker(ui);
        });
    }

    fn tool_picker(&mut self, ui: &mut egui::Ui) {
        for (tool, icon, name) in [(Tool::Pen, "✏", "Draw"), (Tool::Eraser, "🗑", "Erase")] {
            let icon = RichText::new(icon).size(TOOL_ICON_SIZE);
            ui.selectable_value(&mut self.tool, tool, icon)
                .on_hover_text(name);
        }
    }

    fn color_picker(&mut self, ui: &mut egui::Ui) {
        for (color, name) in NAMED_PRESET_COLORS {
            let (rect, response) = allocate_preset_button(ui, self.color == color);
            ui.painter()
                .circle_filled(rect.center(), COLOR_SWATCH_RADIUS_PX, color);
            ui.painter()
                .circle_stroke(rect.center(), COLOR_SWATCH_RADIUS_PX, COLOR_SWATCH_OUTLINE);
            if response.on_hover_text(name).clicked() {
                self.color = color;
            }
        }
        color_picker::color_edit_button_srgba(ui, &mut self.color, Alpha::Opaque)
            .on_hover_text("Any colour");
    }

    fn stroke_width_picker(&mut self, ui: &mut egui::Ui) {
        for width_px in PRESET_STROKE_WIDTHS_PX {
            let (rect, response) = allocate_preset_button(ui, self.stroke_width_px == width_px);
            let line = rect.shrink2(Vec2::new(STROKE_WIDTH_PREVIEW_INSET_PX, 0.0));
            let stroke = Stroke::new(f32::from(width_px), ui.visuals().text_color());
            ui.painter()
                .line_segment([line.left_center(), line.right_center()], stroke);
            if response.on_hover_text(format!("{width_px} px")).clicked() {
                self.stroke_width_px = width_px;
            }
        }
        let any_width = egui::DragValue::new(&mut self.stroke_width_px)
            .range(MIN_STROKE_WIDTH_PX..=MAX_STROKE_WIDTH_PX)
            .suffix(" px");
        ui.add(any_width)
            .on_hover_text("Any width: drag, or click and type");
    }
}

fn allocate_preset_button(ui: &mut egui::Ui, is_selected: bool) -> (Rect, Response) {
    let (rect, response) = ui.allocate_exact_size(PRESET_BUTTON_SIZE, Sense::click());
    let visuals = ui.visuals();
    let background = if is_selected {
        visuals.selection.bg_fill
    } else if response.hovered() {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, visuals.widgets.inactive.corner_radius, background);
    (rect, response)
}

pub fn action_bar_not_wired_yet(ui: &mut egui::Ui) {
    let icons_and_descriptions = [
        ("💾", "Save the board to a file (T43 #87)"),
        ("📂", "Open a board file (T35 #72)"),
        ("🌐", "Connect to a peer (T35 #72)"),
        ("📄", "Export pages to PDF (T32 #69)"),
    ];
    ui.horizontal(|ui| {
        for (icon, description) in icons_and_descriptions {
            let icon = RichText::new(icon).size(ACTION_ICON_SIZE);
            ui.button(icon).on_hover_text(description);
        }
    });
}
