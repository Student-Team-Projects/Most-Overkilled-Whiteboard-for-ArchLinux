//! The whiteboard desktop application.

use std::hash::{BuildHasher, RandomState};

use wb_app::config::{INITIAL_WINDOW_SIZE, TOOLBAR_WIDTH_PX, TOOLTIP_DELAY_SECONDS};
use wb_app::controller::{Controller, SinglePageController};
use wb_app::model::Model;
use wb_app::view;
use wb_core::PeerId;

struct App {
    model: Model,
    controller: SinglePageController,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("actions").show(ui, view::action_bar::show_not_wired_yet);
        let toolbar_commands = egui::Panel::left("toolbar")
            .resizable(false)
            .exact_size(TOOLBAR_WIDTH_PX)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        view::toolbar::Toolbar::new(&self.model.tools).show(ui)
                    })
                    .inner
            })
            .inner;
        for command in toolbar_commands {
            self.controller.apply(&mut self.model, command);
        }
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let (input, canvas) = view::canvas::AllocatedCanvas::new(ui);
            self.controller.handle_canvas_input(&mut self.model, &input);
            canvas.paint(&mut self.model);
        });
    }
}

fn random_peer_id() -> PeerId {
    PeerId(RandomState::new().hash_one(std::process::id()))
}

fn show_tooltips_while_pointer_moves(context: &egui::Context) {
    context.all_styles_mut(|style| {
        style.interaction.show_tooltips_only_when_still = false;
        style.interaction.tooltip_delay = TOOLTIP_DELAY_SECONDS;
    });
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Most Overkilled Whiteboard",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(INITIAL_WINDOW_SIZE),
            ..Default::default()
        },
        Box::new(|creation_context| {
            show_tooltips_while_pointer_moves(&creation_context.egui_ctx);
            Ok(Box::new(App {
                model: Model::new(random_peer_id()),
                controller: SinglePageController::default(),
            }))
        }),
    )
}
