//! The left toolbar: tool, colour and stroke width.

use eframe::egui::color_picker::{self, Alpha};
use eframe::egui::{self, Color32, Rect, Response, RichText, Sense, Stroke, Vec2};

use crate::config::{
    MAX_STROKE_WIDTH_PX, MIN_STROKE_WIDTH_PX, NAMED_PRESET_COLORS, PRESET_STROKE_WIDTHS_PX,
};
use crate::controller::Command;
use crate::stand_in::Rgba;
use crate::tool::{Tool, ToolState};

const TOOL_ICON_SIZE: f32 = 24.0;
const PRESET_BUTTON_SIZE: Vec2 = Vec2::new(40.0, 28.0);
const COLOR_SWATCH_RADIUS_PX: f32 = 9.0;
const COLOR_SWATCH_OUTLINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::GRAY,
};
const STROKE_WIDTH_PREVIEW_INSET_PX: f32 = 8.0;

/// Shows the tool, colour and width pickers with the current choice highlighted.
/// Returns one command per choice changed this frame.
pub fn show(ui: &mut egui::Ui, tools: &ToolState) -> Vec<Command> {
    let mut commands = Vec::new();
    ui.vertical_centered(|ui| {
        tool_picker(ui, tools, &mut commands);
        ui.separator();
        color_picker(ui, tools, &mut commands);
        ui.separator();
        stroke_width_picker(ui, tools, &mut commands);
    });
    commands
}

fn tool_picker(ui: &mut egui::Ui, tools: &ToolState, commands: &mut Vec<Command>) {
    for (tool, icon, name) in [(Tool::Pen, "✏", "Draw"), (Tool::Eraser, "🗑", "Erase")] {
        let icon = RichText::new(icon).size(TOOL_ICON_SIZE);
        let button = ui.selectable_label(tools.tool == tool, icon);
        if button.on_hover_text(name).clicked() {
            commands.push(Command::SelectTool(tool));
        }
    }
}

fn color_picker(ui: &mut egui::Ui, tools: &ToolState, commands: &mut Vec<Command>) {
    for (color, name) in NAMED_PRESET_COLORS {
        let (rect, response) = allocate_preset_button(ui, tools.color == color);
        ui.painter()
            .circle_filled(rect.center(), COLOR_SWATCH_RADIUS_PX, Color32::from(color));
        ui.painter()
            .circle_stroke(rect.center(), COLOR_SWATCH_RADIUS_PX, COLOR_SWATCH_OUTLINE);
        if response.on_hover_text(name).clicked() {
            commands.push(Command::SelectColor(color));
        }
    }
    let mut any_color = Color32::from(tools.color);
    let any_color_button = color_picker::color_edit_button_srgba(ui, &mut any_color, Alpha::Opaque);
    if any_color_button.on_hover_text("Any colour").changed() {
        commands.push(Command::SelectColor(Rgba(
            any_color.to_srgba_unmultiplied(),
        )));
    }
}

fn stroke_width_picker(ui: &mut egui::Ui, tools: &ToolState, commands: &mut Vec<Command>) {
    for width_px in PRESET_STROKE_WIDTHS_PX {
        let (rect, response) = allocate_preset_button(ui, tools.stroke_width_px == width_px);
        let line = rect.shrink2(Vec2::new(STROKE_WIDTH_PREVIEW_INSET_PX, 0.0));
        let stroke = Stroke::new(f32::from(width_px), ui.visuals().text_color());
        ui.painter()
            .line_segment([line.left_center(), line.right_center()], stroke);
        if response.on_hover_text(format!("{width_px} px")).clicked() {
            commands.push(Command::SelectStrokeWidth(width_px));
        }
    }
    let mut any_width_px = tools.stroke_width_px;
    let any_width = egui::DragValue::new(&mut any_width_px)
        .range(MIN_STROKE_WIDTH_PX..=MAX_STROKE_WIDTH_PX)
        .suffix(" px");
    if ui
        .add(any_width)
        .on_hover_text("Any width: drag, or click and type")
        .changed()
    {
        commands.push(Command::SelectStrokeWidth(any_width_px));
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
