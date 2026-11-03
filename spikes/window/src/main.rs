//! T27 spike: an eframe window with a toolbar and a freehand canvas on stand-in types.

mod canvas;
mod config;
mod stand_in;
mod tool;

use eframe::egui;

use canvas::StackedPageCanvas;
use config::{INITIAL_WINDOW_SIZE, TOOLBAR_WIDTH_PX, TOOLTIP_DELAY_SECONDS};
use tool::ToolState;

#[derive(Debug, Default)]
struct App {
    tools: ToolState,
    canvas: StackedPageCanvas,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("actions").show(ui, tool::action_bar_not_wired_yet);
        egui::Panel::left("toolbar")
            .resizable(false)
            .exact_size(TOOLBAR_WIDTH_PX)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.tools.toolbar(ui));
            });
        egui::Panel::bottom("pages").show(ui, |ui| self.canvas.page_bar(ui));
        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas.ui(ui, &self.tools));
    }
}

fn show_tooltips_while_pointer_moves(context: &egui::Context) {
    context.all_styles_mut(|style| {
        style.interaction.show_tooltips_only_when_still = false;
        style.interaction.tooltip_delay = TOOLTIP_DELAY_SECONDS;
    });
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Window spike (T27)",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(INITIAL_WINDOW_SIZE),
            ..Default::default()
        },
        Box::new(|creation_context| {
            show_tooltips_while_pointer_moves(&creation_context.egui_ctx);
            Ok(Box::<App>::default())
        }),
    )
}
