#[path = "data.rs"]
mod data;

use egui_ltreeview::{node::NodeBuilder, TreeView};

fn main() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),

        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview example",
        options,
        Box::new(|_| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            TreeView::new(ui.make_persistent_id("Names tree view")).show(ui, |mut builder| {
                builder.node(
                    NodeBuilder::dir(0)
                        .default_open(false)
                        .label(|ui| _ = ui.label("root")),
                );

                builder.node(
                    NodeBuilder::dir(1)
                        .default_open(false)
                        .label(|ui| _ = ui.label("Foo")),
                );
                builder.leaf(2, "Ava");
                builder.node(
                    NodeBuilder::dir(3)
                        .default_open(false)
                        .label(|ui| _ = ui.label("Bar")),
                );
                builder.leaf(4, "Benjamin");
                builder.leaf(5, "Charlotte");
                builder.close_dir();
                builder.close_dir();
                builder.leaf(6, "Daniel");
                builder.leaf(7, "Emma");
                builder.node(
                    NodeBuilder::dir(8)
                        .default_open(false)
                        .label(|ui| _ = ui.label("Baz")),
                );
                builder.leaf(9, "Finn");
                builder.leaf(10, "Grayson");
                builder.close_dir();

                builder.close_dir();
            });
        });
    }
}
