mod data;
use std::env;

use data::*;
use egui::{vec2, DragValue, Id, Ui};
use egui_ltreeview::{builder::NodeBuilder, RowLayout, TreeView, TreeViewBuilder, VLineStyle};
use uuid::Uuid;

fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_BACKTRACE", "1");
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        //viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),
        default_theme: eframe::Theme::Dark,
        follow_system_theme: false,
        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview example",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    tree: Node,
    settings_id: Uuid,
    selected_node: Option<Uuid>,
    settings: Settings,
}

#[derive(Default)]
struct Settings {
    override_indent: Option<f32>,
    vline_style: VLineStyle,
    row_layout: RowLayout,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tree: make_tree(),
            settings_id: Uuid::new_v4(),
            selected_node: None,
            settings: Settings {
                row_layout: RowLayout::CompactAlignedLables,
                ..Default::default()
            },
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left(Id::new("tree view"))
            .resizable(true)
            .show(ctx, |ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.0));
                let response = TreeView::new(ui.make_persistent_id("Names tree view"))
                    .override_indent(self.settings.override_indent)
                    .vline_style(self.settings.vline_style)
                    .row_layout(self.settings.row_layout)
                    .show(ui, |mut builder| {
                        builder.node(
                            NodeBuilder::leaf(self.settings_id).icon(|ui| {
                                egui::Image::new(egui::include_image!("settings.png"))
                                    .tint(ui.visuals().widgets.noninteractive.fg_stroke.color)
                                    .paint_at(ui, ui.max_rect());
                            }),
                            |ui| {
                                ui.label("Settings");
                            },
                        );
                        show_node(&mut builder, &self.tree);
                    });
                self.selected_node = response.selected_node;
                //response.draw_nodes(ui);
                response.context_menu(ui, |ui, node_id| {
                    self.tree.find_mut(&node_id, &mut |node| match node {
                        Node::Directory(dir) => {
                            ui.label("dir:");
                            ui.label(&dir.name);
                        }
                        Node::File(file) => {
                            ui.label("file:");
                            ui.label(&file.name);
                        }
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected_node) = self.selected_node.as_ref() {
                if selected_node == &self.settings_id {
                    show_settings(ui, &mut self.settings);
                } else {
                    self.tree.find_mut(selected_node, &mut |node| {
                        show_node_content(ui, node);
                    });
                }
            }
        });
    }
}

fn show_node(builder: &mut TreeViewBuilder<Uuid>, node: &Node) {
    match node {
        Node::Directory(dir) => show_dir(builder, dir),
        Node::File(file) => show_file(builder, file),
    }
}
fn show_dir(builder: &mut TreeViewBuilder<Uuid>, dir: &Directory) {
    let mut node = NodeBuilder::dir(dir.id);
    if dir.icon {
        node = node.icon(|ui| {
            egui::Image::new(egui::include_image!("folder.png"))
                .tint(ui.visuals().widgets.noninteractive.fg_stroke.color)
                .paint_at(ui, ui.max_rect());
        });
    }
    if dir.custom_closer {
        node = node.closer(|ui, state| {
            let color = if state.is_hovered {
                ui.visuals().widgets.hovered.fg_stroke.color
            } else {
                ui.visuals().widgets.noninteractive.fg_stroke.color
            };
            if state.is_open {
                egui::Image::new(egui::include_image!("folder_open.png"))
                    .tint(color)
                    .paint_at(ui, ui.max_rect());
            } else {
                egui::Image::new(egui::include_image!("folder.png"))
                    .tint(color)
                    .paint_at(ui, ui.max_rect());
            }
        });
    }
    builder.node(node, |ui| {
        ui.label(&dir.name);
    });

    for node in dir.children.iter() {
        show_node(builder, node);
    }

    builder.close_dir();
}
fn show_file(builder: &mut TreeViewBuilder<Uuid>, file: &File) {
    let mut node = NodeBuilder::leaf(file.id);
    if file.icon {
        node = node.icon(|ui| {
            egui::Image::new(egui::include_image!("user.png"))
                .tint(ui.visuals().widgets.noninteractive.fg_stroke.color)
                .paint_at(ui, ui.max_rect());
        });
    }
    builder.node(node, |ui| {
        ui.label(&file.name);
    });
}

fn show_settings(ui: &mut Ui, settings: &mut Settings) {
    egui::Grid::new("settings grid").show(ui, |ui| {
        ui.strong("Egui:");
        ui.end_row();

        ui.label("Indent:");
        let mut indent = ui.ctx().style().spacing.indent;
        ui.add(DragValue::new(&mut indent).clamp_range(0.0..=f32::INFINITY));
        ui.ctx().style_mut(|style| {
            style.spacing.indent = indent;
        });
        ui.end_row();

        ui.label("Item spacing:");
        ui.horizontal(|ui| {
            let mut spacing = ui.ctx().style().spacing.item_spacing;
            ui.add(DragValue::new(&mut spacing.x));
            ui.add(DragValue::new(&mut spacing.y));
            ui.ctx().style_mut(|style| {
                style.spacing.item_spacing = spacing;
            });
        });
        ui.end_row();

        ui.end_row();

        ui.strong("Tree view settings");
        ui.end_row();

        ui.label("Override indent");
        ui.horizontal(|ui| {
            let mut override_enabled = settings.override_indent.is_some();
            if ui.checkbox(&mut override_enabled, "").changed() {
                if override_enabled {
                    settings.override_indent = Some(ui.spacing().indent);
                } else {
                    settings.override_indent = None;
                }
            };
            ui.add_enabled_ui(override_enabled, |ui| {
                let mut override_indent_value =
                    settings.override_indent.unwrap_or(ui.spacing().indent);
                let res = ui.add(
                    egui::DragValue::new(&mut override_indent_value)
                        .clamp_range(0.0..=f32::INFINITY),
                );
                if res.changed() && override_enabled {
                    settings.override_indent = Some(override_indent_value);
                }
            });
        });
        ui.end_row();

        ui.label("Vline style");
        egui::ComboBox::from_id_source("vline style combo box")
            .selected_text(match settings.vline_style {
                VLineStyle::None => "None",
                VLineStyle::VLine => "VLine",
                VLineStyle::Hook => "Hook",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.vline_style, VLineStyle::None, "None");
                ui.selectable_value(&mut settings.vline_style, VLineStyle::VLine, "VLine");
                ui.selectable_value(&mut settings.vline_style, VLineStyle::Hook, "Hook");
            });
        ui.end_row();

        ui.label("Row layout");
        egui::ComboBox::from_id_source("row layout combo box")
            .selected_text(match settings.row_layout {
                RowLayout::Compact => "Compact",
                RowLayout::CompactAlignedLables => "CompactAlignedLables",
                RowLayout::AlignedIcons => "AlignedIcons",
                RowLayout::AlignedIconsAndLabels => "AlignedLabels",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.row_layout, RowLayout::Compact, "Compact");
                ui.selectable_value(
                    &mut settings.row_layout,
                    RowLayout::CompactAlignedLables,
                    "Compact aligned lables",
                );
                ui.selectable_value(
                    &mut settings.row_layout,
                    RowLayout::AlignedIcons,
                    "Aligned icons",
                );
                ui.selectable_value(
                    &mut settings.row_layout,
                    RowLayout::AlignedIconsAndLabels,
                    "Aligned icons and labels",
                );
            });
        ui.end_row();
    });
}

fn show_node_content(ui: &mut Ui, node: &mut Node) {
    egui::Grid::new("settings grid").show(ui, |ui| {
        ui.label("Id");
        ui.label(format!("{:?}", node.id()));
        ui.end_row();

        match node {
            Node::Directory(dir) => {
                ui.label("Name");
                ui.text_edit_singleline(&mut dir.name);
                ui.end_row();

                ui.label("Show custom closer");
                ui.checkbox(&mut dir.custom_closer, "");
                ui.end_row();

                ui.label("Show icon");
                ui.checkbox(&mut dir.icon, "");
                ui.end_row();
            }
            Node::File(file) => {
                ui.label("Name");
                ui.text_edit_singleline(&mut file.name);
                ui.end_row();

                ui.label("Show icon");
                ui.checkbox(&mut file.icon, "");
                ui.end_row();
            }
        }
    });
}
