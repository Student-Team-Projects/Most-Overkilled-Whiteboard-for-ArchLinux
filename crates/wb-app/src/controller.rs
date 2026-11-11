//! The `Controller` trait and its controllers: they turn what the user did into changes of the
//! model, and are the model's only writer.

mod page_stack;

pub use page_stack::PageStackController;

use egui::{Pos2, Rect, Vec2};
use wb_core::Rgba;

use crate::model::Model;
use crate::tool::Tool;

/// What the user asked for with a toolbar button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Switch to this tool.
    SelectTool(Tool),
    /// Draw the next strokes in this colour.
    SelectColor(Rgba),
    /// Draw the next strokes this wide, in pixels at zoom 1.
    SelectStrokeWidth(u16),
    /// Append a page below the last one; the view stays where it is.
    AddPage,
}

/// What the pointer did on the canvas during one frame, in screen coordinates.
#[derive(Clone, Copy, Debug)]
pub struct CanvasInput {
    /// The canvas rectangle on screen.
    pub area: Rect,
    /// Where the primary button is held or was just clicked; `None` otherwise.
    pub pointer_pressed_at: Option<Pos2>,
    /// Whether the primary button is still held on the canvas.
    pub is_pointer_held: bool,
    /// How far the mouse wheel or touchpad scrolled over the canvas, in screen pixels.
    pub scroll_delta: Vec2,
}

/// Turns toolbar commands and canvas input into changes of a model. Board modes differ in how
/// the canvas reacts; toolbar commands mean the same everywhere.
pub trait Controller {
    /// Changes `model` as `command` asks.
    fn apply(&mut self, model: &mut Model, command: Command) {
        let tools = &mut model.tools;
        match command {
            Command::SelectTool(tool) => tools.tool = tool,
            Command::SelectColor(color) => tools.color = color,
            Command::SelectStrokeWidth(width_px) => tools.stroke_width_px = width_px,
            Command::AddPage => {
                model.pages.add_page();
            }
        }
    }

    /// Lets the camera and the current tool react to one frame of canvas input.
    fn handle_canvas_input(&mut self, model: &mut Model, input: &CanvasInput);
}
