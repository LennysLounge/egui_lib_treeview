use egui::{
    epaint::{self, RectShape},
    layers::ShapeIdx,
    pos2, vec2, Pos2, Rangef, Rect, Response, Shape, Stroke, Ui, WidgetText,
};

use crate::{
    row::{DropQuarter, Row},
    DragState, DropPosition, NodeInfo, TreeViewSettings, TreeViewState, VLineStyle,
};

#[derive(Clone)]
struct DirectoryState<NodeIdType> {
    /// Id of the directory node.
    id: NodeIdType,
    /// If directory is expanded
    is_open: bool,
    /// Wether dropping on this or any of its child nodes is allowed.
    drop_forbidden: bool,
    /// The rectangle of the row.
    row_rect: Rect,
    /// The rectangle of the icon.
    icon_rect: Rect,
    /// Positions of each child node of this directory.
    child_node_positions: Vec<Pos2>,
    /// The level of indentation.
    indent_level: usize,
    /// If this dir was flattened.
    flattened: bool,
}

/// The builder used to construct the tree view.
///
/// Use this to add directories or leaves to the tree.
pub struct TreeViewBuilder<'ui, NodeIdType>
where
    NodeIdType: Clone,
{
    ui: &'ui mut Ui,
    state: &'ui mut TreeViewState<NodeIdType>,
    stack: Vec<DirectoryState<NodeIdType>>,
    background_idx: ShapeIdx,
    settings: &'ui TreeViewSettings,
}

impl<'ui, NodeIdType> TreeViewBuilder<'ui, NodeIdType>
where
    NodeIdType: Clone + Copy + Send + Sync + std::hash::Hash + PartialEq + Eq + 'static,
{
    pub(crate) fn new(
        ui: &'ui mut Ui,
        state: &'ui mut TreeViewState<NodeIdType>,
        settings: &'ui TreeViewSettings,
    ) -> Self {
        Self {
            background_idx: ui.painter().add(Shape::Noop),
            ui,
            state,
            stack: Vec::new(),
            settings,
        }
    }

    /// Set the selected node id.
    pub fn set_selected(&mut self, id: NodeIdType) {
        self.state.peristant.selected = Some(id);
    }

    /// Add a leaf to the tree.
    pub fn leaf(&mut self, id: NodeIdType, label: impl Into<WidgetText>) {
        let widget_text = label.into();
        self.node(NodeBuilder::leaf(id), |ui| {
            ui.add(egui::Label::new(widget_text.clone()).selectable(false));
        });
    }

    /// Add a directory to the tree.
    /// Must call [Self::close_dir] to close the directory.
    pub fn dir(&mut self, id: NodeIdType, label: impl Into<WidgetText>) {
        let widget_text = label.into();
        self.node(NodeBuilder::dir(id), |ui| {
            ui.add(egui::Label::new(widget_text.clone()).selectable(false));
        });
    }

    /// Close the current directory.
    pub fn close_dir(&mut self) {
        let Some(current_dir) = self.stack.pop() else {
            return;
        };

        // Draw the drop marker over the entire dir if it is the target.
        if let Some((drop_parent, DropPosition::Last)) = &self.state.drop {
            if drop_parent == &current_dir.id {
                let mut rect = current_dir.row_rect;
                *rect.bottom_mut() =
                    self.ui.cursor().top() - self.ui.spacing().item_spacing.y * 0.5;
                self.ui.painter().set(
                    self.state.drop_marker_idx,
                    RectShape::new(
                        rect,
                        self.ui.visuals().widgets.active.rounding,
                        self.ui.visuals().selection.bg_fill.linear_multiply(0.5),
                        Stroke::NONE,
                    ),
                );
            }
        }

        // Draw vline
        if current_dir.is_open {
            let top = current_dir.icon_rect.center_bottom() + vec2(0.0, 2.0);

            let bottom = match self.settings.vline_style {
                VLineStyle::None => top,
                VLineStyle::VLine => pos2(
                    top.x,
                    self.ui.cursor().min.y - self.ui.spacing().item_spacing.y,
                ),
                VLineStyle::Hook => pos2(
                    top.x,
                    current_dir
                        .child_node_positions
                        .last()
                        .map(|pos| pos.y)
                        .unwrap_or(top.y),
                ),
            };
            self.ui.painter().line_segment(
                [top, bottom],
                self.ui.visuals().widgets.noninteractive.bg_stroke,
            );
            if matches!(self.settings.vline_style, VLineStyle::Hook) {
                for child_pos in current_dir.child_node_positions.iter() {
                    let p1 = pos2(top.x, child_pos.y);
                    let p2 = *child_pos + vec2(-2.0, 0.0);
                    self.ui
                        .painter()
                        .line_segment([p1, p2], self.ui.visuals().widgets.noninteractive.bg_stroke);
                }
            }
        }

        // Add child markers to next dir if this one was flattened.
        if current_dir.flattened {
            if let Some(parent_dir) = self.stack.last_mut() {
                parent_dir
                    .child_node_positions
                    .extend(current_dir.child_node_positions);
            }
        }
    }

    /// Add a node to the tree.
    pub fn node(&mut self, node: NodeBuilder<NodeIdType>, mut add_label: impl FnMut(&mut Ui)) {
        let parent_node_id = self.parent_dir().map(|dir| dir.id);
        let depth = self.get_indent_level();
        let node_id = node.id;

        let rect = if node.is_dir {
            self.dir_internal(node, &mut add_label)
        } else {
            self.leaf_internal(node, &mut add_label)
        };

        self.state.node_info.push(NodeInfo {
            depth,
            node_id,
            visible: rect.is_some(),
            rect: rect.unwrap_or(Rect::NOTHING),
            parent_node_id,
        });
    }

    fn leaf_internal(
        &mut self,
        mut node: NodeBuilder<NodeIdType>,
        add_label: &mut dyn FnMut(&mut Ui),
    ) -> Option<Rect> {
        if !self.parent_dir_is_open() {
            return None;
        }
        let row_config = Row {
            id: node.id,
            drop_on_allowed: false,
            is_open: true,
            is_dir: false,
            depth: self.get_indent_level() as f32
                * self
                    .settings
                    .override_indent
                    .unwrap_or(self.ui.spacing().indent),
            is_selected: self.state.is_selected(&node.id),
            is_focused: self.state.has_focus,
        };

        let (row_response, _closer_response) = self.row(
            &row_config,
            add_label,
            node.icon.as_deref_mut(),
            node.closer.as_deref_mut(),
        );

        Some(row_response.rect)
    }

    fn dir_internal(
        &mut self,
        mut node: NodeBuilder<NodeIdType>,
        add_label: &mut dyn FnMut(&mut Ui),
    ) -> Option<Rect> {
        if !self.parent_dir_is_open() {
            self.stack.push(DirectoryState {
                is_open: false,
                id: node.id,
                drop_forbidden: true,
                row_rect: Rect::NOTHING,
                icon_rect: Rect::NOTHING,
                child_node_positions: Vec::new(),
                indent_level: self.get_indent_level(),
                flattened: false,
            });
            return None;
        }
        if node.flatten {
            self.stack.push(DirectoryState {
                is_open: self.parent_dir_is_open(),
                id: node.id,
                drop_forbidden: self.parent_dir_drop_forbidden(),
                row_rect: Rect::NOTHING,
                icon_rect: Rect::NOTHING,
                child_node_positions: Vec::new(),
                indent_level: self.get_indent_level(),
                flattened: true,
            });
            return None;
        }

        let mut open = self
            .state
            .peristant
            .dir_states
            .get(&node.id)
            .copied()
            .unwrap_or(node.default_open);

        let row_config = Row {
            id: node.id,
            drop_on_allowed: node.is_dir,
            is_open: open,
            is_dir: node.is_dir,
            depth: self.get_indent_level() as f32
                * self
                    .settings
                    .override_indent
                    .unwrap_or(self.ui.spacing().indent),
            is_selected: self.state.is_selected(&node.id),
            is_focused: self.state.has_focus,
        };

        let (row_response, closer_response) = self.row(
            &row_config,
            add_label,
            node.icon.as_deref_mut(),
            node.closer.as_deref_mut(),
        );

        let closer_response =
            closer_response.expect("Closer response should be availabel for dirs");

        let closer_interaction = self.state.interact(&closer_response.rect);
        if closer_interaction.clicked {
            open = !open;
            self.state.peristant.selected = Some(node.id);
        }

        let row_interaction = self.state.interact(&row_response.rect);
        if row_interaction.double_clicked {
            open = !open;
        }

        self.state
            .peristant
            .dir_states
            .entry(node.id)
            .and_modify(|e| *e = open)
            .or_insert(open);

        self.stack.push(DirectoryState {
            is_open: open,
            id: node.id,
            drop_forbidden: self.parent_dir_drop_forbidden() || self.state.is_dragged(&node.id),
            row_rect: row_response.rect,
            icon_rect: closer_response.rect,
            child_node_positions: Vec::new(),
            indent_level: self.get_indent_level() + 1,
            flattened: false,
        });
        Some(row_response.rect)
    }

    fn row(
        &mut self,
        row_config: &Row<NodeIdType>,
        mut add_label: impl FnMut(&mut Ui),
        mut add_icon: Option<&mut AddIcon<'_>>,
        mut add_closer: Option<&mut AddCloser<'_>>,
    ) -> (Response, Option<Response>) {
        let (row_response, closer_response, label_rect) = row_config.draw_row(
            self.ui,
            self.state,
            self.settings,
            &mut add_label,
            &mut add_icon,
            &mut add_closer,
        );

        let row_interaction = self.state.interact(&row_response.rect);

        if row_interaction.clicked {
            self.state.peristant.selected = Some(row_config.id);
        }
        if self.state.is_selected(&row_config.id) {
            self.ui.painter().set(
                self.background_idx,
                epaint::RectShape::new(
                    row_response.rect,
                    self.ui.visuals().widgets.active.rounding,
                    if self.state.has_focus {
                        self.ui.visuals().selection.bg_fill
                    } else {
                        self.ui
                            .visuals()
                            .widgets
                            .inactive
                            .weak_bg_fill
                            .linear_multiply(0.3)
                    },
                    Stroke::NONE,
                ),
            );
        }
        if row_interaction.drag_started {
            let pointer_pos = self.ui.ctx().pointer_latest_pos().unwrap_or_default();
            self.state.peristant.dragged = Some(DragState {
                node_id: row_config.id,
                drag_row_offset: row_response.rect.min - pointer_pos,
                drag_start_pos: pointer_pos,
                drag_valid: false,
            });
        }
        if let Some(drag_state) = self.state.peristant.dragged.as_mut() {
            // Test if the drag becomes valid
            if !drag_state.drag_valid {
                drag_state.drag_valid = drag_state
                    .drag_start_pos
                    .distance(self.ui.ctx().pointer_latest_pos().unwrap_or_default())
                    > 5.0;
            }
            if drag_state.node_id == row_config.id && drag_state.drag_valid {
                row_config.draw_row_dragged(
                    self.ui,
                    self.settings,
                    self.state,
                    &mut add_label,
                    &mut add_icon,
                    &mut add_closer,
                );
            }
        }
        if let Some(drop_quarter) = self
            .state
            .interaction_response
            .hover_pos()
            .and_then(|pos| DropQuarter::new(row_response.rect.y_range(), pos.y))
        {
            self.do_drop(row_config, &row_response, drop_quarter);
        }

        self.push_child_node_position(label_rect.left_center());

        (row_response, closer_response)
    }

    fn do_drop(
        &mut self,
        row_config: &Row<NodeIdType>,
        row_response: &Response,
        drop_quarter: DropQuarter,
    ) {
        if !self.ui.ctx().memory(|m| m.is_anything_being_dragged()) {
            return;
        }
        if self.state.peristant.dragged.is_none() {
            return;
        }
        if !self.state.drag_valid() {
            return;
        }
        if self.parent_dir_drop_forbidden() {
            return;
        }
        // For dirs and for nodes that allow dropping on them, it is not
        // allowed to drop itself onto itself.
        if self.state.is_dragged(&row_config.id) && row_config.drop_on_allowed {
            return;
        }

        let drop_position = self.get_drop_position(row_config, &drop_quarter);
        let shape = self.drop_marker_shape(row_response, drop_position.as_ref());

        // It is allowed to drop itself `After´ or `Before` itself.
        // This however doesn't make sense and makes executing the command more
        // difficult for the caller.
        // Instead we display the markers only.
        if self.state.is_dragged(&row_config.id) {
            self.ui.painter().set(self.state.drop_marker_idx, shape);
            return;
        }

        self.state.drop = drop_position;
        self.ui.painter().set(self.state.drop_marker_idx, shape);
    }

    fn get_drop_position(
        &self,
        node_config: &Row<NodeIdType>,
        drop_quater: &DropQuarter,
    ) -> Option<(NodeIdType, DropPosition<NodeIdType>)> {
        let Row {
            id,
            drop_on_allowed,
            is_open,
            ..
        } = node_config;

        match drop_quater {
            DropQuarter::Top => {
                if let Some(parent_dir) = self.parent_dir() {
                    return Some((parent_dir.id, DropPosition::Before(*id)));
                }
                if *drop_on_allowed {
                    return Some((*id, DropPosition::Last));
                }
                None
            }
            DropQuarter::MiddleTop => {
                if *drop_on_allowed {
                    return Some((*id, DropPosition::Last));
                }
                if let Some(parent_dir) = self.parent_dir() {
                    return Some((parent_dir.id, DropPosition::Before(*id)));
                }
                None
            }
            DropQuarter::MiddleBottom => {
                if *drop_on_allowed {
                    return Some((*id, DropPosition::Last));
                }
                if let Some(parent_dir) = self.parent_dir() {
                    return Some((parent_dir.id, DropPosition::After(*id)));
                }
                None
            }
            DropQuarter::Bottom => {
                if *drop_on_allowed && *is_open {
                    return Some((*id, DropPosition::First));
                }
                if let Some(parent_dir) = self.parent_dir() {
                    return Some((parent_dir.id, DropPosition::After(*id)));
                }
                if *drop_on_allowed {
                    return Some((*id, DropPosition::Last));
                }
                None
            }
        }
    }

    fn drop_marker_shape(
        &self,
        interaction: &Response,
        drop_position: Option<&(NodeIdType, DropPosition<NodeIdType>)>,
    ) -> Shape {
        pub const DROP_LINE_HEIGHT: f32 = 3.0;

        let drop_marker = match drop_position {
            Some((_, DropPosition::Before(_))) => {
                Rangef::point(interaction.rect.min.y).expand(DROP_LINE_HEIGHT * 0.5)
            }
            Some((_, DropPosition::First)) | Some((_, DropPosition::After(_))) => {
                Rangef::point(interaction.rect.max.y).expand(DROP_LINE_HEIGHT * 0.5)
            }
            Some((_, DropPosition::Last)) => interaction.rect.y_range(),
            None => return Shape::Noop,
        };

        epaint::RectShape::new(
            Rect::from_x_y_ranges(interaction.rect.x_range(), drop_marker),
            self.ui.visuals().widgets.active.rounding,
            self.ui
                .style()
                .visuals
                .selection
                .bg_fill
                .linear_multiply(0.6),
            Stroke::NONE,
        )
        .into()
    }

    fn parent_dir(&self) -> Option<&DirectoryState<NodeIdType>> {
        if self.stack.is_empty() {
            None
        } else {
            self.stack.last()
        }
    }
    fn parent_dir_is_open(&self) -> bool {
        self.parent_dir().map_or(true, |dir| dir.is_open)
    }

    fn parent_dir_drop_forbidden(&self) -> bool {
        self.parent_dir().is_some_and(|dir| dir.drop_forbidden)
    }

    fn push_child_node_position(&mut self, pos: Pos2) {
        if let Some(parent_dir) = self.stack.last_mut() {
            parent_dir.child_node_positions.push(pos);
        }
    }
    fn get_indent_level(&self) -> usize {
        self.stack.last().map(|d| d.indent_level).unwrap_or(0)
    }
}

pub type AddIcon<'icon> = dyn FnMut(&mut Ui) + 'icon;
pub type AddCloser<'closer> = dyn FnMut(&mut Ui, CloserState) + 'closer;

pub struct NodeBuilder<'icon, 'closer, NodeIdType> {
    id: NodeIdType,
    is_dir: bool,
    flatten: bool,
    default_open: bool,
    icon: Option<Box<AddIcon<'icon>>>,
    closer: Option<Box<AddCloser<'closer>>>,
}
impl<'icon, 'closer, NodeIdType> NodeBuilder<'icon, 'closer, NodeIdType> {
    /// Create a new node builder from a leaf prototype.
    pub fn leaf(id: NodeIdType) -> Self {
        Self {
            id,
            is_dir: false,
            flatten: false,
            icon: None,
            closer: None,
            default_open: true,
        }
    }

    /// Create a new node builder from a directory prorotype.
    pub fn dir(id: NodeIdType) -> Self {
        Self {
            id,
            is_dir: true,
            flatten: false,
            icon: None,
            closer: None,
            default_open: true,
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
}

/// State of the closer when it is drawn.
pub struct CloserState {
    /// Wether the current directory this closer represents is currently open or closed.
    pub is_open: bool,
    /// Wether the pointer is hovering over the closer.
    pub is_hovered: bool,
}
