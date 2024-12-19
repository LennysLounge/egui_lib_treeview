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
                builder.dir(0, "root");

                // Sometimes you want to a section of the tree to behave like a dir
                // without incrasing the depth of the tree. In that case you can flatten
                // the dir. This will not render the dir but still register it in the tree.
                builder.node(
                    NodeBuilder::dir(1)
                        .flatten(true)
                        .label(|ui| _ = ui.label("Foo")),
                );
                builder.leaf(2, "Ava");
                builder.node(
                    NodeBuilder::dir(3)
                        .flatten(true)
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
                        .flatten(true)
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
