use eframe::egui::{self, RichText};

const ACTION_ICON_SIZE: f32 = 20.0;

pub fn show_not_wired_yet(ui: &mut egui::Ui) {
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
