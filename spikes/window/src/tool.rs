//! The pen settings chosen in the toolbar. No egui types; moves into `wb-app` in T13.

use crate::config::{DEFAULT_STROKE_COLOR, DEFAULT_STROKE_WIDTH_PX, WORLD_UNITS_PER_PIXEL};
use crate::stand_in::Rgba;

/// What the primary pointer button does on the canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Draws a freehand stroke while held.
    Pen,
    /// Removes every stroke it touches, whole (ADR 0002).
    Eraser,
}

/// The pen settings chosen in the toolbar.
#[derive(Debug)]
pub struct ToolState {
    /// The selected tool.
    pub tool: Tool,
    /// Colour of the next stroke.
    pub color: Rgba,
    /// Width of the next stroke, in page pixels (zoom 1).
    pub stroke_width_px: u16,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            tool: Tool::Pen,
            color: DEFAULT_STROKE_COLOR,
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
        }
    }
}

impl ToolState {
    /// Width of the next stroke in world units, the unit strokes are stored in.
    pub fn stroke_width_in_world_units(&self) -> u32 {
        u32::from(self.stroke_width_px) * u32::from(WORLD_UNITS_PER_PIXEL)
    }
}
