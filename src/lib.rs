mod layout;
pub mod rendering;
pub mod widgets;

use color::{AlphaColor, Srgb};
use intmap::{IntKey, IntMap};
use parley::FontContext;
use parley::LayoutContext as TextLayoutContext;
use parley::layout::Layout as TextLayout;
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::ops::Deref;

pub use parley::Alignment as HorizontalTextAlignment;
pub use parley::style::{FontFamily, FontStack, FontStyle, FontWeight, FontWidth, GenericFamily};

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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Sizing {
    #[default]
    FitContent,
    Grow,
    Fixed(Pixel),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Padding {
    pub left: Pixel,
    pub right: Pixel,
    pub top: Pixel,
    pub bottom: Pixel,
}

impl From<Pixel> for Padding {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

impl From<(Pixel, Pixel)> for Padding {
    #[inline]
    fn from(value: (Pixel, Pixel)) -> Self {
        Self {
            left: value.0,
            right: value.0,
            top: value.1,
            bottom: value.1,
        }
    }
}

impl From<(Pixel, Pixel, Pixel, Pixel)> for Padding {
    #[inline]
    fn from(value: (Pixel, Pixel, Pixel, Pixel)) -> Self {
        Self {
            left: value.0,
            right: value.1,
            top: value.2,
            bottom: value.3,
        }
    }
}

impl From<[Pixel; 2]> for Padding {
    #[inline]
    fn from(value: [Pixel; 2]) -> Self {
        Self {
            left: value[0],
            right: value[0],
            top: value[1],
            bottom: value[1],
        }
    }
}

impl From<[Pixel; 4]> for Padding {
    #[inline]
    fn from(value: [Pixel; 4]) -> Self {
        Self {
            left: value[0],
            right: value[1],
            top: value[2],
            bottom: value[3],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    #[default]
    LeftToRight,
    TopToBottom,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalTextAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Brush {
    #[default]
    None,
    Solid(AlphaColor<Srgb>),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState {
    pub position: Position,
    pub button1_pressed: bool,
    pub button2_pressed: bool,
    pub button3_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property<T> {
    Inherit,
    Override(T),
}

impl<T> Default for Property<T> {
    #[inline]
    fn default() -> Self {
        Self::Inherit
    }
}

impl<T> Property<T> {
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Inherit => default,
            Self::Override(value) => value,
        }
    }

    #[inline]
    pub fn unwrap_or_else(self, f: impl FnOnce() -> T) -> T {
        match self {
            Self::Inherit => f(),
            Self::Override(value) => value,
        }
    }
}

impl<T: Clone> Property<&T> {
    #[inline]
    pub fn cloned(self) -> Property<T> {
        match self {
            Self::Inherit => Property::Inherit,
            Self::Override(value) => Property::Override(value.clone()),
        }
    }
}

impl<T: Copy> Property<&T> {
    #[inline]
    pub fn copied(self) -> Property<T> {
        match self {
            Self::Inherit => Property::Inherit,
            Self::Override(value) => Property::Override(*value),
        }
    }
}

impl<T> From<T> for Property<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Override(value)
    }
}

impl<T> From<Option<T>> for Property<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::Inherit,
            Some(value) => Self::Override(value),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Style {
    // non-inherited properties
    pub width: Sizing,
    pub height: Sizing,
    pub min_width: Option<Pixel>,
    pub min_height: Option<Pixel>,
    pub max_width: Option<Pixel>,
    pub max_height: Option<Pixel>,
    pub flex_ratio: Option<f32>,
    pub allow_horizontal_scoll: bool,
    pub allow_vertical_scoll: bool,

    // inherited properties
    pub padding: Property<Padding>,
    pub child_spacing: Property<Pixel>,
    pub layout_direction: Property<Direction>,
    pub child_alignment: Property<Alignment>,
    pub cross_axis_alignment: Property<Alignment>,
    pub background: Property<Brush>,
    pub foreground: Property<Brush>,
    pub font: Property<FontStack<'static>>,
    pub font_size: Property<Pixel>,
    pub font_weight: Property<FontWeight>,
    pub font_width: Property<FontWidth>,
    pub text_underline: Property<bool>,
    pub text_strikethrough: Property<bool>,
    pub allow_text_wrap: Property<bool>,
    pub horizontal_text_alignment: Property<HorizontalTextAlignment>,
    pub vertical_text_alignment: Property<VerticalTextAlignment>,
}

#[derive(Debug, Clone)]
pub struct RootStyle {
    pub padding: Padding,
    pub child_spacing: Pixel,
    pub layout_direction: Direction,
    pub child_alignment: Alignment,
    pub cross_axis_alignment: Alignment,
    pub background: Brush,
    pub foreground: Brush,
    pub font: FontStack<'static>,
    pub font_size: Pixel,
    pub font_weight: FontWeight,
    pub font_width: FontWidth,
    pub text_underline: bool,
    pub text_strikethrough: bool,
    pub allow_text_wrap: bool,
    pub horizontal_text_alignment: HorizontalTextAlignment,
    pub vertical_text_alignment: VerticalTextAlignment,
}

impl Default for RootStyle {
    fn default() -> Self {
        Self {
            padding: Padding::default(),
            child_spacing: 0.0,
            layout_direction: Direction::default(),
            child_alignment: Alignment::default(),
            cross_axis_alignment: Alignment::default(),
            background: Brush::Solid(AlphaColor::BLACK),
            foreground: Brush::Solid(AlphaColor::WHITE),
            font: FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi)),
            font_size: 16.0,
            font_weight: FontWeight::NORMAL,
            font_width: FontWidth::NORMAL,
            text_underline: false,
            text_strikethrough: false,
            allow_text_wrap: true,
            horizontal_text_alignment: HorizontalTextAlignment::default(),
            vertical_text_alignment: VerticalTextAlignment::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub width: Sizing,
    pub height: Sizing,
    pub flex_ratio: f32,

    pub padding: Padding,
    pub child_spacing: Pixel,
    pub layout_direction: Direction,
    pub child_alignment: Alignment,
    pub cross_axis_alignment: Alignment,
    pub background: Brush,
    pub foreground: Brush,
    pub font: FontStack<'static>,
    pub font_size: Pixel,
    pub font_weight: FontWeight,
    pub font_width: FontWidth,
    pub text_underline: bool,
    pub text_strikethrough: bool,
    pub allow_text_wrap: bool,
    pub horizontal_text_alignment: HorizontalTextAlignment,
    pub vertical_text_alignment: VerticalTextAlignment,
}

impl ComputedStyle {
    fn from_root_style(style: &RootStyle, screen_size: Size) -> Self {
        Self {
            width: Sizing::Fixed(screen_size.width),
            height: Sizing::Fixed(screen_size.height),
            flex_ratio: 1.0,

            padding: style.padding,
            child_spacing: style.child_spacing,
            layout_direction: style.layout_direction,
            child_alignment: style.child_alignment,
            cross_axis_alignment: style.cross_axis_alignment,
            background: style.background.clone(),
            foreground: style.foreground.clone(),
            font: style.font.clone(),
            font_size: style.font_size,
            font_weight: style.font_weight,
            font_width: style.font_width,
            text_underline: style.text_underline,
            text_strikethrough: style.text_strikethrough,
            allow_text_wrap: style.allow_text_wrap,
            horizontal_text_alignment: style.horizontal_text_alignment,
            vertical_text_alignment: style.vertical_text_alignment,
        }
    }

    pub fn into_style(self) -> Style {
        Style {
            width: self.width,
            height: self.height,
            flex_ratio: Some(self.flex_ratio),

            padding: Property::Override(self.padding),
            child_spacing: Property::Override(self.child_spacing),
            layout_direction: Property::Override(self.layout_direction),
            child_alignment: Property::Override(self.child_alignment),
            cross_axis_alignment: Property::Override(self.cross_axis_alignment),
            background: Property::Override(self.background),
            foreground: Property::Override(self.foreground),
            font: Property::Override(self.font),
            font_size: Property::Override(self.font_size),
            font_weight: Property::Override(self.font_weight),
            font_width: Property::Override(self.font_width),
            text_underline: Property::Override(self.text_underline),
            text_strikethrough: Property::Override(self.text_strikethrough),
            allow_text_wrap: Property::Override(self.allow_text_wrap),
            horizontal_text_alignment: Property::Override(self.horizontal_text_alignment),
            vertical_text_alignment: Property::Override(self.vertical_text_alignment),

            ..Default::default()
        }
    }
}

impl Style {
    #[must_use]
    pub fn compute(&self, base: &ComputedStyle) -> ComputedStyle {
        ComputedStyle {
            width: self.width,
            height: self.height,
            flex_ratio: self.flex_ratio.unwrap_or(1.0),

            padding: self.padding.unwrap_or(base.padding),
            child_spacing: self.child_spacing.unwrap_or(base.child_spacing),
            layout_direction: self.layout_direction.unwrap_or(base.layout_direction),
            child_alignment: self.child_alignment.unwrap_or(base.child_alignment),
            cross_axis_alignment: self
                .cross_axis_alignment
                .unwrap_or(base.cross_axis_alignment),
            background: self
                .background
                .clone()
                .unwrap_or_else(|| base.background.clone()),
            foreground: self
                .foreground
                .clone()
                .unwrap_or_else(|| base.foreground.clone()),
            font: self.font.clone().unwrap_or_else(|| base.font.clone()),
            font_size: self.font_size.unwrap_or(base.font_size),
            font_weight: self.font_weight.unwrap_or(base.font_weight),
            font_width: self.font_width.unwrap_or(base.font_width),
            text_underline: self.text_underline.unwrap_or(base.text_underline),
            text_strikethrough: self.text_strikethrough.unwrap_or(base.text_strikethrough),
            allow_text_wrap: self.allow_text_wrap.unwrap_or(base.allow_text_wrap),
            horizontal_text_alignment: self
                .horizontal_text_alignment
                .unwrap_or(base.horizontal_text_alignment),
            vertical_text_alignment: self
                .vertical_text_alignment
                .unwrap_or(base.vertical_text_alignment),
        }
    }
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
    fn new_root(style: &RootStyle, screen_size: Size) -> Self {
        let style = ComputedStyle::from_root_style(style, screen_size);

        Node {
            uid: None,
            style,
            text_layout: None,
            min_size: screen_size,
            max_size: screen_size,
            size: Size::default(),
            position: Position::default(),
            vertical_text_offset: 0.0,
        }
    }

    fn new(uid: Option<Uid>, style: &Style, parent_style: &ComputedStyle) -> Self {
        let min_size = Size {
            width: style.min_width.unwrap_or(0.0),
            height: style.min_height.unwrap_or(0.0),
        };
        let max_size = Size {
            width: style.max_width.unwrap_or(Pixel::MAX),
            height: style.max_height.unwrap_or(Pixel::MAX),
        };
        let style = style.compute(parent_style);

        Self {
            uid,
            style,
            text_layout: None,
            min_size,
            max_size,
            size: Size::default(),
            position: Position::default(),
            vertical_text_offset: 0.0,
        }
    }
}

#[derive(Default)]
pub struct PersistentState {
    pub horizontal_scroll: Option<Pixel>,
    pub vertical_scroll: Option<Pixel>,
}

const INLINE_NODE_ID_COUNT: usize = (2 * size_of::<usize>()) / size_of::<NodeId>();
type NodeIdVec = SmallVec<[NodeId; INLINE_NODE_ID_COUNT]>;

#[derive(Default)]
pub struct ByorGui {
    nodes: SlotMap<NodeId, Node>,
    children: SecondaryMap<NodeId, NodeIdVec>,
    uid_map: IntMap<Uid, NodeId>,
    ancestor_stack: Vec<NodeId>,
    text_layouts: SlotMap<TextLayoutId, TextLayout<Brush>>,
    persistent_state: IntMap<Uid, PersistentState>,

    root_style: RootStyle,
    prev_mouse_state: MouseState,
    mouse_state: MouseState,
    hovered_node_uid: Option<Uid>,

    text_layout_context: TextLayoutContext<Brush>,
    font_context: FontContext,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeResponse<R> {
    pub hovered: bool,
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
    pub fn root_style(&self) -> &RootStyle {
        &self.root_style
    }

    #[must_use]
    #[inline]
    pub fn root_style_mut(&mut self) -> &mut RootStyle {
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

    fn find_hovered_element(&self, node_id: NodeId) -> Option<Uid> {
        for &child_id in self.child_ids(node_id) {
            if let Some(uid) = self.find_hovered_element(child_id) {
                return Some(uid);
            }
        }

        let node = &self.nodes[node_id];
        if let Some(uid) = node.uid {
            let mouse_position = self.mouse_state.position;
            if (mouse_position.x >= node.position.x)
                && (mouse_position.x <= node.position.x + node.size.width)
                && (mouse_position.y >= node.position.y)
                && (mouse_position.y <= node.position.y + node.size.height)
            {
                return Some(uid);
            }
        }

        None
    }

    pub fn end_frame<R: rendering::Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let root_id = self.ancestor_stack.pop().unwrap();
        assert!(self.ancestor_stack.is_empty());

        self.layout(root_id);
        self.hovered_node_uid = self.find_hovered_element(root_id);
        self.render(root_id, renderer)
    }

    fn update_persistent_state(&mut self, uid: Uid, style: &Style) {
        let state = self
            .persistent_state
            .entry(uid)
            .or_insert(PersistentState::default());

        if style.allow_horizontal_scoll {
            if state.horizontal_scroll.is_none() {
                state.horizontal_scroll = Some(0.0);
            }
        } else {
            state.horizontal_scroll = None;
        }

        if style.allow_vertical_scoll {
            if state.vertical_scroll.is_none() {
                state.vertical_scroll = Some(0.0);
            }
        } else {
            state.vertical_scroll = None;
        }
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
            self.update_persistent_state(uid, style);
        }

        node_id
    }

    #[must_use]
    fn compute_node_response<R>(&self, uid: Option<Uid>, result: R) -> NodeResponse<R> {
        let hovered = self.hovered_node_uid.zip(uid).is_some_and(|(a, b)| a == b);
        let clicked = hovered && self.mouse_state.button1_pressed;

        NodeResponse {
            hovered,
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
        builder.push_default(StyleProperty::Brush(style.foreground.clone()));
        builder.push_default(StyleProperty::FontStack(style.font.clone()));
        builder.push_default(StyleProperty::FontSize(style.font_size));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(1.3)));
        builder.push_default(StyleProperty::FontWeight(style.font_weight));
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

    fn get_persistent_state(&self, uid: Uid) -> Option<&PersistentState>;

    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState;
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

    #[inline]
    fn get_persistent_state(&self, uid: Uid) -> Option<&PersistentState> {
        self.persistent_state.get(uid)
    }

    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        self.persistent_state
            .entry(uid)
            .or_insert(PersistentState::default())
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
    fn get_persistent_state(&self, uid: Uid) -> Option<&PersistentState> {
        self.gui.get_persistent_state(uid)
    }

    #[inline]
    fn get_persistent_state_mut(&mut self, uid: Uid) -> &mut PersistentState {
        self.gui.get_persistent_state_mut(uid)
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
