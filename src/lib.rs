mod forest;
pub mod input;
mod layout;
mod math;
mod multi_vec;
pub mod rendering;
pub mod style;
#[cfg(test)]
mod tests;
pub mod theme;
pub mod widgets;

use cranelift_entity::PrimaryMap;
use cranelift_entity::packed_option::PackedOption;
use forest::*;
use input::*;
use intmap::{IntKey, IntMap};
pub use math::*;
use parley::layout::Layout as TextLayout;
use smallbox::smallbox;
use std::any::Any;
use std::fmt;
use std::hash::Hasher;
use std::num::NonZeroU64;
use style::computed::*;
use style::*;
use theme::Theme;

use crate::rendering::NodeRenderer;

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
pub struct Uid(NonZeroU64);

#[must_use]
#[inline]
const fn uid_hash(seed: u64, data: &[u8]) -> u64 {
    let secrets = rapidhash::v3::RapidSecrets::seed_cpp(seed);
    rapidhash::v3::rapidhash_v3_inline::<true, false, false>(data, &secrets)
}

#[derive(Default)]
#[repr(transparent)]
struct UidHasher {
    seed: u64,
}

impl std::hash::Hasher for UidHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.seed
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.seed = uid_hash(self.seed, bytes);
    }

    // This is the same implementation as in the standard library.
    // We override it anyway to ensure this always stays identical to the functions below.
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_ne_bytes())
    }
}

#[must_use]
#[inline]
const fn hash_fallback(hash: u64) -> NonZeroU64 {
    // In case the astronomically small chance for 0 does occur, simply use 1 instead.
    // This does create more collisions, but the chance for both to occur simultaneously
    // is even more astronomically low, so we'll take it.
    match NonZeroU64::new(hash) {
        Some(hash) => hash,
        None => NonZeroU64::MIN,
    }
}

impl Uid {
    #[must_use]
    pub const fn from_array<const N: usize>(data: &[u8; N]) -> Self {
        let seed = uid_hash(0, &N.to_ne_bytes());
        Self(hash_fallback(uid_hash(seed, data)))
    }

    #[must_use]
    pub const fn from_slice(data: &[u8]) -> Self {
        let seed = uid_hash(0, &data.len().to_ne_bytes());
        Self(hash_fallback(uid_hash(seed, data)))
    }

    #[must_use]
    pub fn new(data: impl std::hash::Hash) -> Self {
        let mut hasher = UidHasher::default();
        data.hash(&mut hasher);
        Self(hash_fallback(hasher.finish()))
    }

    #[must_use]
    pub const fn concat(self, other: Self) -> Self {
        let low_bytes = self.0.get().to_ne_bytes();
        let high_bytes = other.0.get().to_ne_bytes();
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

        Self(hash_fallback(uid_hash(0, &bytes)))
    }
}

impl IntKey for Uid {
    type Int = u64;

    // values are pre-hashed so we don't need to "hash" again
    const PRIME: Self::Int = 1;

    #[inline]
    fn into_int(self) -> Self::Int {
        self.0.get()
    }
}

macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        struct $name(u32);

        cranelift_entity::entity_impl!($name);
    };
}

define_id_type!(TextLayoutId);
define_id_type!(NodeRendererId);

struct Node {
    uid: Option<Uid>,
    text_layout: PackedOption<TextLayoutId>,
    renderer: PackedOption<NodeRendererId>,
    style: ComputedStyle,
    position: Vec2<Pixel>,
    vertical_text_offset: Float<Pixel>,
}

impl Node {
    #[must_use]
    #[inline]
    fn new_root(style: ComputedStyle) -> Self {
        Self {
            uid: None,
            text_layout: PackedOption::default(),
            renderer: PackedOption::default(),
            style,
            position: Vec2::default(),
            vertical_text_offset: 0.px(),
        }
    }

    #[must_use]
    #[inline]
    fn new(
        uid: Option<Uid>,
        text_layout: Option<TextLayoutId>,
        renderer: Option<NodeRendererId>,
        style: ComputedStyle,
    ) -> Self {
        Self {
            uid,
            text_layout: text_layout.into(),
            renderer: renderer.into(),
            style,
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
    PreviousPopupState,

    Custom(&'static str),
}

type PersistentStateStorage = rapidhash::RapidHashMap<PersistentStateKey, SmallBox<dyn Any, 2>>;

#[derive(Default)]
enum PersistentStateRepr {
    #[default]
    Empty,
    Populated {
        storage: PersistentStateStorage,
    },
}

#[derive(Default)]
#[repr(transparent)]
pub struct PersistentState(PersistentStateRepr);

impl PersistentState {
    const EMPTY: Self = Self(PersistentStateRepr::Empty);

    #[must_use]
    #[inline]
    pub fn get<T: Any>(&self, key: PersistentStateKey) -> Option<&T> {
        let PersistentStateRepr::Populated { storage } = &self.0 else {
            return None;
        };

        let any = storage.get(&key)?;
        any.downcast_ref()
    }

    #[must_use]
    #[inline]
    pub fn get_mut<T: Any>(&mut self, key: PersistentStateKey) -> Option<&mut T> {
        let PersistentStateRepr::Populated { storage } = &mut self.0 else {
            return None;
        };

        let any = storage.get_mut(&key)?;
        any.downcast_mut()
    }

    #[must_use]
    #[inline]
    fn storage_mut<'a>(&'a mut self) -> &'a mut PersistentStateStorage {
        if let PersistentStateRepr::Empty = self.0 {
            self.0 = PersistentStateRepr::Populated {
                storage: PersistentStateStorage::default(),
            };
        }

        let PersistentStateRepr::Populated { storage } = &mut self.0 else {
            unreachable!()
        };
        storage
    }

    #[must_use]
    pub fn get_or_insert<T: Any>(&mut self, key: PersistentStateKey, default: T) -> Option<&mut T> {
        let any = self
            .storage_mut()
            .entry(key)
            .or_insert_with(|| smallbox!(default));
        any.downcast_mut()
    }

    #[must_use]
    pub fn get_or_insert_with<T: Any>(
        &mut self,
        key: PersistentStateKey,
        default: impl FnOnce() -> T,
    ) -> Option<&mut T> {
        let any = self
            .storage_mut()
            .entry(key)
            .or_insert_with(|| smallbox!(default()));
        any.downcast_mut()
    }

    pub fn insert<T: Any>(&mut self, key: PersistentStateKey, value: T) {
        self.storage_mut().insert(key, smallbox!(value));
    }
}

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

type NodeRendererStorage<Renderer> = SmallBox<dyn rendering::NodeRenderer<Renderer = Renderer>, 8>;

#[derive(Default)]
struct ByorGuiData<Renderer: rendering::Renderer> {
    text_layouts: PrimaryMap<TextLayoutId, TextLayout<Color>>,
    renderers: PrimaryMap<NodeRendererId, NodeRendererStorage<Renderer>>,
    persistent_state: IntMap<Uid, PersistentState>,
    previous_state: IntMap<Uid, PreviousState>,
    float_positions: IntMap<Uid, PersistentFloatPosition>,

    theme: Theme,
    scale_factor: f32,
    input_state: InputState,
    hovered_node_override: Option<Uid>,

    uid_stack: Vec<Uid>,
}

#[derive(Default)]
pub struct ByorGui<Renderer: rendering::Renderer> {
    forest: Forest<Node>,
    data: ByorGuiData<Renderer>,
}

#[must_use]
fn compute_previous_state<Renderer: rendering::Renderer>(
    tree: TreeRef<'_, Node, Shared>,
    data: &mut ByorGuiData<Renderer>,
    mouse_in_parent_clip_bounds: bool,
) -> Option<Uid> {
    let mut hovered_node = None;

    let TreeRef {
        parent: node,
        descendants,
        ..
    } = tree;

    let mouse_position = data.input_state.mouse_position();
    let mouse_in_bounds = mouse_in_parent_clip_bounds
        && point_in_rect(mouse_position, node.position, node.style.fixed_size);

    let (clip_position, clip_size) = node.clip_bounds();
    let mouse_in_clip_bounds =
        mouse_in_bounds && point_in_rect(mouse_position, clip_position, clip_size);

    iter_subtrees!(descendants => |subtree| {
        if subtree.is_root {
            continue;
        }

        if let Some(uid) = compute_previous_state(subtree, data, mouse_in_clip_bounds) {
            assert!(hovered_node.is_none(), "multiple nodes hovered");
            hovered_node = Some(uid);
        }
    });

    if let Some(uid) = node.uid {
        let mut child_count = 0u32;
        let mut total_content_size = Vec2::default();
        let mut max_content_size = Vec2::default();
        iter_children!(descendants => |child| {
            child_count += 1;
            total_content_size += child.style.fixed_size;
            max_content_size = max_content_size.max(child.style.fixed_size);
        });

        let total_spacing = (child_count.saturating_sub(1) as f32) * node.style.child_spacing();

        let state = data.previous_state.entry(uid).or_default();
        state.referenced = true; // this state is indeed still referenced

        state.hover_state = if let Some(hovered_node_override) = data.hovered_node_override {
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

impl<Renderer: rendering::Renderer> ByorGui<Renderer> {
    #[must_use]
    #[inline]
    pub fn theme(&self) -> &Theme {
        &self.data.theme
    }

    #[must_use]
    #[inline]
    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.data.theme
    }

    fn update_previous_states(&mut self) {
        if self.data.input_state.pressed_buttons().is_empty() {
            self.data.hovered_node_override = None;
        }

        let mut hovered_node = None;
        let mut trees = self.forest.trees();
        while let Some(tree) = trees.next() {
            // FIXME: floating nodes should stop nodes underneath from being hovered
            if let Some(uid) = compute_previous_state(tree, &mut self.data, true) {
                hovered_node = Some(uid);
            }
        }

        self.data.previous_state.retain(|_, state| state.referenced);

        if !self.data.input_state.pressed_buttons().is_empty() {
            self.data.hovered_node_override = hovered_node;
        }
    }

    #[must_use]
    #[inline(never)]
    fn begin_frame<'gui>(
        &'gui mut self,
        screen_size: Vec2<Pixel>,
        scale_factor: f32,
        mouse_state: MouseState,
    ) -> ByorGuiContext<'gui, Renderer> {
        self.data.text_layouts.clear();
        self.data.renderers.clear();
        self.data
            .previous_state
            .values_mut()
            .for_each(|state| state.referenced = false);
        self.data
            .float_positions
            .values_mut()
            .for_each(PersistentFloatPosition::reset_referenced);

        self.data.scale_factor = scale_factor;
        self.data.input_state.update(mouse_state);

        let input_state = NodeInputState::default();
        let root_style = self
            .data
            .theme
            .build_style(None, &[], Theme::ROOT_TYPE_CLASS);
        let cascaded_style = root_style.cascade_root(screen_size, input_state);
        let computed_style = compute_style(&root_style, &cascaded_style, None, scale_factor);
        let primary_builder = self.forest.insert_primary(Node::new_root(computed_style));

        ByorGuiContext {
            builder: primary_builder,
            data: &mut self.data,
            parent_style: cascaded_style,
            parent_input_state: input_state,
        }
    }

    #[inline(never)]
    fn end_frame(&mut self) {
        self.data.float_positions.retain(|_, pos| pos.referenced());
        self.layout();
        self.update_previous_states();
    }

    #[inline]
    pub fn frame<T>(
        &mut self,
        screen_size: Vec2<Pixel>,
        scale_factor: f32,
        mouse_state: MouseState,
        builder: impl FnOnce(ByorGuiContext<'_, Renderer>) -> T,
    ) -> T {
        let context = self.begin_frame(screen_size, scale_factor, mouse_state);
        let result = builder(context);
        self.end_frame();

        result
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NodeInputState {
    pub hover_state: HoverState,
    pub pressed_buttons: MouseButtons,
    pub clicked_buttons: MouseButtons,
    pub released_buttons: MouseButtons,
}

impl NodeInputState {
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

pub struct ByorGuiContext<'gui, Renderer: rendering::Renderer> {
    builder: ForestBuilder<'gui, Node>,
    data: &'gui mut ByorGuiData<Renderer>,
    parent_style: CascadedStyle,
    parent_input_state: NodeInputState,
}

impl<Renderer: rendering::Renderer> ByorGuiContext<'_, Renderer> {
    #[must_use]
    #[inline]
    pub fn theme(&self) -> &Theme {
        &self.data.theme
    }

    #[must_use]
    #[inline]
    pub fn scale_factor(&self) -> f32 {
        self.data.scale_factor
    }

    #[must_use]
    #[inline]
    pub fn parent_style(&self) -> &CascadedStyle {
        &self.parent_style
    }

    #[must_use]
    #[inline]
    pub fn computed_parent_style(&self) -> &ComputedStyle {
        &self.builder.parent_node().style
    }

    #[must_use]
    #[inline]
    pub fn global_input_state(&self) -> &InputState {
        &self.data.input_state
    }

    #[must_use]
    #[inline]
    pub fn parent_input_state(&self) -> NodeInputState {
        self.parent_input_state
    }

    #[must_use]
    #[inline]
    fn compute_recursive_uid(&self, uid: Uid) -> Uid {
        if let Some(&parent_uid) = self.data.uid_stack.last() {
            parent_uid.concat(uid)
        } else {
            uid
        }
    }

    #[must_use]
    pub fn persistent_state(&self, uid: Uid) -> &PersistentState {
        let uid = self.compute_recursive_uid(uid);
        self.data
            .persistent_state
            .get(uid)
            .unwrap_or(&PersistentState::EMPTY)
    }

    #[must_use]
    pub fn persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        let uid = self.compute_recursive_uid(uid);
        self.data.persistent_state.entry(uid).or_default()
    }

    #[must_use]
    pub fn previous_state(&self, uid: Uid) -> Option<&PreviousState> {
        let uid = self.compute_recursive_uid(uid);
        self.data.previous_state.get(uid)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeResponse<T> {
    pub input_state: NodeInputState,
    pub result: T,
}

impl<T> NodeResponse<T> {
    #[inline]
    pub fn map_result<U>(self, f: impl FnOnce(T) -> U) -> NodeResponse<U> {
        NodeResponse {
            input_state: self.input_state,
            result: f(self.result),
        }
    }

    #[inline]
    pub fn is_hovered(&self) -> bool {
        self.input_state.is_hovered()
    }

    #[inline]
    pub fn is_directly_hovered(&self) -> bool {
        self.input_state.is_directly_hovered()
    }

    #[inline]
    pub fn pressed(&self, buttons: MouseButtons) -> bool {
        self.input_state.pressed(buttons)
    }

    #[inline]
    pub fn clicked(&self, buttons: MouseButtons) -> bool {
        self.input_state.clicked(buttons)
    }

    #[inline]
    pub fn released(&self, buttons: MouseButtons) -> bool {
        self.input_state.released(buttons)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DuplicateUidError {
    location: &'static std::panic::Location<'static>,
}

impl fmt::Display for DuplicateUidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "duplicate UID at {}:{}:{}",
            self.location.file(),
            self.location.line(),
            self.location.column(),
        )
    }
}

impl std::error::Error for DuplicateUidError {}

pub type InsertNodeResult<T> = widgets::WidgetResult<NodeResponse<T>>;

pub trait GuiBuilder<Renderer: rendering::Renderer> {
    type Result;

    fn build(self, gui: ByorGuiContext<'_, Renderer>) -> Self::Result;
}

impl<Renderer, R, F> GuiBuilder<Renderer> for F
where
    Renderer: rendering::Renderer,
    F: FnOnce(ByorGuiContext<'_, Renderer>) -> R,
{
    type Result = R;

    #[inline]
    fn build(self, gui: ByorGuiContext<'_, Renderer>) -> Self::Result {
        self(gui)
    }
}

impl<Renderer: rendering::Renderer> GuiBuilder<Renderer> for () {
    type Result = ();

    #[inline]
    fn build(self, _gui: ByorGuiContext<'_, Renderer>) -> Self::Result {}
}

pub struct NodeContents<'text, Renderer, Builder = ()>
where
    Renderer: rendering::Renderer,
    Builder: GuiBuilder<Renderer>,
{
    pub text: Option<&'text str>,
    pub renderer: Option<NodeRendererStorage<Renderer>>,
    pub builder: Builder,
}

impl<'text, Renderer: rendering::Renderer> NodeContents<'text, Renderer> {
    pub const EMPTY: Self = Self {
        text: None,
        renderer: None,
        builder: (),
    };

    #[must_use]
    #[inline]
    pub const fn text(text: &'text str) -> Self {
        Self {
            text: Some(text),
            ..Self::EMPTY
        }
    }

    #[must_use]
    #[inline]
    pub fn renderer(renderer: impl NodeRenderer<Renderer = Renderer>) -> Self {
        Self {
            renderer: Some(smallbox!(renderer)),
            ..Self::EMPTY
        }
    }
}

impl<Renderer, R, F> NodeContents<'_, Renderer, F>
where
    Renderer: rendering::Renderer,
    F: FnOnce(ByorGuiContext<'_, Renderer>) -> R,
{
    #[must_use]
    #[inline]
    pub const fn builder(f: F) -> Self {
        Self {
            text: None,
            renderer: None,
            builder: f,
        }
    }
}

impl<Renderer: rendering::Renderer> ByorGuiContext<'_, Renderer> {
    #[must_use]
    #[inline]
    fn compute_node_input_state(&self, uid: Option<Uid>) -> NodeInputState {
        let hover_state = uid
            .and_then(|uid| self.data.previous_state.get(uid))
            .map(|previous_state| previous_state.hover_state)
            .unwrap_or_default();

        let (pressed_buttons, clicked_buttons, released_buttons) =
            if hover_state == HoverState::DirectlyHovered {
                (
                    self.data.input_state.pressed_buttons(),
                    self.data.input_state.clicked_buttons(),
                    self.data.input_state.released_buttons(),
                )
            } else {
                (
                    MouseButtons::empty(),
                    MouseButtons::empty(),
                    MouseButtons::empty(),
                )
            };

        NodeInputState {
            hover_state,
            pressed_buttons,
            clicked_buttons,
            released_buttons,
        }
    }

    #[must_use]
    #[inline]
    fn layout_text(&mut self, text: &str) -> TextLayoutId {
        use parley::style::{LineHeight, OverflowWrap, StyleProperty};

        global_cache::with_parley_global_data(|parley_global_data| {
            let mut builder = parley_global_data.builder(text, 1.0);

            let style = &self.builder.parent_node().style;
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

            self.data.text_layouts.push(builder.build(text))
        })
    }

    #[track_caller]
    #[must_use]
    #[inline(never)] // Don't inline this to avoid monomorphization duplication
    fn insert_leaf_node<'gui>(
        &'gui mut self,
        uid: Option<Uid>,
        style: &Style,
        is_root: bool,
        text: Option<&str>,
        renderer: Option<NodeRendererStorage<Renderer>>,
    ) -> widgets::WidgetResult<ByorGuiContext<'gui, Renderer>> {
        let input_state = self.compute_node_input_state(uid);
        let cascaded_style = style.cascade(&self.parent_style, input_state);
        let computed_style = compute_style(
            style,
            &cascaded_style,
            Some(&self.builder.parent_node().style),
            self.data.scale_factor,
        );

        let text_layout = text.map(|text| self.layout_text(text));
        let renderer = renderer.map(|renderer| self.data.renderers.push(renderer));
        let node = Node::new(uid, text_layout, renderer, computed_style);
        let builder = self.builder.insert(node, is_root);

        if let Some(uid) = uid {
            let prev_state = self.data.previous_state.entry(uid).or_default();
            if prev_state.referenced {
                return Err(DuplicateUidError {
                    location: std::panic::Location::caller(),
                });
            }
            prev_state.referenced = true;
        }

        Ok(ByorGuiContext {
            builder,
            data: self.data,
            parent_style: cascaded_style,
            parent_input_state: input_state,
        })
    }

    #[inline(never)] // Don't inline this to avoid monomorphization duplication
    fn update_float_position(&mut self, uid: Uid, position: FloatPosition) {
        match position {
            FloatPosition::Cursor => {
                let cursor_position = self.data.input_state.mouse_position();

                self.data.float_positions.insert(
                    uid,
                    PersistentFloatPosition::Cursor {
                        referenced: true,
                        x: cursor_position.x,
                        y: cursor_position.y,
                    },
                );
            }
            FloatPosition::CursorFixed => {
                let persistent_position = self.data.float_positions.entry(uid).or_default();

                if let PersistentFloatPosition::CursorFixed { referenced, .. } = persistent_position
                {
                    *referenced = true;
                } else {
                    let cursor_position = self.data.input_state.mouse_position();

                    self.data.float_positions.insert(
                        uid,
                        PersistentFloatPosition::CursorFixed {
                            referenced: true,
                            x: cursor_position.x,
                            y: cursor_position.y,
                        },
                    );
                }
            }
            FloatPosition::Fixed { x, y } => {
                let parent_font_size = self.builder.parent_node().style.font_size().value();

                self.data.float_positions.insert(
                    uid,
                    PersistentFloatPosition::Fixed {
                        referenced: true,
                        x: x.to_pixel(self.data.scale_factor, parent_font_size),
                        y: y.to_pixel(self.data.scale_factor, parent_font_size),
                    },
                );
            }
            FloatPosition::Popup { x, y } => {
                self.data.float_positions.insert(
                    uid,
                    PersistentFloatPosition::Popup {
                        referenced: true,
                        x,
                        y,
                    },
                );
            }
        }
    }

    pub fn uid_scope<R>(
        &mut self,
        uid: Uid,
        contents: impl FnOnce(&mut ByorGuiContext<'_, Renderer>) -> R,
    ) -> R {
        let uid = self.compute_recursive_uid(uid);
        self.data.uid_stack.push(uid);
        let result = contents(self);
        self.data.uid_stack.pop();
        result
    }

    #[track_caller]
    pub fn insert_node<Builder: GuiBuilder<Renderer>>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: NodeContents<Renderer, Builder>,
    ) -> InsertNodeResult<Builder::Result> {
        let uid = uid.map(|uid| self.compute_recursive_uid(uid));
        let context = self.insert_leaf_node(uid, style, false, contents.text, contents.renderer)?;

        Ok(NodeResponse {
            input_state: context.parent_input_state,
            result: contents.builder.build(context),
        })
    }

    #[track_caller]
    pub fn insert_floating_node<Builder: GuiBuilder<Renderer>>(
        &mut self,
        uid: Uid,
        position: FloatPosition,
        style: &Style,
        contents: NodeContents<Renderer, Builder>,
    ) -> InsertNodeResult<Builder::Result> {
        let uid = self.compute_recursive_uid(uid);
        self.update_float_position(uid, position);
        let context =
            self.insert_leaf_node(Some(uid), style, true, contents.text, contents.renderer)?;

        Ok(NodeResponse {
            input_state: context.parent_input_state,
            result: contents.builder.build(context),
        })
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
