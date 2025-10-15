mod layout;
pub mod rendering;
pub mod style;
mod widgets;

use intmap::{IntKey, IntMap};
use parley::layout::Layout as TextLayout;
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::ops::Deref;
use style::*;

#[derive(Default)]
struct ParleyGlobalData {
    layout_context: parley::LayoutContext<Color>,
    font_context: parley::FontContext,
}

impl ParleyGlobalData {
    fn builder<'a>(&'a mut self, text: &'a str, scale: f32) -> parley::RangedBuilder<'a, Color> {
        self.layout_context
            .ranged_builder(&mut self.font_context, text, scale, true)
    }
}

#[cfg(feature = "unique_global_cache")]
mod global_cache {
    use super::ParleyGlobalData;
    use std::sync::{LazyLock, Mutex};

    static PARLEY_GLOBAL_DATA: LazyLock<Mutex<ParleyGlobalData>> =
        LazyLock::new(|| Mutex::new(ParleyGlobalData::default()));

    pub(crate) fn with_parley_global_data<R>(f: impl FnOnce(&mut ParleyGlobalData) -> R) -> R {
        let mut lock = PARLEY_GLOBAL_DATA.lock().unwrap();
        f(&mut *lock)
    }
}

#[cfg(not(feature = "unique_global_cache"))]
mod global_cache {
    use super::ParleyGlobalData;
    use std::cell::RefCell;

    thread_local! {
        static PARLEY_GLOBAL_DATA: RefCell<ParleyGlobalData> = RefCell::new(ParleyGlobalData::default());
    }

    #[inline]
    pub(crate) fn with_parley_global_data<R>(f: impl FnOnce(&mut ParleyGlobalData) -> R) -> R {
        PARLEY_GLOBAL_DATA.with_borrow_mut(f)
    }
}

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

fn point_in_rect(point: Position, position: Position, size: Size) -> bool {
    (point.x >= position.x)
        && (point.x <= position.x + size.width)
        && (point.y >= position.y)
        && (point.y <= position.y + size.height)
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

    fn new(uid: Option<Uid>, style: ComputedStyle) -> Self {
        Self {
            uid,
            style,
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
            x: self.position.x + self.style.padding().left,
            y: self.position.y + self.style.padding().top,
        };

        let clip_size = Size {
            width: self.size.width - self.style.padding().left - self.style.padding().right,
            height: self.size.height - self.style.padding().top - self.style.padding().bottom,
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
    text_layouts: SlotMap<TextLayoutId, TextLayout<Color>>,

    persistent_state: IntMap<Uid, PersistentState>,
    previous_state: IntMap<Uid, PreviousState>,

    root_style: Style,
    prev_mouse_state: MouseState,
    mouse_state: MouseState,
    root_id: Option<NodeId>,
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

    #[must_use]
    fn compute_previous_state(
        &mut self,
        node_id: NodeId,
        mouse_in_parent_clip_bounds: bool,
    ) -> Option<Uid> {
        let mut hovered_node = None;

        let node = &self.nodes[node_id];
        let mouse_position = self.mouse_state.position;
        let mouse_in_bounds =
            mouse_in_parent_clip_bounds && point_in_rect(mouse_position, node.position, node.size);

        let (clip_position, clip_size) = node.clip_bounds();
        let mouse_in_clip_bounds =
            mouse_in_bounds && point_in_rect(mouse_position, clip_position, clip_size);

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
                (self.child_count(node_id).saturating_sub(1) as Pixel) * node.style.child_spacing();

            let state = self.previous_state.entry(uid).or_default();
            state.referenced = true; // this state is indeed still referenced

            state.hovered = if mouse_in_bounds && hovered_node.is_none() {
                hovered_node = Some(uid);
                true
            } else {
                false
            };

            state.inner_size = Size {
                width: node.size.width - node.style.padding().left - node.style.padding().right,
                height: node.size.height - node.style.padding().top - node.style.padding().bottom,
            };

            state.content_size = match node.style.layout_direction() {
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

    pub fn frame<R>(
        &mut self,
        screen_size: Size,
        mouse_state: MouseState,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        self.nodes.clear();
        self.children.clear();
        self.text_layouts.clear();

        self.prev_mouse_state = self.mouse_state;
        self.mouse_state = mouse_state;

        let root = Node::new_root(self.root_style(), screen_size);
        let root_id = self.nodes.insert(root);
        self.root_id = Some(root_id);

        let result = contents(ByorGuiContext {
            gui: self,
            parent_id: root_id,
        });

        self.layout(root_id);
        self.update_previous_states(root_id);

        result
    }

    pub fn render<R: rendering::Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        if let Some(root_id) = self.root_id {
            return self.render_impl(root_id, renderer);
        }

        Ok(())
    }

    #[must_use]
    fn insert_leaf_node(&mut self, uid: Option<Uid>, style: &Style, parent_id: NodeId) -> NodeId {
        let parent_style = &self.nodes[parent_id].style;
        let style = style.compute(parent_style);
        let node_id = self.nodes.insert(Node::new(uid, style));

        self.children
            .entry(parent_id)
            .expect("invalid node ID in ancestor stack")
            .or_default()
            .push(node_id);

        node_id
    }

    #[must_use]
    fn compute_node_response<R>(&self, uid: Option<Uid>, result: R) -> NodeResponse<R> {
        let hovered = uid
            .and_then(|uid| self.previous_state.get(uid))
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

        global_cache::with_parley_global_data(|parley_global_data| {
            let mut builder = parley_global_data.builder(text, 1.0);

            let style = &self.nodes[node_id].style;
            builder.push_default(StyleProperty::Brush(style.text_color()));
            builder.push_default(StyleProperty::FontStack(style.font_family().clone()));
            builder.push_default(StyleProperty::FontSize(style.font_size()));
            builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.3)));
            builder.push_default(StyleProperty::FontWeight(style.font_weight()));
            builder.push_default(StyleProperty::FontWidth(style.font_width()));
            builder.push_default(StyleProperty::Underline(style.text_underline()));
            builder.push_default(StyleProperty::Strikethrough(style.text_strikethrough()));
            builder.push_default(StyleProperty::OverflowWrap(OverflowWrap::BreakWord));

            let text_layout = self.text_layouts.insert(builder.build(text));
            self.nodes[node_id].text_layout = Some(text_layout);
        });
    }
}

pub struct ByorGuiContext<'gui> {
    gui: &'gui mut ByorGui,
    parent_id: NodeId,
}

impl ByorGuiContext<'_> {
    pub fn insert_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeResponse<()> {
        let _ = self.gui.insert_leaf_node(uid, style, self.parent_id);
        self.gui.compute_node_response(uid, ())
    }

    pub fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> NodeResponse<R> {
        let node_id = self.gui.insert_leaf_node(uid, style, self.parent_id);

        let result = contents(ByorGuiContext {
            gui: self.gui,
            parent_id: node_id,
        });

        self.gui.compute_node_response(uid, result)
    }

    pub fn insert_text_node(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        text: &str,
    ) -> NodeResponse<()> {
        let node_id = self.gui.insert_leaf_node(uid, style, self.parent_id);
        self.gui.layout_text(text, node_id);

        self.gui.compute_node_response(uid, ())
    }

    pub fn parent_style(&self) -> &ComputedStyle {
        &self.gui.nodes[self.parent_id].style
        //ComputedStyleRef::new(
        //    &self.gui.nodes[self.parent_id].style,
        //    &self.gui.style_data,
        //)
    }

    pub fn get_persistent_state(&self, uid: Uid) -> &PersistentState {
        self.gui
            .persistent_state
            .get(uid)
            .unwrap_or(&PersistentState::DEFAULT)
    }

    pub fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        self.gui
            .persistent_state
            .entry(uid)
            .or_insert(PersistentState::DEFAULT)
    }

    #[inline]
    pub fn get_previous_state(&self, uid: Uid) -> Option<&PreviousState> {
        self.gui.previous_state.get(uid)
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
