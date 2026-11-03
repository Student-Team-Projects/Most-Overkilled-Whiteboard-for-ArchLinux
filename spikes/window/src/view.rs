pub mod action_bar;
pub mod canvas;
pub mod page_bar;
pub mod toolbar;

use eframe::egui::Color32;

use crate::stand_in::Rgba;

impl From<Rgba> for Color32 {
    fn from(Rgba([r, g, b, a]): Rgba) -> Self {
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}
