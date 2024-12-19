pub mod builder;
pub mod node;

use std::hash::Hash;

use egui::{
    self, layers::ShapeIdx, vec2, Event, EventFilter, Id, Key, Layout, NumExt, Pos2, Rect,
    Response, Sense, Shape, Ui, Vec2,
};

pub use builder::TreeViewBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait TreeViewId: Clone + Copy + PartialEq + Eq + Hash {}
impl<T> TreeViewId for T where T: Clone + Copy + PartialEq + Eq + Hash {}

/// Represents the state of the tree view.
///
/// This holds which node is selected and the open/close
/// state of the directories.
#[derive(Clone, Serialize, Deserialize)]
pub struct TreeViewState<NodeIdType> {
    /// Id of the node that was selected.
    selected: Option<NodeIdType>,
    /// Information about the dragged node.
    dragged: Option<DragState<NodeIdType>>,
    /// Id of the node that was right clicked.
    secondary_selection: Option<NodeIdType>,
    /// The rectangle the tree view occupied.
    size: Vec2,
    /// Open states of the dirs in this tree.
    node_states: Vec<NodeState<NodeIdType>>,
}
impl<NodeIdType> Default for TreeViewState<NodeIdType> {
    fn default() -> Self {
        Self {
            selected: Default::default(),
            dragged: Default::default(),
            secondary_selection: Default::default(),
            size: Vec2::ZERO,
            node_states: Vec::new(),
        }
    }
}
impl<NodeIdType: TreeViewId> TreeViewState<NodeIdType> {
    /// Return the selected node if any is selected.
    pub fn selected(&self) -> Option<NodeIdType> {
        self.selected
    }

    /// Set the selected node for this tree.
    /// If [`None`] then no node is selected.
    pub fn set_selected(&mut self, selected: Option<NodeIdType>) {
        self.selected = selected;
    }

    /// Expand all parent nodes of the node with the given id.
    pub fn expand_parents_of(&mut self, id: NodeIdType, include_self: bool) {
        let mut current_node = if include_self {
            Some(id)
        } else {
            self.node_state_of(&id)
                .and_then(|node_state| node_state.parent_id)
        };

        while let Some(node_id) = &current_node {
            if let Some(node_state) = self.node_state_of_mut(node_id) {
                node_state.open = true;
                current_node = node_state.parent_id;
            } else {
                current_node = None;
            }
        }
    }

    /// Get the parent id of a node.
    pub fn parent_id_of(&self, id: NodeIdType) -> Option<NodeIdType> {
        self.node_state_of(&id)
            .and_then(|node_state| node_state.parent_id)
    }

    /// Get the node state for an id.
    pub(crate) fn node_state_of(&self, id: &NodeIdType) -> Option<&NodeState<NodeIdType>> {
        self.node_states.iter().find(|ns| &ns.id == id)
    }
    /// Get the node state for an id.
    pub(crate) fn node_state_of_mut(
        &mut self,
        id: &NodeIdType,
    ) -> Option<&mut NodeState<NodeIdType>> {
        self.node_states.iter_mut().find(|ns| &ns.id == id)
    }
}

impl<NodeIdType> TreeViewState<NodeIdType>
where
    NodeIdType: Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    pub fn load(ui: &mut Ui, id: Id) -> Option<Self> {
        ui.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ui: &mut Ui, id: Id) {
        ui.data_mut(|d| d.insert_persisted(id, self));
    }
}
/// State of the dragged node.
#[derive(Clone, Serialize, Deserialize)]
struct DragState<NodeIdType> {
    /// Id of the dragged node.
    pub node_id: NodeIdType,
    /// Offset of the drag overlay to the pointer.
    pub drag_row_offset: Vec2,
    /// Position of the pointer when the drag started.
    pub drag_start_pos: Pos2,
    /// A drag only becomes valid after it has been dragged for
    /// a short distance.
    pub drag_valid: bool,
}
/// State of each node in the tree.
#[derive(Clone, Serialize, Deserialize)]
struct NodeState<NodeIdType> {
    /// Id of this node.
    id: NodeIdType,
    /// The parent node of this node.
    parent_id: Option<NodeIdType>,
    /// Wether the node is open or not.
    open: bool,
    /// Wether the node is visible or not.
    visible: bool,
}

pub struct TreeView {
    id: Id,
    settings: TreeViewSettings,
}
impl TreeView {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            settings: TreeViewSettings::default(),
        }
    }

    /// Override the indent value from the current ui style with this value.
    ///
    /// If `None`, the value of the current ui style is used.
    /// Defaults to `None`.
    pub fn override_indent(mut self, indent: Option<f32>) -> Self {
        self.settings.override_indent = indent;
        self
    }

    /// Set the style of the vline to show the indentation level.
    pub fn vline_style(mut self, style: VLineStyle) -> Self {
        self.settings.vline_style = style;
        self
    }

    /// Set the row layout for this tree.
    pub fn row_layout(mut self, layout: RowLayout) -> Self {
        self.settings.row_layout = layout;
        self
    }

    /// Set whether or not the tree should fill all available horizontal space.
    ///
    /// If the tree is part of a horizontally justified layout, this property has no
    /// effect and the tree will always fill horizontal space.
    ///
    /// Default is `true`.
    pub fn fill_space_horizontal(mut self, fill_space_horizontal: bool) -> Self {
        self.settings.fill_space_horizontal = fill_space_horizontal;
        self
    }

    /// Set whether or not the tree should fill all available vertical space.
    ///
    /// If the tree is part of a vertically justified layout, this property has no
    /// effect and the tree will always fill vertical space.
    ///
    /// Default is `false`.
    pub fn fill_space_vertical(mut self, fill_space_vertical: bool) -> Self {
        self.settings.fill_space_vertical = fill_space_vertical;
        self
    }

    /// Set the maximum width the tree can have.
    ///
    /// If the tree is part of a horizontally justified layout, this property has no
    /// effect and the tree will always fill the available horizontal space.
    pub fn max_width(mut self, width: f32) -> Self {
        self.settings.max_width = width;
        self
    }

    /// Set the maximum hight the tree can have.
    ///
    /// If the tree is part of a vertical justified layout, this property has no
    /// effect and the tree will always fill the available vertical space.
    pub fn max_height(mut self, height: f32) -> Self {
        self.settings.max_height = height;
        self
    }

    /// Set the minimum width the tree can have.
    pub fn min_width(mut self, width: f32) -> Self {
        self.settings.min_width = width;
        self
    }

    /// Set the minimum hight the tree can have.
    pub fn min_height(mut self, height: f32) -> Self {
        self.settings.min_height = height;
        self
    }

    /// Start displaying the tree view.
    ///
    /// Construct the tree view using the [`TreeViewBuilder`] by adding
    /// directories or leaves to the tree.
    pub fn show<NodeIdType>(
        self,
        ui: &mut Ui,
        build_tree_view: impl FnMut(TreeViewBuilder<'_, '_, NodeIdType>),
    ) -> TreeViewResponse<NodeIdType>
    where
        NodeIdType: TreeViewId + Send + Sync + 'static + Serialize + DeserializeOwned,
    {
        let id = self.id;
        let mut state = TreeViewState::load(ui, id).unwrap_or_default();
        let res = self.show_state(ui, &mut state, build_tree_view);
        state.store(ui, id);
        res
    }

    /// Start displaying the tree view with a [`TreeViewState`].
    ///
    /// Construct the tree view using the [`TreeViewBuilder`] by addind
    /// directories or leaves to the tree.
    pub fn show_state<NodeIdType>(
        mut self,
        ui: &mut Ui,
        state: &mut TreeViewState<NodeIdType>,
        mut build_tree_view: impl FnMut(TreeViewBuilder<'_, '_, NodeIdType>),
    ) -> TreeViewResponse<NodeIdType>
    where
        NodeIdType: TreeViewId + Send + Sync + 'static,
    {
        // Justified layouts override these settings
        if ui.layout().horizontal_justify() {
            self.settings.fill_space_horizontal = true;
            self.settings.max_width = f32::INFINITY;
        }
        if ui.layout().vertical_justify() {
            self.settings.fill_space_vertical = true;
            self.settings.max_height = f32::INFINITY;
        }

        // Set the focus filter to get correct keyboard navigation while focused.
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                self.id,
                EventFilter {
                    tab: false,
                    escape: false,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                },
            )
        });

        // Create the tree state by loading the previous frame and setting up the state.
        let mut data = TreeViewData::new(ui, state, self.id);
        let prev_selection = data.peristant.selected;

        // Calculate the desired size of the tree view widget.
        let size = vec2(
            if self.settings.fill_space_horizontal {
                ui.available_width().at_most(self.settings.max_width)
            } else {
                data.peristant.size.x.at_most(self.settings.max_width)
            }
            .at_least(self.settings.min_width),
            if self.settings.fill_space_vertical {
                ui.available_height().at_most(self.settings.max_height)
            } else {
                data.peristant.size.y.at_most(self.settings.max_height)
            }
            .at_least(self.settings.min_height),
        );

        // Run the build tree view closure
        let used_rect = ui
            .allocate_ui_with_layout(size, Layout::top_down(egui::Align::Min), |ui| {
                ui.set_min_size(vec2(self.settings.min_width, self.settings.min_height));
                ui.add_space(ui.spacing().item_spacing.y * 0.5);
                build_tree_view(TreeViewBuilder::new(ui, &mut data, &self.settings));
                // Add negative space because the place will add the item spacing on top of this.
                ui.add_space(-ui.spacing().item_spacing.y * 0.5);

                if self.settings.fill_space_horizontal {
                    ui.set_min_width(ui.available_width());
                }
                if self.settings.fill_space_vertical {
                    ui.set_min_height(ui.available_height());
                }
            })
            .response
            .rect;

        // use new node states
        data.peristant.node_states = data.new_node_states.clone();

        // If the tree was clicked it should receive focus.
        let tree_view_interact = data.interact(&used_rect);
        if tree_view_interact.clicked || tree_view_interact.drag_started {
            ui.memory_mut(|m| m.request_focus(self.id));
        }

        if ui.memory(|m| m.has_focus(self.id)) {
            // If the widget is focused but no node is selected we want to select any node
            // to allow navigating throught the tree.
            // In case we gain focus from a drag action we select the dragged node directly.
            if data.peristant.selected.is_none() {
                data.peristant.selected = data
                    .peristant
                    .dragged
                    .as_ref()
                    .map(|drag_state| drag_state.node_id)
                    .or(data.peristant.node_states.first().map(|n| n.id));
            }
            ui.input(|i| {
                for event in i.events.iter() {
                    match event {
                        Event::Key { key, pressed, .. } if *pressed => {
                            handle_input(data.peristant, key)
                        }
                        _ => (),
                    }
                }
            });
        }
        // Update the drag state
        // A drag only becomes a valid drag after the pointer has traveled some distance.
        if let Some(drag_state) = data.peristant.dragged.as_mut() {
            if !drag_state.drag_valid {
                drag_state.drag_valid = drag_state
                    .drag_start_pos
                    .distance(ui.ctx().pointer_latest_pos().unwrap_or_default())
                    > 5.0;
            }
        }

        // Create a drag or move action.
        if data.drag_valid() {
            if let Some((drag_state, (drop_id, position))) =
                data.peristant.dragged.as_ref().zip(data.drop)
            {
                if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                    data.actions.push(Action::Move {
                        source: drag_state.node_id,
                        target: drop_id,
                        position,
                    })
                } else {
                    data.actions.push(Action::Drag {
                        source: drag_state.node_id,
                        target: drop_id,
                        position,
                    })
                }
            }
        }
        // Create a selection action.
        if data.peristant.selected != prev_selection {
            data.actions
                .push(Action::SetSelected(data.peristant.selected));
        }

        // Reset the drag state.
        if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
            data.peristant.dragged = None;
        }

        // Remember the size of the tree for next frame.
        data.peristant.size = used_rect.size();

        TreeViewResponse {
            response: data.interaction_response,
            drop_marker_idx: data.drop_marker_idx,
            actions: data.actions,
        }
    }
}

fn handle_input<NodeIdType: TreeViewId>(state: &mut TreeViewState<NodeIdType>, key: &Key) {
    let Some(selected_id) = &state.selected else {
        return;
    };
    let Some(selected_index) = state
        .node_states
        .iter()
        .position(|ns| &ns.id == selected_id)
    else {
        return;
    };
    let node_state = &mut state.node_states[selected_index];

    match key {
        Key::ArrowUp => {
            if selected_index > 0 {
                if let Some(node) =
                    // Search for previous visible node.
                    state.node_states[0..selected_index]
                        .iter()
                        .rev()
                        .find(|node| node.visible)
                {
                    state.selected = Some(node.id);
                }
            }
        }
        Key::ArrowDown => {
            if selected_index < state.node_states.len() - 1 {
                // Search for previous visible node.
                if let Some(node) = state.node_states[(selected_index + 1)..]
                    .iter()
                    .find(|node| node.visible)
                {
                    state.selected = Some(node.id);
                }
            }
        }
        Key::ArrowLeft => {
            if node_state.open {
                node_state.open = false;
            } else if node_state.parent_id.is_some() {
                state.selected = node_state.parent_id;
            }
        }
        Key::ArrowRight => {
            if node_state.open {
                if selected_index < state.node_states.len() - 1 {
                    // Search for previous visible node.
                    if let Some(node) = state.node_states[(selected_index + 1)..]
                        .iter()
                        .find(|node| node.visible)
                    {
                        state.selected = Some(node.id);
                    }
                }
            } else {
                node_state.open = true;
            }
        }
        _ => (),
    }
}

/// Holds the data that is required to display a tree view.
/// This is simply a blob of all the data together without
/// further structure because abstracting this more simply
/// increases the complexity without much benefit.
struct TreeViewData<'state, NodeIdType> {
    /// State of the tree that is persistant across frames.
    peristant: &'state mut TreeViewState<NodeIdType>,
    /// Response of the interaction.
    interaction_response: Response,
    /// NodeId and Drop position of the drop target.
    drop: Option<(NodeIdType, DropPosition<NodeIdType>)>,
    /// Shape index of the drop marker
    drop_marker_idx: ShapeIdx,
    /// Wether or not the tree view has keyboard focus.
    has_focus: bool,
    /// Actions for the tree view.
    actions: Vec<Action<NodeIdType>>,
    /// New node states for when this frame is done.
    new_node_states: Vec<NodeState<NodeIdType>>,
}
impl<'state, NodeIdType> TreeViewData<'state, NodeIdType> {
    fn new(ui: &mut Ui, state: &'state mut TreeViewState<NodeIdType>, id: Id) -> Self {
        let interaction_response = interact_no_expansion(
            ui,
            Rect::from_min_size(ui.cursor().min, state.size),
            id,
            Sense::click_and_drag(),
        );
        let has_focus = ui.memory(|m| m.has_focus(id));

        TreeViewData {
            peristant: state,
            drop: None,
            drop_marker_idx: ui.painter().add(Shape::Noop),
            interaction_response,
            has_focus,
            actions: Vec::new(),
            new_node_states: Vec::new(),
        }
    }
}
impl<NodeIdType: TreeViewId> TreeViewData<'_, NodeIdType> {
    pub fn interact(&self, rect: &Rect) -> Interaction {
        if !self
            .interaction_response
            .hover_pos()
            .is_some_and(|pos| rect.contains(pos))
        {
            return Interaction {
                clicked: false,
                double_clicked: false,
                secondary_clicked: false,
                hovered: false,
                drag_started: false,
            };
        }

        Interaction {
            clicked: self.interaction_response.clicked(),
            double_clicked: self.interaction_response.double_clicked(),
            secondary_clicked: self.interaction_response.secondary_clicked(),
            hovered: self.interaction_response.hovered(),
            drag_started: self
                .interaction_response
                .drag_started_by(egui::PointerButton::Primary),
        }
    }
    /// Is the current drag valid.
    /// `false` if no drag is currently registered.
    pub fn drag_valid(&self) -> bool {
        self.peristant
            .dragged
            .as_ref()
            .is_some_and(|drag_state| drag_state.drag_valid)
    }
    /// Is the given id part of a valid drag.
    pub fn is_dragged(&self, id: &NodeIdType) -> bool {
        self.peristant
            .dragged
            .as_ref()
            .is_some_and(|drag_state| drag_state.drag_valid && &drag_state.node_id == id)
    }

    pub fn is_selected(&self, id: &NodeIdType) -> bool {
        self.peristant.selected.as_ref().is_some_and(|n| n == id)
    }

    pub fn is_secondary_selected(&self, id: &NodeIdType) -> bool {
        self.peristant
            .secondary_selection
            .as_ref()
            .is_some_and(|n| n == id)
    }
}

struct Interaction {
    pub clicked: bool,
    pub double_clicked: bool,
    pub secondary_clicked: bool,
    pub hovered: bool,
    pub drag_started: bool,
}

/// Contains information about a drag and drop that the
/// tree view produced.
#[derive(Debug)]
pub struct DragDropAction<NodeIdType> {
    /// Id of the dragged node.
    pub source: NodeIdType,
    /// Id of the node where the dragged node is added to.
    pub target: NodeIdType,
    /// Position of the dragged node in the drop node.
    pub position: DropPosition<NodeIdType>,
    /// Wether or not the dnd is just hovering or should be commited.  
    /// `true` -> The drag and drop should be commited.  
    /// `false` -> The drag and drop is hovering.
    pub commit: bool,
}

/// Where a dragged item should be dropped to in a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPosition<NodeIdType> {
    First,
    Last,
    After(NodeIdType),
    Before(NodeIdType),
}

struct TreeViewSettings {
    override_indent: Option<f32>,
    vline_style: VLineStyle,
    row_layout: RowLayout,
    max_width: f32,
    max_height: f32,
    min_width: f32,
    min_height: f32,
    fill_space_horizontal: bool,
    fill_space_vertical: bool,
}

impl Default for TreeViewSettings {
    fn default() -> Self {
        Self {
            override_indent: None,
            vline_style: Default::default(),
            row_layout: Default::default(),
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            min_width: 0.0,
            min_height: 0.0,
            fill_space_horizontal: true,
            fill_space_vertical: false,
        }
    }
}

/// Style of the vertical line to show the indentation level.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum VLineStyle {
    /// No vline is shown.
    None,
    /// A single vertical line is show for the full hight of the directory.
    VLine,
    /// A vline is show with horizontal hooks to the child nodes of the directory.
    #[default]
    Hook,
}

/// How rows in the tree are layed out.
///
/// Each row in the tree is made up of three elements. A closer,
/// an icon and a label. The layout of these elements is controlled
/// by this value.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum RowLayout {
    /// No icons are displayed.
    /// Directories only show the closer and the label.
    /// Leaves only show the label and allocate no additional space for a closer.
    /// Labels between leaves and directories do not align.
    Compact,
    /// The labels of leaves and directories are aligned.
    /// Icons are displayed for leaves only.
    CompactAlignedLables,
    /// The icons of leaves and directories are aligned.
    /// If a leaf or directory does not show an icon, the label will fill the
    /// space. Lables between leaves and directories can be misaligned.
    #[default]
    AlignedIcons,
    /// The labels of leaves and directories are alignd.
    /// If a leaf or directory does not show an icon, the label will not fill
    /// the space.
    AlignedIconsAndLabels,
}

/// An action the tree view would like to take as a result
/// of some user input like drag and drop.
#[derive(Clone)]
pub enum Action<NodeIdType> {
    /// Set the selected node to be this.
    SetSelected(Option<NodeIdType>),
    /// Move a node from one place to another.
    Move {
        source: NodeIdType,
        target: NodeIdType,
        position: DropPosition<NodeIdType>,
    },
    /// An inprocess drag and drop action where the node
    /// is currently dragged but not yet dropped.
    Drag {
        source: NodeIdType,
        target: NodeIdType,
        position: DropPosition<NodeIdType>,
    },
}

pub struct TreeViewResponse<NodeIdType> {
    pub response: Response,
    /// Actions this tree view would like to perform.
    pub actions: Vec<Action<NodeIdType>>,
    // /// If a row was dragged in the tree this will contain information about
    // /// who was dragged to who and at what position.
    // pub drag_drop_action: Option<DragDropAction<NodeIdType>>,
    drop_marker_idx: ShapeIdx,
}
impl<NodeIdType: TreeViewId> TreeViewResponse<NodeIdType> {
    /// Remove the drop marker from the tree view.
    ///
    /// Use this to remove the drop marker if a proposed drag and drop action
    /// is disallowed.
    pub fn remove_drop_marker(&self, ui: &mut Ui) {
        ui.painter().set(self.drop_marker_idx, Shape::Noop);
    }
}

/// Interact with the ui without egui adding any extra space.
fn interact_no_expansion(ui: &mut Ui, rect: Rect, id: Id, sense: Sense) -> Response {
    let spacing_before = ui.spacing().clone();
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let res = ui.interact(rect, id, sense);
    *ui.spacing_mut() = spacing_before;
    res
}
