mod data;
use std::env;

use data::*;
use egui::{Color32, DragValue, Id, Label, Layout, Response, Ui};
use egui_ltreeview::{node::NodeBuilder, Action, RowLayout, TreeView, TreeViewBuilder, VLineStyle};
use uuid::Uuid;

fn main() -> Result<(), eframe::Error> {
    env::set_var("RUST_BACKTRACE", "1");
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        //viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Egui_ltreeview example",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
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
    layout_h_justify: bool,
    layout_v_justify: bool,
    override_indent: Option<f32>,
    vline_style: VLineStyle,
    row_layout: RowLayout,
    fill_space_horizontal: bool,
    fill_space_vertical: bool,
    max_width_enabled: bool,
    max_width: f32,
    max_height_enabled: bool,
    max_height: f32,
    min_width_enabled: bool,
    min_width: f32,
    min_height_enabled: bool,
    min_height: f32,
    show_size: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tree: make_tree(),
            settings_id: Uuid::new_v4(),
            selected_node: None,
            settings: Settings {
                row_layout: RowLayout::CompactAlignedLables,
                fill_space_horizontal: true,
                fill_space_vertical: false,
                max_width: 100.0,
                max_height: 100.0,
                show_size: true,
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
                ui.set_min_width(ui.available_width());
                ui.with_layout(
                    Layout::top_down(egui::Align::Min)
                        .with_main_justify(self.settings.layout_v_justify)
                        .with_cross_justify(self.settings.layout_h_justify),
                    |ui| {
                        show_tree_view(ui, self);
                    },
                );
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

fn show_tree_view(ui: &mut Ui, app: &mut MyApp) -> Response {
    let response = TreeView::new(ui.make_persistent_id("Names tree view"))
        .override_indent(app.settings.override_indent)
        .vline_style(app.settings.vline_style)
        .row_layout(app.settings.row_layout)
        .fill_space_horizontal(app.settings.fill_space_horizontal)
        .fill_space_vertical(app.settings.fill_space_vertical)
        .max_width(if app.settings.max_width_enabled {
            app.settings.max_width
        } else {
            f32::INFINITY
        })
        .max_height(if app.settings.max_height_enabled {
            app.settings.max_height
        } else {
            f32::INFINITY
        })
        .min_width(if app.settings.min_width_enabled {
            app.settings.min_width
        } else {
            0.0
        })
        .min_height(if app.settings.min_height_enabled {
            app.settings.min_height
        } else {
            0.0
        })
        .show(ui, |mut builder| {
            builder.node(NodeBuilder::dir(Uuid::default()).flatten(true));
            //builder.set_root_id(Uuid::default());
            builder.node(
                NodeBuilder::leaf(app.settings_id)
                    .icon(|ui| {
                        egui::Image::new(egui::include_image!("settings.png"))
                            .tint(ui.visuals().widgets.noninteractive.fg_stroke.color)
                            .paint_at(ui, ui.max_rect());
                    })
                    .label(|ui| {
                        ui.add(Label::new("Settings").selectable(false));
                    }),
            );
            show_node(&mut builder, &app.tree);
            builder.close_dir();
        });
    for action in response.actions.iter() {
        match action {
            Action::SetSelected(id) => app.selected_node = *id,
            Action::Move {
                source,
                target,
                position,
            } => {
                if let Some(source) = app.tree.remove(source) {
                    _ = app.tree.insert(target, *position, source);
                }
            }
            Action::Drag { .. } => (),
        }
    }
    if app.settings.show_size {
        ui.painter()
            .rect_stroke(response.response.rect, 0.0, (1.0, Color32::BLACK));
    }
    response.response
}

fn show_node(builder: &mut TreeViewBuilder<Uuid>, node: &Node) {
    match node {
        Node::Directory(dir) => show_dir(builder, dir),
        Node::File(file) => show_file(builder, file),
    }
}
fn show_dir(builder: &mut TreeViewBuilder<Uuid>, dir: &Directory) {
    let mut node = NodeBuilder::dir(dir.id)
        .label(|ui| {
            ui.add(Label::new(&dir.name).selectable(false));
        })
        .context_menu(|ui| {
            ui.label("dir:");
            ui.label(&dir.name);
        });
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
    builder.node(node);

    for node in dir.children.iter() {
        show_node(builder, node);
    }

    builder.close_dir();
}
fn show_file(builder: &mut TreeViewBuilder<Uuid>, file: &File) {
    let mut node = NodeBuilder::leaf(file.id)
        .label(|ui| {
            ui.add(Label::new(&file.name).selectable(false));
        })
        .context_menu(|ui| {
            ui.label("file:");
            ui.label(&file.name);
        });
    if file.icon {
        node = node.icon(|ui| {
            egui::Image::new(egui::include_image!("user.png"))
                .tint(ui.visuals().widgets.noninteractive.fg_stroke.color)
                .paint_at(ui, ui.max_rect());
        });
    }
    builder.node(node);
}

fn show_settings(ui: &mut Ui, settings: &mut Settings) {
    egui::Grid::new("settings grid").show(ui, |ui| {
        ui.strong("Egui:");
        ui.end_row();

        ui.label("Indent:");
        let mut indent = ui.ctx().style().spacing.indent;
        ui.add(DragValue::new(&mut indent).range(0.0..=f32::INFINITY));
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
        ui.label("Layout h justify:");
        ui.checkbox(&mut settings.layout_h_justify, "");
        ui.end_row();
        ui.label("Layout v justify:");
        ui.checkbox(&mut settings.layout_v_justify, "");
        ui.end_row();
        ui.label("Show size:");
        ui.checkbox(&mut settings.show_size, "");
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
                    egui::DragValue::new(&mut override_indent_value).range(0.0..=f32::INFINITY),
                );
                if res.changed() && override_enabled {
                    settings.override_indent = Some(override_indent_value);
                }
            });
        });
        ui.end_row();

        ui.label("Vline style");
        egui::ComboBox::from_id_salt("vline style combo box")
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
        egui::ComboBox::from_id_salt("row layout combo box")
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

        ui.label("fill horizontal");
        ui.checkbox(&mut settings.fill_space_horizontal, "");
        ui.end_row();

        ui.label("fill vertical");
        ui.checkbox(&mut settings.fill_space_vertical, "");
        ui.end_row();

        ui.label("max width");
        ui.horizontal(|ui| {
            ui.checkbox(&mut settings.max_width_enabled, "");
            ui.add_enabled(
                settings.max_width_enabled,
                egui::DragValue::new(&mut settings.max_width).range(0.0..=f32::INFINITY),
            );
        });
        ui.end_row();

        ui.label("max height");
        ui.horizontal(|ui| {
            ui.checkbox(&mut settings.max_height_enabled, "");
            ui.add_enabled(
                settings.max_height_enabled,
                egui::DragValue::new(&mut settings.max_height).range(0.0..=f32::INFINITY),
            );
        });
        ui.end_row();

        ui.label("min width");
        ui.horizontal(|ui| {
            ui.checkbox(&mut settings.min_width_enabled, "");
            ui.add_enabled(
                settings.min_width_enabled,
                egui::DragValue::new(&mut settings.min_width).range(0.0..=f32::INFINITY),
            );
        });
        ui.end_row();

        ui.label("min height");
        ui.horizontal(|ui| {
            ui.checkbox(&mut settings.min_height_enabled, "");
            ui.add_enabled(
                settings.min_height_enabled,
                egui::DragValue::new(&mut settings.min_height).range(0.0..=f32::INFINITY),
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
