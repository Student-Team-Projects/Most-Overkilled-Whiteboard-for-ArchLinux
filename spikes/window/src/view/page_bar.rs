//! The bottom bar that moves between pages and adds pages.

use eframe::egui;

use crate::controller::Command;
use crate::model::Model;

/// Shows previous / next page buttons, "Page n / m" and an add-page button.
/// Returns the command of the button clicked this frame, if any.
pub fn show(ui: &mut egui::Ui, model: &Model) -> Option<Command> {
    let current_page = model.page_nearest_to_top_of_view();
    let last_page = model.board().last_page_index();
    let page_number = |page_index: u16| u32::from(page_index) + 1;
    let mut command = None;
    ui.horizontal(|ui| {
        let previous = ui.add_enabled(current_page > 0, egui::Button::new("◀"));
        if previous
            .on_hover_text("Scroll to the previous page")
            .clicked()
        {
            command = Some(Command::ScrollToTopOf(current_page - 1));
        }
        ui.label(format!(
            "Page {} / {}",
            page_number(current_page),
            page_number(last_page)
        ));
        let next = ui.add_enabled(current_page < last_page, egui::Button::new("▶"));
        if next.on_hover_text("Scroll to the next page").clicked() {
            command = Some(Command::ScrollToTopOf(current_page + 1));
        }
        if ui
            .button("➕")
            .on_hover_text("Add a page at the end")
            .clicked()
        {
            command = Some(Command::AddPageAtEnd);
        }
    });
    command
}
