use egui::{
    emath, epaint, remap, vec2, CursorIcon, Id, InnerResponse, LayerId, Order, Rangef, Rect, Shape,
    Stroke, Ui, Vec2,
};

use crate::{Interaction, RowLayout, TreeViewSettings, TreeViewState};

pub type AddIcon<'icon> = dyn FnMut(&mut Ui) + 'icon;
pub type AddCloser<'closer> = dyn FnMut(&mut Ui, CloserState) + 'closer;

pub struct NodeBuilder<'icon, 'closer, NodeIdType> {
    pub(crate) id: NodeIdType,
    pub(crate) is_dir: bool,
    pub(crate) flatten: bool,
    pub(crate) is_open: bool,
    pub(crate) default_open: bool,
    pub(crate) drop_allowed: bool,
    indent: usize,
    icon: Option<Box<AddIcon<'icon>>>,
    closer: Option<Box<AddCloser<'closer>>>,
}
impl<'icon, 'closer, NodeIdType> NodeBuilder<'icon, 'closer, NodeIdType>
where
    NodeIdType: Clone + std::hash::Hash,
{
    /// Create a new node builder from a leaf prototype.
    pub fn leaf(id: NodeIdType) -> Self {
        Self {
            id,
            is_dir: false,
            flatten: false,
            drop_allowed: false,
            icon: None,
            closer: None,
            is_open: false,
            default_open: true,
            indent: 0,
        }
    }

    /// Create a new node builder from a directory prorotype.
    pub fn dir(id: NodeIdType) -> Self {
        Self {
            id,
            is_dir: true,
            flatten: false,
            drop_allowed: true,
            icon: None,
            closer: None,
            is_open: false,
            default_open: true,
            indent: 0,
        }
    }

    /// Whether or not the directory should be flattened into the
    /// parent directiron. A directory that is flattened does not
    /// show a label and cannot be navigated to. Its children appear
    /// like the children of the grand parent directory.
    pub fn flatten(mut self, flatten: bool) -> Self {
        self.flatten = flatten;
        self
    }

    /// Whether or not a directory should be open by default or closed.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Whether or not dropping onto this node is allowed.
    pub fn drop_allowed(mut self, drop_allowed: bool) -> Self {
        self.drop_allowed = drop_allowed;
        self
    }

    /// Add a icon to the node.
    pub fn icon<'new_icon>(
        self,
        add_icon: impl FnMut(&mut Ui) + 'new_icon,
    ) -> NodeBuilder<'new_icon, 'closer, NodeIdType> {
        NodeBuilder {
            icon: Some(Box::new(add_icon)),
            ..self
        }
    }

    /// Add a custom closer to the directory node.
    /// Leaves do not show a closer.
    pub fn closer<'new_closer>(
        self,
        add_closer: impl FnMut(&mut Ui, CloserState) + 'new_closer,
    ) -> NodeBuilder<'icon, 'new_closer, NodeIdType> {
        NodeBuilder {
            closer: Some(Box::new(add_closer)),
            ..self
        }
    }

    pub(crate) fn set_is_open(&mut self, open: bool) {
        self.is_open = open;
    }

    pub(crate) fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
    }

    pub(crate) fn show_node(
        &mut self,
        ui: &mut Ui,
        add_label: &mut dyn FnMut(&mut Ui),
        state: &TreeViewState<NodeIdType>,
        settings: &TreeViewSettings,
    ) -> (Rect, Option<Rect>, Option<Rect>, Rect) {
        let (reserve_closer, draw_closer, reserve_icon, draw_icon) = match settings.row_layout {
            RowLayout::Compact => (self.is_dir, self.is_dir, false, false),
            RowLayout::CompactAlignedLables => (
                self.is_dir,
                self.is_dir,
                !self.is_dir,
                !self.is_dir && self.icon.is_some(),
            ),
            RowLayout::AlignedIcons => {
                (true, self.is_dir, self.icon.is_some(), self.icon.is_some())
            }
            RowLayout::AlignedIconsAndLabels => (true, self.is_dir, true, self.icon.is_some()),
        };

        let InnerResponse {
            inner: (closer, icon, label),
            response: row_response,
        } = ui.horizontal(|ui| {
            // The layouting in the row has to be pretty tight so we tunr of the item spacing here.
            let original_item_spacing = ui.spacing().item_spacing;
            ui.spacing_mut().item_spacing = Vec2::ZERO;

            ui.add_space(original_item_spacing.x);

            // Add a little space so the closer/icon/label doesnt touch the left side
            // and add the indentation space.
            ui.add_space(ui.spacing().item_spacing.x);
            ui.add_space(
                self.indent as f32 * settings.override_indent.unwrap_or(ui.spacing().indent),
            );

            // Draw the closer
            let closer = draw_closer.then(|| {
                let (small_rect, big_rect) = ui
                    .spacing()
                    .icon_rectangles(ui.available_rect_before_wrap());

                let res = ui.allocate_ui_at_rect(big_rect, |ui| {
                    let closer_interaction = state.interact(&ui.max_rect());
                    if closer_interaction.hovered {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                    if let Some(add_closer) = self.closer.as_mut() {
                        (add_closer)(
                            ui,
                            CloserState {
                                is_open: self.is_open,
                                is_hovered: closer_interaction.hovered,
                            },
                        );
                    } else {
                        let icon_id = Id::new(&self.id).with("tree view closer icon");
                        let openness = ui.ctx().animate_bool(icon_id, self.is_open);
                        let closer_interaction = state.interact(&ui.max_rect());
                        paint_default_icon(ui, openness, &small_rect, &closer_interaction);
                    }
                    ui.allocate_space(ui.available_size_before_wrap());
                });
                res.response.rect
            });
            if closer.is_none() && reserve_closer {
                ui.add_space(ui.spacing().icon_width);
            }

            // Draw icon
            let icon = draw_icon
                .then(|| {
                    self.icon.as_mut().map(|add_icon| {
                        let (_, big_rect) = ui
                            .spacing()
                            .icon_rectangles(ui.available_rect_before_wrap());
                        ui.allocate_ui_at_rect(big_rect, |ui| {
                            ui.set_min_size(big_rect.size());
                            add_icon(ui);
                        })
                        .response
                        .rect
                    })
                })
                .flatten();
            if icon.is_none() && reserve_icon {
                ui.add_space(ui.spacing().icon_width);
            }

            ui.add_space(2.0);
            // Draw icon
            let label = ui
                .scope(|ui| {
                    ui.spacing_mut().item_spacing = original_item_spacing;
                    add_label(ui);
                })
                .response
                .rect;

            ui.add_space(original_item_spacing.x);

            (closer, icon, label)
        });

        let mut row = row_response
            .rect
            .expand2(vec2(0.0, ui.spacing().item_spacing.y * 0.5));
        row.set_width(ui.available_width());

        (row, closer, icon, label)
    }

    /// Draw the content as a drag overlay if it is beeing dragged.
    pub(crate) fn show_node_dragged(
        &mut self,
        ui: &mut Ui,
        add_label: &mut dyn FnMut(&mut Ui),
        state: &TreeViewState<NodeIdType>,
        settings: &TreeViewSettings,
    ) -> bool {
        ui.ctx().set_cursor_icon(CursorIcon::Alias);

        let drag_source_id = ui.make_persistent_id("Drag source");

        // Paint the content to a new layer for the drag overlay.
        let layer_id = LayerId::new(Order::Tooltip, drag_source_id);

        let background_rect = ui
            .child_ui(ui.available_rect_before_wrap(), *ui.layout())
            .with_layer_id(layer_id, |ui| {
                let background_position = ui.painter().add(Shape::Noop);

                let (row, _, _, _) = self.show_node(ui, add_label, state, settings);

                ui.painter().set(
                    background_position,
                    epaint::RectShape::new(
                        row,
                        ui.visuals().widgets.active.rounding,
                        ui.visuals().selection.bg_fill.linear_multiply(0.4),
                        Stroke::NONE,
                    ),
                );
                row
            })
            .inner;

        // Move layer to the drag position
        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            //let delta = -background_rect.min.to_vec2() + pointer_pos.to_vec2() + drag_offset;
            let delta = -background_rect.min.to_vec2()
                + pointer_pos.to_vec2()
                + state.peristant.dragged.as_ref().unwrap().drag_row_offset;
            ui.ctx().translate_layer(layer_id, delta);
        }

        true
    }
}

/// Paint the arrow icon that indicated if the region is open or not
pub(crate) fn paint_default_icon(
    ui: &mut Ui,
    openness: f32,
    rect: &Rect,
    interaction: &Interaction,
) {
    let visuals = if interaction.hovered {
        ui.visuals().widgets.hovered
    } else {
        ui.visuals().widgets.inactive
    };

    // Draw a pointy triangle arrow:
    let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation = emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        Stroke::NONE,
    ));
}

pub enum DropQuarter {
    Top,
    MiddleTop,
    MiddleBottom,
    Bottom,
}

impl DropQuarter {
    pub fn new(range: Rangef, cursor_pos: f32) -> Option<DropQuarter> {
        pub const DROP_LINE_HOVER_HEIGHT: f32 = 5.0;

        let h0 = range.min;
        let h1 = range.min + DROP_LINE_HOVER_HEIGHT;
        let h2 = (range.min + range.max) / 2.0;
        let h3 = range.max - DROP_LINE_HOVER_HEIGHT;
        let h4 = range.max;

        match cursor_pos {
            y if y >= h0 && y < h1 => Some(Self::Top),
            y if y >= h1 && y < h2 => Some(Self::MiddleTop),
            y if y >= h2 && y < h3 => Some(Self::MiddleBottom),
            y if y >= h3 && y < h4 => Some(Self::Bottom),
            _ => None,
        }
    }
}

/// State of the closer when it is drawn.
pub struct CloserState {
    /// Wether the current directory this closer represents is currently open or closed.
    pub is_open: bool,
    /// Wether the pointer is hovering over the closer.
    pub is_hovered: bool,
}