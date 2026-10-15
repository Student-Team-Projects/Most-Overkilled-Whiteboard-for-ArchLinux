//! The whiteboard desktop application.

struct EmptyApp;

impl eframe::App for EmptyApp {
    fn ui(&mut self, _ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {}
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Most Overkilled Whiteboard",
        options,
        Box::new(|_creation_context| Ok(Box::new(EmptyApp))),
    )
}
