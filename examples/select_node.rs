//! Using the [`TreeViewState`] it is possible to change which
//! node is selected without interacting with the tree.

#[path = "data.rs"]
mod data;

use egui::Id;
use egui_ltreeview::{TreeView, TreeViewState};

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),
        default_theme: eframe::Theme::Dark,
        follow_system_theme: false,
        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview select node example",
        options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

struct MyApp {
    tree: TreeViewState<i32>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tree: TreeViewState::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left(Id::new("left")).show(ctx, |ui| {
            TreeView::new(ui.make_persistent_id("Names tree view")).show_state(
                ui,
                &mut self.tree,
                |mut builder| {
                    builder.dir(0, "Root");
                    builder.dir(1, "Foo");
                    builder.leaf(2, "Ava");
                    builder.dir(3, "Bar");
                    builder.leaf(4, "Benjamin");
                    builder.leaf(5, "Charlotte");
                    builder.close_dir();
                    builder.close_dir();
                    builder.leaf(6, "Daniel");
                    builder.leaf(7, "Emma");
                    builder.dir(8, "Baz");
                    builder.leaf(9, "Finn");
                    builder.leaf(10, "Grayson");
                    builder.close_dir();
                    builder.close_dir();
                },
            );
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("select next").clicked() {
                let selected_index = self.tree.selected().unwrap_or(0);
                self.tree.set_selected(Some((selected_index + 1) % 11));
            }
        });
    }
}
