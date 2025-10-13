mod layout;
pub mod rendering;
pub mod style;
pub mod widgets;

use intmap::{IntKey, IntMap};
use parley::FontContext;
use parley::LayoutContext as TextLayoutContext;
use parley::layout::Layout as TextLayout;
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::ops::Deref;
use style::*;

pub type Pixel = f32;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: Pixel,
    pub y: Pixel,
}

impl From<Pixel> for Position {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self { x: value, y: value }
    }
}

impl From<(Pixel, Pixel)> for Position {
    #[inline]
    fn from(value: (Pixel, Pixel)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<[Pixel; 2]> for Position {
    #[inline]
    fn from(value: [Pixel; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: Pixel,
    pub height: Pixel,
}

impl From<Pixel> for Size {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

impl From<[Pixel; 2]> for Size {
    #[inline]
    fn from(value: [Pixel; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState {
    pub position: Position,
    pub button1_pressed: bool,
    pub button2_pressed: bool,
    pub button3_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Uid(u64);

impl Uid {
    #[must_use]
    pub const fn new(value: &[u8]) -> Self {
        Self(rapidhash::v3::rapidhash_v3(value))
    }

    #[must_use]
    pub const fn concat(self, other: Self) -> Self {
        let low_bytes = self.0.to_le_bytes();
        let high_bytes = other.0.to_le_bytes();
        let bytes = [
            low_bytes[0],
            low_bytes[1],
            low_bytes[2],
            low_bytes[3],
            low_bytes[4],
            low_bytes[5],
            low_bytes[6],
            low_bytes[7],
            high_bytes[0],
            high_bytes[1],
            high_bytes[2],
            high_bytes[3],
            high_bytes[4],
            high_bytes[5],
            high_bytes[6],
            high_bytes[7],
        ];
        Self::new(&bytes)
    }
}

impl IntKey for Uid {
    type Int = u64;

    // values are pre-hashed so we don't need to "hash" again
    const PRIME: Self::Int = 1;

    fn into_int(self) -> Self::Int {
        self.0
    }
}

slotmap::new_key_type! {
    struct NodeId;

    struct TextLayoutId;
}

struct Node {
    uid: Option<Uid>,
    style: ComputedStyle,
    text_layout: Option<TextLayoutId>,
    min_size: Size,
    max_size: Size,
    size: Size,
    position: Position,
    vertical_text_offset: Pixel,
}

impl Node {
    fn new_root(style: &Style, screen_size: Size) -> Self {
        Node {
            uid: None,
            style: style.compute_root(screen_size),
            text_layout: None,
            min_size: Size::default(),
            max_size: Size::default(),
            size: Size::default(),
            position: Position::default(),
            vertical_text_offset: 0.0,
        }
    }

    fn new(uid: Option<Uid>, style: &Style, parent_style: &ComputedStyle) -> Self {
        Self {
            uid,
            style: style.compute(parent_style),
            text_layout: None,
            min_size: Size::default(),
            max_size: Size::default(),
            size: Size::default(),
            position: Position::default(),
            vertical_text_offset: 0.0,
        }
    }

    fn clip_bounds(&self) -> (Position, Size) {
        let clip_position = Position {
            x: self.position.x + self.style.padding.left,
            y: self.position.y + self.style.padding.top,
        };

        let clip_size = Size {
            width: self.size.width - self.style.padding.left - self.style.padding.right,
            height: self.size.height - self.style.padding.top - self.style.padding.bottom,
        };

        (clip_position, clip_size)
    }
}

pub struct PersistentState {
    pub horizontal_scroll: Option<Pixel>,
    pub vertical_scroll: Option<Pixel>,
}

impl PersistentState {
    pub const DEFAULT: Self = Self {
        horizontal_scroll: None,
        vertical_scroll: None,
    };
}

impl Default for PersistentState {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Default)]
pub struct PreviousState {
    /// Keeps track of whether this state still needs to be stored
    referenced: bool,

    pub hovered: bool,
    pub inner_size: Size,
    pub content_size: Size,
}

const INLINE_NODE_ID_COUNT: usize = (2 * size_of::<usize>()) / size_of::<NodeId>();
type NodeIdVec = SmallVec<[NodeId; INLINE_NODE_ID_COUNT]>;

#[derive(Default)]
pub struct ByorGui {
    nodes: SlotMap<NodeId, Node>,
    children: SecondaryMap<NodeId, NodeIdVec>,
    uid_map: IntMap<Uid, NodeId>,
    ancestor_stack: Vec<NodeId>,
    text_layouts: SlotMap<TextLayoutId, TextLayout<Color>>,
    persistent_state: IntMap<Uid, PersistentState>,
    previous_state: IntMap<Uid, PreviousState>,

    root_style: Style,
    prev_mouse_state: MouseState,
    mouse_state: MouseState,

    text_layout_context: TextLayoutContext<Color>,
    font_context: FontContext,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeResponse<R> {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub result: R,
}

struct ChildMutIter<'a> {
    nodes: &'a mut SlotMap<NodeId, Node>,
    children: &'a [NodeId],
    index: usize,
}

impl ChildMutIter<'_> {
    #[inline]
    fn reset(&mut self) {
        self.index = 0;
    }

    // this is a lending iterator, so we can't use the trait
    fn next(&mut self) -> Option<&mut Node> {
        self.children.get(self.index).map(|&child_id| {
            self.index += 1;
            &mut self.nodes[child_id]
        })
    }
}

impl ByorGui {
    #[must_use]
    #[inline]
    pub fn root_style(&self) -> &Style {
        &self.root_style
    }

    #[must_use]
    #[inline]
    pub fn root_style_mut(&mut self) -> &mut Style {
        &mut self.root_style
    }

    #[must_use]
    fn current_node_id(&self) -> NodeId {
        *self.ancestor_stack.last().expect("no root node present")
    }

    #[must_use]
    fn child_count(&self, node_id: NodeId) -> usize {
        self.children
            .get(node_id)
            .map(SmallVec::len)
            .unwrap_or_default()
    }

    #[must_use]
    fn child_ids(&self, node_id: NodeId) -> &[NodeId] {
        self.children.get(node_id).map(Deref::deref).unwrap_or(&[])
    }

    #[must_use]
    fn iter_children(&self, node_id: NodeId) -> impl Iterator<Item = (NodeId, &Node)> {
        self.child_ids(node_id)
            .iter()
            .map(|&child_id| (child_id, &self.nodes[child_id]))
    }

    #[must_use]
    fn iter_children_mut(&mut self, node_id: NodeId) -> ChildMutIter<'_> {
        ChildMutIter {
            nodes: &mut self.nodes,
            children: self.children.get(node_id).map(Deref::deref).unwrap_or(&[]),
            index: 0,
        }
    }

    pub fn begin_frame(&mut self, screen_size: Size, mouse_state: MouseState) {
        self.nodes.clear();
        self.children.clear();
        self.uid_map.clear();
        assert!(self.ancestor_stack.is_empty());
        self.text_layouts.clear();

        let root = Node::new_root(self.root_style(), screen_size);
        let root_id = self.nodes.insert(root);
        self.ancestor_stack.push(root_id);

        self.prev_mouse_state = self.mouse_state;
        self.mouse_state = mouse_state;
    }

    #[must_use]
    fn compute_previous_state(
        &mut self,
        node_id: NodeId,
        mouse_in_parent_clip_bounds: bool,
    ) -> Option<Uid> {
        let mut hovered_node = None;

        let node = &self.nodes[node_id];
        let mouse_position = self.mouse_state.position;
        let mouse_in_bounds = mouse_in_parent_clip_bounds
            && (mouse_position.x >= node.position.x)
            && (mouse_position.x <= node.position.x + node.size.width)
            && (mouse_position.y >= node.position.y)
            && (mouse_position.y <= node.position.y + node.size.height);

        let (clip_position, clip_size) = node.clip_bounds();
        let mouse_in_clip_bounds = mouse_in_bounds
            && (mouse_position.x >= clip_position.x)
            && (mouse_position.x <= clip_position.x + clip_size.width)
            && (mouse_position.y >= clip_position.y)
            && (mouse_position.y <= clip_position.y + clip_size.height);

        // we have to use index-based iteration because of borrowing
        let child_count = self.child_count(node_id);
        for i in 0..child_count {
            let child_id = self.child_ids(node_id)[i];
            if let Some(uid) = self.compute_previous_state(child_id, mouse_in_clip_bounds) {
                assert!(hovered_node.is_none(), "multiple nodes hovered");
                hovered_node = Some(uid);
            }
        }

        let node = &self.nodes[node_id];
        if let Some(uid) = node.uid {
            let mut total_content_size = Size::default();
            let mut max_content_size = Size::default();
            for (_, child) in self.iter_children(node_id) {
                total_content_size.width += child.size.width;
                total_content_size.height += child.size.height;

                max_content_size.width = max_content_size.width.max(child.size.width);
                max_content_size.height = max_content_size.height.max(child.size.height);
            }

            let total_spacing =
                (self.child_count(node_id).saturating_sub(1) as Pixel) * node.style.child_spacing;

            let state = self.previous_state.entry(uid).or_default();
            state.referenced = true; // this state is indeed still referenced

            state.hovered = if mouse_in_bounds && hovered_node.is_none() {
                hovered_node = Some(uid);
                true
            } else {
                false
            };

            state.inner_size = Size {
                width: node.size.width - node.style.padding.left - node.style.padding.right,
                height: node.size.height - node.style.padding.top - node.style.padding.bottom,
            };

            state.content_size = match node.style.layout_direction {
                Direction::LeftToRight => Size {
                    width: total_content_size.width + total_spacing,
                    height: max_content_size.height,
                },
                Direction::TopToBottom => Size {
                    width: max_content_size.width,
                    height: total_content_size.height + total_spacing,
                },
            };
        }

        hovered_node
    }

    fn update_previous_states(&mut self, root_id: NodeId) {
        self.previous_state
            .values_mut()
            .for_each(|state| state.referenced = false);
        let _ = self.compute_previous_state(root_id, true);
        self.previous_state.retain(|_, state| state.referenced);
    }

    pub fn end_frame<R: rendering::Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let root_id = self.ancestor_stack.pop().unwrap();
        assert!(self.ancestor_stack.is_empty());

        self.layout(root_id);
        self.update_previous_states(root_id);
        self.render(root_id, renderer)
    }

    #[must_use]
    fn insert_leaf_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeId {
        assert!(
            !self.ancestor_stack.is_empty(),
            "inserting nodes is only possible between calls to `begin_frame` and `end_frame`",
        );

        let parent_node_id = self.current_node_id();
        let parent_style = &self.nodes[parent_node_id].style;
        let node_id = self.nodes.insert(Node::new(uid, style, parent_style));

        self.children
            .entry(parent_node_id)
            .expect("invalid node ID in ancestor stack")
            .or_default()
            .push(node_id);

        if let Some(uid) = uid {
            assert!(self.uid_map.insert_checked(uid, node_id), "duplicate UID");
        }

        node_id
    }

    #[must_use]
    fn compute_node_response<R>(&self, uid: Option<Uid>, result: R) -> NodeResponse<R> {
        let hovered = uid
            .and_then(|uid| self.get_previous_state(uid))
            .map(|previous_state| previous_state.hovered)
            .unwrap_or_default();
        let pressed = hovered && self.mouse_state.button1_pressed;
        let clicked = pressed && !self.prev_mouse_state.button1_pressed;

        NodeResponse {
            hovered,
            pressed,
            clicked,
            result,
        }
    }

    fn layout_text(&mut self, text: &str, node_id: NodeId) {
        use parley::style::{LineHeight, OverflowWrap, StyleProperty};

        let mut builder =
            self.text_layout_context
                .ranged_builder(&mut self.font_context, text, 1.0, true);

        let style = &self.nodes[node_id].style;
        builder.push_default(StyleProperty::Brush(style.text_color));
        builder.push_default(StyleProperty::FontStack(style.font.clone()));
        builder.push_default(StyleProperty::FontSize(style.font_size));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.3)));
        builder.push_default(StyleProperty::FontWeight(style.font_weight.into()));
        builder.push_default(StyleProperty::FontWidth(style.font_width));
        builder.push_default(StyleProperty::Underline(style.text_underline));
        builder.push_default(StyleProperty::Strikethrough(style.text_strikethrough));
        builder.push_default(StyleProperty::OverflowWrap(OverflowWrap::BreakWord));

        let text_layout = self.text_layouts.insert(builder.build(text));
        self.nodes[node_id].text_layout = Some(text_layout);
    }
}

pub struct ByorGuiContext<'gui> {
    gui: &'gui mut ByorGui,
    parent_id: NodeId,
}

pub trait GuiBuilder {
    fn insert_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeResponse<()>;

    fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> NodeResponse<R>;

    fn insert_text_node(&mut self, uid: Option<Uid>, style: &Style, text: &str)
    -> NodeResponse<()>;

    fn parent_style(&self) -> &ComputedStyle;

    fn get_persistent_state(&self, uid: Uid) -> &PersistentState;

    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState;

    fn get_previous_state(&self, uid: Uid) -> Option<&PreviousState>;
}

impl GuiBuilder for ByorGui {
    fn insert_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeResponse<()> {
        let _ = self.insert_leaf_node(uid, style);
        self.compute_node_response(uid, ())
    }

    fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> NodeResponse<R> {
        let node_id = self.insert_leaf_node(uid, style);

        self.ancestor_stack.push(node_id);
        let result = contents(ByorGuiContext {
            gui: self,
            parent_id: node_id,
        });
        assert!(self.ancestor_stack.pop().is_some());

        self.compute_node_response(uid, result)
    }

    fn insert_text_node(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        text: &str,
    ) -> NodeResponse<()> {
        let node_id = self.insert_leaf_node(uid, style);
        self.layout_text(text, node_id);

        self.compute_node_response(uid, ())
    }

    fn parent_style(&self) -> &ComputedStyle {
        let root_id = *self.ancestor_stack.first().expect(
            "the root style is only available between calls to `begin_frame` and `end_frame`",
        );
        &self.nodes[root_id].style
    }

    fn get_persistent_state(&self, uid: Uid) -> &PersistentState {
        self.persistent_state
            .get(uid)
            .unwrap_or(&PersistentState::DEFAULT)
    }

    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        self.persistent_state
            .entry(uid)
            .or_insert(PersistentState::DEFAULT)
    }

    #[inline]
    fn get_previous_state(&self, uid: Uid) -> Option<&PreviousState> {
        self.previous_state.get(uid)
    }
}

impl GuiBuilder for ByorGuiContext<'_> {
    #[inline]
    fn insert_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeResponse<()> {
        self.gui.insert_node(uid, style)
    }

    #[inline]
    fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> NodeResponse<R> {
        self.gui.insert_container_node(uid, style, contents)
    }

    #[inline]
    fn insert_text_node(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        text: &str,
    ) -> NodeResponse<()> {
        self.gui.insert_text_node(uid, style, text)
    }

    fn parent_style(&self) -> &ComputedStyle {
        &self.gui.nodes[self.parent_id].style
    }

    #[inline]
    fn get_persistent_state(&self, uid: Uid) -> &PersistentState {
        self.gui.get_persistent_state(uid)
    }

    #[inline]
    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        self.gui.get_persistent_state_mut(uid)
    }

    #[inline]
    fn get_previous_state(&self, uid: Uid) -> Option<&PreviousState> {
        self.gui.get_previous_state(uid)
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
