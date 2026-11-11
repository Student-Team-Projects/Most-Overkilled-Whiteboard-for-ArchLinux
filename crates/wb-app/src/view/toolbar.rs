//! The left toolbar: tool, colour and stroke width.

use egui::color_picker::{self, Alpha};
use egui::{Color32, Rect, Response, RichText, Sense, Stroke, Vec2};

use crate::config::{
    COLOR_SWATCH_OUTLINE, COLOR_SWATCH_RADIUS_PX, MAX_STROKE_WIDTH_PX, MIN_STROKE_WIDTH_PX,
    NAMED_PRESET_COLORS, PRESET_BUTTON_SIZE, PRESET_STROKE_WIDTHS_PX,
    STROKE_WIDTH_PREVIEW_INSET_PX, TOOL_ICON_SIZE,
};
use wb_core::Rgba;

use crate::controller::Command;
use crate::tool::{Tool, ToolState};

/// The left toolbar for one frame: shows the tool, colour and width pickers with the current
/// choice highlighted, and collects what the user picked.
pub struct Toolbar<'a> {
    tools: &'a ToolState,
    commands: Vec<Command>,
}

impl<'a> Toolbar<'a> {
    /// A toolbar highlighting the choices in `tools`.
    #[must_use]
    pub fn new(tools: &'a ToolState) -> Self {
        Self {
            tools,
            commands: Vec::new(),
        }
    }

    /// Shows the pickers in `ui` and returns one command per choice changed this frame.
    pub fn show(mut self, ui: &mut egui::Ui) -> Vec<Command> {
        ui.vertical_centered(|ui| {
            self.tool_picker(ui);
            ui.separator();
            self.color_picker(ui);
            ui.separator();
            self.stroke_width_picker(ui);
        });
        self.commands
    }

    fn tool_picker(&mut self, ui: &mut egui::Ui) {
        for (tool, icon, name) in [(Tool::Pen, "✏", "Draw"), (Tool::Eraser, "🗑", "Erase")] {
            let icon = RichText::new(icon).size(TOOL_ICON_SIZE);
            let button = ui.selectable_label(self.tools.tool == tool, icon);
            if button.on_hover_text(name).clicked() {
                self.commands.push(Command::SelectTool(tool));
            }
        }
    }

    fn color_picker(&mut self, ui: &mut egui::Ui) {
        for (color, name) in NAMED_PRESET_COLORS {
            let (rect, response) = Self::preset_button(ui, self.tools.color == color);
            ui.painter()
                .circle_filled(rect.center(), COLOR_SWATCH_RADIUS_PX, Color32::from(color));
            ui.painter()
                .circle_stroke(rect.center(), COLOR_SWATCH_RADIUS_PX, COLOR_SWATCH_OUTLINE);
            if response.on_hover_text(name).clicked() {
                self.commands.push(Command::SelectColor(color));
            }
        }
        let mut any_color = Color32::from(self.tools.color);
        let any_color_button =
            color_picker::color_edit_button_srgba(ui, &mut any_color, Alpha::Opaque);
        if any_color_button.on_hover_text("Any colour").changed() {
            self.commands
                .push(Command::SelectColor(Rgba::from(any_color)));
        }
    }

    fn stroke_width_picker(&mut self, ui: &mut egui::Ui) {
        for width_px in PRESET_STROKE_WIDTHS_PX {
            let (rect, response) = Self::preset_button(ui, self.tools.stroke_width_px == width_px);
            let line = rect.shrink2(Vec2::new(STROKE_WIDTH_PREVIEW_INSET_PX, 0.0));
            let stroke = Stroke::new(f32::from(width_px), ui.visuals().text_color());
            ui.painter()
                .line_segment([line.left_center(), line.right_center()], stroke);
            if response.on_hover_text(format!("{width_px} px")).clicked() {
                self.commands.push(Command::SelectStrokeWidth(width_px));
            }
        }
        let mut any_width_px = self.tools.stroke_width_px;
        let any_width = egui::DragValue::new(&mut any_width_px)
            .range(MIN_STROKE_WIDTH_PX..=MAX_STROKE_WIDTH_PX)
            .suffix(" px");
        if ui
            .add(any_width)
            .on_hover_text("Any width: drag, or click and type")
            .changed()
        {
            self.commands.push(Command::SelectStrokeWidth(any_width_px));
        }
    }

    /// A clickable preset slot, its background showing whether it is selected or hovered.
    fn preset_button(ui: &mut egui::Ui, is_selected: bool) -> (Rect, Response) {
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
}
