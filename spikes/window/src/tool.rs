use eframe::egui::{self, Color32, RichText};

use crate::stand_in::WORLD_UNITS_PER_PIXEL;

const DEFAULT_STROKE_WIDTH_PX: u16 = 4;
const MIN_STROKE_WIDTH_PX: u16 = 1;
const MAX_STROKE_WIDTH_PX: u16 = 32;
const ACTION_ICON_SIZE: f32 = 20.0;

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
    pub fn stroke_width_in_world_units(&self) -> u16 {
        self.stroke_width_px * WORLD_UNITS_PER_PIXEL
    }

    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        self.tool_picker(ui);
        ui.separator();
        self.stroke_style_picker(ui);
        ui.separator();
        infinite_canvas_toggle_disabled_until_after_mvp(ui);
        ui.separator();
        ui.horizontal_wrapped(action_buttons_not_wired_yet);
    }

    fn tool_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Pen, "Draw");
            ui.selectable_value(&mut self.tool, Tool::Eraser, "Erase");
        });
    }

    fn stroke_style_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut self.color);
            let width_range = MIN_STROKE_WIDTH_PX..=MAX_STROKE_WIDTH_PX;
            ui.add(egui::Slider::new(&mut self.stroke_width_px, width_range))
                .on_hover_text("Stroke width in pixels");
        });
    }
}

fn infinite_canvas_toggle_disabled_until_after_mvp(ui: &mut egui::Ui) {
    let mut infinite_canvas = false;
    let toggle = egui::Checkbox::new(&mut infinite_canvas, "Infinite canvas");
    ui.add_enabled(false, toggle)
        .on_disabled_hover_text("Draw anywhere instead of on pages (M4, after the MVP)");
}

fn action_buttons_not_wired_yet(ui: &mut egui::Ui) {
    let icons_and_descriptions = [
        ("💾", "Save the board to a file (T43 #87)"),
        ("📂", "Open a board file (T35 #72)"),
        ("🔗", "Connect to a peer (T35 #72)"),
        ("📄", "Export pages to PDF (T32 #69)"),
    ];
    for (icon, description) in icons_and_descriptions {
        let icon = RichText::new(icon).size(ACTION_ICON_SIZE);
        ui.button(icon).on_hover_text(description);
    }
}
