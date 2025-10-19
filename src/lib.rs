pub mod input;
mod layout;
mod math;
pub mod rendering;
pub mod style;
mod widgets;

use input::*;
use intmap::{IntKey, IntMap};
pub use math::*;
use parley::layout::Layout as TextLayout;
use slotmap::{SecondaryMap, SlotMap};
use smallbox::smallbox;
use smallvec::SmallVec;
use std::any::Any;
use std::ops::Deref;
use style::computed::*;
use style::*;

type SmallBox<T, const INLINE_SIZE: usize> = smallbox::SmallBox<T, [usize; INLINE_SIZE]>;

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

fn point_in_rect<U: Unit>(point: Vec2<U>, position: Vec2<U>, size: Vec2<U>) -> bool {
    (point.x >= position.x)
        && (point.x <= position.x + size.x)
        && (point.y >= position.y)
        && (point.y <= position.y + size.y)
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
    position: Vec2<Pixel>,
    vertical_text_offset: Float<Pixel>,
}

impl Node {
    fn new(uid: Option<Uid>, style: ComputedStyle) -> Self {
        Self {
            uid,
            style,
            text_layout: None,
            position: Vec2::default(),
            vertical_text_offset: 0.px(),
        }
    }

    fn clip_bounds(&self) -> (Vec2<Pixel>, Vec2<Pixel>) {
        let clip_position = Vec2 {
            x: self.position.x + self.style.padding().left,
            y: self.position.y + self.style.padding().top,
        };

        let clip_size = Vec2 {
            x: self.style.fixed_size.x - self.style.padding().left - self.style.padding().right,
            y: self.style.fixed_size.y - self.style.padding().top - self.style.padding().bottom,
        };

        (clip_position, clip_size)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistentStateKey {
    HorizontalScroll,
    VerticalScroll,
    ScrollBarThumbMouseOffset,

    Custom(&'static str),
}

type PersistentState = rapidhash::RapidHashMap<PersistentStateKey, SmallBox<dyn Any, 1>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverState {
    /// The node is not hovered
    #[default]
    NotHovered,
    /// The node or one of its children is hovered
    Hovered,
    /// The node is hovered but none of its children are hovered
    DirectlyHovered,
}

#[derive(Default)]
pub struct PreviousState {
    /// Keeps track of whether this state still needs to be stored
    referenced: bool,

    pub hover_state: HoverState,
    pub size: Vec2<Pixel>,
    pub content_size: Vec2<Pixel>,
    pub position: Vec2<Pixel>,
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
    scale_factor: f32,
    input_state: InputState,
    root_id: Option<NodeId>,
    hovered_node_override: Option<Uid>,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeResponse<T> {
    pub hover_state: HoverState,
    pub pressed_buttons: MouseButtons,
    pub clicked_buttons: MouseButtons,
    pub released_buttons: MouseButtons,
    pub result: T,
}

impl<T> NodeResponse<T> {
    #[inline]
    pub fn map_result<U>(self, f: impl FnOnce(T) -> U) -> NodeResponse<U> {
        NodeResponse {
            hover_state: self.hover_state,
            pressed_buttons: self.pressed_buttons,
            clicked_buttons: self.clicked_buttons,
            released_buttons: self.released_buttons,
            result: f(self.result),
        }
    }

    #[inline]
    pub fn is_hovered(&self) -> bool {
        matches!(
            self.hover_state,
            HoverState::Hovered | HoverState::DirectlyHovered,
        )
    }

    #[inline]
    pub fn is_directly_hovered(&self) -> bool {
        matches!(self.hover_state, HoverState::DirectlyHovered)
    }

    #[inline]
    pub fn pressed(&self, buttons: MouseButtons) -> bool {
        self.pressed_buttons.contains(buttons)
    }

    #[inline]
    pub fn clicked(&self, buttons: MouseButtons) -> bool {
        self.clicked_buttons.contains(buttons)
    }

    #[inline]
    pub fn released(&self, buttons: MouseButtons) -> bool {
        self.released_buttons.contains(buttons)
    }
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
        let mouse_position = self.input_state.mouse_position();
        let mouse_in_bounds = mouse_in_parent_clip_bounds
            && point_in_rect(mouse_position, node.position, node.style.fixed_size);

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
            let mut total_content_size = Vec2::default();
            let mut max_content_size = Vec2::default();
            for (_, child) in self.iter_children(node_id) {
                total_content_size += child.style.fixed_size;
                max_content_size = max_content_size.max(child.style.fixed_size);
            }

            let total_spacing =
                (self.child_count(node_id).saturating_sub(1) as f32) * node.style.child_spacing();

            let state = self.previous_state.entry(uid).or_default();
            state.referenced = true; // this state is indeed still referenced

            state.hover_state = if let Some(hovered_node_override) = self.hovered_node_override {
                if uid == hovered_node_override {
                    hovered_node = Some(uid);
                    HoverState::DirectlyHovered
                } else {
                    HoverState::NotHovered
                }
            } else if mouse_in_bounds {
                if hovered_node.is_none() {
                    hovered_node = Some(uid);
                    HoverState::DirectlyHovered
                } else {
                    HoverState::Hovered
                }
            } else {
                HoverState::NotHovered
            };

            state.size = node.style.fixed_size;
            state.content_size = match node.style.layout_direction() {
                Direction::LeftToRight => Vec2 {
                    x: total_content_size.x + total_spacing,
                    y: max_content_size.y,
                },
                Direction::TopToBottom => Vec2 {
                    x: max_content_size.x,
                    y: total_content_size.y + total_spacing,
                },
            };
            state.position = node.position;
        }

        hovered_node
    }

    fn update_previous_states(&mut self, root_id: NodeId) {
        if self.input_state.pressed_buttons().is_empty() {
            self.hovered_node_override = None;
        }

        self.previous_state
            .values_mut()
            .for_each(|state| state.referenced = false);
        let hovered_node = self.compute_previous_state(root_id, true);
        self.previous_state.retain(|_, state| state.referenced);

        if !self.input_state.pressed_buttons().is_empty() {
            self.hovered_node_override = hovered_node;
        }
    }

    #[must_use]
    #[inline(never)]
    fn begin_frame<'gui>(
        &'gui mut self,
        screen_size: Vec2<Pixel>,
        scale_factor: f32,
        mouse_state: MouseState,
    ) -> ByorGuiContext<'gui> {
        self.nodes.clear();
        self.children.clear();
        self.text_layouts.clear();

        self.scale_factor = scale_factor;
        self.input_state.update(mouse_state);

        let cascaded_style = self.root_style().cascade_root(screen_size);
        let computed_style = compute_style(self.root_style(), &cascaded_style, None, scale_factor);
        let root_id = self.nodes.insert(Node::new(None, computed_style));
        self.root_id = Some(root_id);

        ByorGuiContext {
            gui: self,
            parent_id: root_id,
            parent_style: cascaded_style,
        }
    }

    #[inline(never)]
    fn end_frame(&mut self) {
        let root_id = self.root_id.unwrap();
        self.layout(root_id);
        self.update_previous_states(root_id);
    }

    #[inline]
    pub fn frame<R>(
        &mut self,
        screen_size: Vec2<Pixel>,
        scale_factor: f32,
        mouse_state: MouseState,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let context = self.begin_frame(screen_size, scale_factor, mouse_state);
        let result = contents(context);
        self.end_frame();

        result
    }

    pub fn render<R: rendering::Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        if let Some(root_id) = self.root_id {
            return self.render_impl(root_id, renderer);
        }

        Ok(())
    }

    #[must_use]
    #[inline(never)]
    fn compute_node_response(&self, uid: Option<Uid>) -> NodeResponse<()> {
        let hover_state = uid
            .and_then(|uid| self.previous_state.get(uid))
            .map(|previous_state| previous_state.hover_state)
            .unwrap_or_default();

        let (pressed_buttons, clicked_buttons, released_buttons) =
            if hover_state == HoverState::DirectlyHovered {
                (
                    self.input_state.pressed_buttons(),
                    self.input_state.clicked_buttons(),
                    self.input_state.released_buttons(),
                )
            } else {
                (
                    MouseButtons::empty(),
                    MouseButtons::empty(),
                    MouseButtons::empty(),
                )
            };

        NodeResponse {
            hover_state,
            pressed_buttons,
            clicked_buttons,
            released_buttons,
            result: (),
        }
    }

    fn layout_text(&mut self, text: &str, node_id: NodeId) {
        use parley::style::{LineHeight, OverflowWrap, StyleProperty};

        global_cache::with_parley_global_data(|parley_global_data| {
            let mut builder = parley_global_data.builder(text, 1.0);

            let style = &self.nodes[node_id].style;
            builder.push_default(StyleProperty::Brush(style.text_color()));
            builder.push_default(StyleProperty::FontStack(style.font_family().clone()));
            builder.push_default(StyleProperty::FontSize(style.font_size().value()));
            builder.push_default(StyleProperty::FontStyle(style.font_style()));
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
    parent_style: CascadedStyle,
}

impl ByorGuiContext<'_> {
    #[must_use]
    #[inline(never)]
    pub fn scale_factor(&self) -> f32 {
        self.gui.scale_factor
    }

    #[must_use]
    #[inline(never)]
    fn insert_leaf_node<'gui>(
        &'gui mut self,
        uid: Option<Uid>,
        style: &Style,
        parent_id: NodeId,
    ) -> ByorGuiContext<'gui> {
        let cascaded_style = style.cascade(&self.parent_style);
        let computed_style = compute_style(
            style,
            &cascaded_style,
            Some(&self.gui.nodes[self.parent_id].style),
            self.gui.scale_factor,
        );
        let node_id = self.gui.nodes.insert(Node::new(uid, computed_style));

        self.gui
            .children
            .entry(parent_id)
            .expect("invalid node ID in ancestor stack")
            .or_default()
            .push(node_id);

        ByorGuiContext {
            gui: self.gui,
            parent_id: node_id,
            parent_style: cascaded_style,
        }
    }

    #[inline]
    pub fn insert_node(&mut self, uid: Option<Uid>, style: &Style) -> NodeResponse<()> {
        let _ = self.insert_leaf_node(uid, style, self.parent_id);
        self.gui.compute_node_response(uid)
    }

    #[inline]
    pub fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> NodeResponse<R> {
        let context = self.insert_leaf_node(uid, style, self.parent_id);
        let result = contents(context);
        self.gui.compute_node_response(uid).map_result(|_| result)
    }

    #[inline]
    pub fn insert_text_node(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        text: &str,
    ) -> NodeResponse<()> {
        let node_id = self.insert_leaf_node(uid, style, self.parent_id).parent_id;
        self.gui.layout_text(text, node_id);
        self.gui.compute_node_response(uid)
    }

    #[must_use]
    #[inline]
    pub fn parent_style(&self) -> &CascadedStyle {
        &self.parent_style
    }

    #[must_use]
    #[inline]
    pub fn computed_parent_style(&self) -> &ComputedStyle {
        &self.gui.nodes[self.parent_id].style
    }

    #[must_use]
    #[inline]
    pub fn input_state(&self) -> &InputState {
        &self.gui.input_state
    }

    #[must_use]
    #[inline]
    pub fn get_persistent_state<T: Any>(&self, uid: Uid, key: PersistentStateKey) -> Option<&T> {
        let state = self.gui.persistent_state.get(uid)?;
        let any = state.get(&key)?;
        any.downcast_ref()
    }

    #[must_use]
    #[inline]
    pub fn get_persistent_state_mut<T: Any>(
        &mut self,
        uid: Uid,
        key: PersistentStateKey,
    ) -> Option<&mut T> {
        let state = self.gui.persistent_state.get_mut(uid)?;
        let any = state.get_mut(&key)?;
        any.downcast_mut()
    }

    #[inline]
    pub fn get_or_insert_persistent_state<T: Any>(
        &mut self,
        uid: Uid,
        key: PersistentStateKey,
        default: impl FnOnce() -> T,
    ) -> Option<&mut T> {
        let state = self.gui.persistent_state.entry(uid).or_default();
        let any = state.entry(key).or_insert_with(|| smallbox!(default()));
        any.downcast_mut()
    }

    #[inline]
    pub fn insert_persistent_state<T: Any>(&mut self, uid: Uid, key: PersistentStateKey, value: T) {
        let state = self.gui.persistent_state.entry(uid).or_default();
        state.insert(key, smallbox!(value));
    }

    #[must_use]
    #[inline]
    pub fn get_previous_state(&self, uid: Uid) -> Option<&PreviousState> {
        self.gui.previous_state.get(uid)
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
