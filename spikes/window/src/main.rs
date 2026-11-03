//! T27 spike: an eframe window with a toolbar and a freehand canvas on stand-in types.

mod camera;
mod config;
mod controller;
mod model;
mod page_layout;
mod stand_in;
mod view;

use eframe::egui;

use config::{INITIAL_WINDOW_SIZE, TOOLBAR_WIDTH_PX, TOOLTIP_DELAY_SECONDS};
use model::Model;

#[derive(Debug, Default)]
struct App {
    model: Model,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("actions").show(ui, view::action_bar::show_not_wired_yet);
        let toolbar_commands = egui::Panel::left("toolbar")
            .resizable(false)
            .exact_size(TOOLBAR_WIDTH_PX)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| view::toolbar::show(ui, &self.model.tools))
                    .inner
            })
            .inner;
        let page_bar_command = egui::Panel::bottom("pages")
            .show(ui, |ui| view::page_bar::show(ui, &self.model))
            .inner;
        for command in toolbar_commands.into_iter().chain(page_bar_command) {
            controller::apply(&mut self.model, command);
        }
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let (input, canvas) = view::canvas::allocate(ui);
            controller::handle_canvas_input(&mut self.model, &input);
            view::canvas::paint(&canvas, &self.model);
        });
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
