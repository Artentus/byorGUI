#![feature(trait_alias)]

mod layout;
pub mod rendering;

use color::{AlphaColor, Srgb};
use intmap::{IntKey, IntMap};
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use std::ops::Deref;

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

#[derive(Debug, Clone)]
pub enum Brush {
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
    pub fn unwrap_or(self, value: T) -> T {
        match self {
            Self::Inherit => value,
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
    pub width: Sizing,
    pub height: Sizing,
    pub min_width: Option<Pixel>,
    pub min_height: Option<Pixel>,
    pub max_width: Option<Pixel>,
    pub max_height: Option<Pixel>,
    pub flex_ratio: Option<f32>,

    pub padding: Property<Padding>,
    pub child_spacing: Property<Pixel>,
    pub layout_direction: Property<Direction>,
    pub child_alignment: Property<Alignment>,
    pub cross_axis_alignment: Property<Alignment>,
    pub background: Property<Brush>,
    pub foreground: Property<Brush>,
    pub font_size: Property<Pixel>,
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
    pub font_size: Pixel,
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
            font_size: 16.0,
        }
    }
}

#[derive(Debug, Clone)]
struct ComputedStyle {
    width: Sizing,
    height: Sizing,
    flex_ratio: f32,

    padding: Padding,
    child_spacing: Pixel,
    layout_direction: Direction,
    child_alignment: Alignment,
    cross_axis_alignment: Alignment,
    background: Brush,
    foreground: Brush,
    font_size: Pixel,
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
            font_size: style.font_size,
        }
    }
}

fn compute_style(style: &Style, base: &ComputedStyle) -> ComputedStyle {
    ComputedStyle {
        width: style.width,
        height: style.height,
        flex_ratio: style.flex_ratio.unwrap_or(1.0),

        padding: style.padding.unwrap_or(base.padding),
        child_spacing: style.child_spacing.unwrap_or(base.child_spacing),
        layout_direction: style.layout_direction.unwrap_or(base.layout_direction),
        child_alignment: style.child_alignment.unwrap_or(base.child_alignment),
        cross_axis_alignment: style
            .cross_axis_alignment
            .unwrap_or(base.cross_axis_alignment),
        background: style.background.clone().unwrap_or(base.background.clone()),
        foreground: style.foreground.clone().unwrap_or(base.foreground.clone()),
        font_size: style.font_size.unwrap_or(base.font_size),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Uid(u64);

impl IntKey for Uid {
    type Int = u64;

    // values are pre-hashed so we don't need to "hash" again
    const PRIME: Self::Int = 1;

    fn into_int(self) -> Self::Int {
        self.0
    }
}

#[must_use]
pub const fn uid(value: &[u8]) -> Uid {
    Uid(rapidhash::v3::rapidhash_v3(value))
}

slotmap::new_key_type! { struct NodeId; }

struct Node {
    uid: Option<Uid>,
    style: ComputedStyle,
    text: Option<String>,
    min_size: Size,
    max_size: Size,
    size: Size,
    position: Position,
}

impl Node {
    fn new_root(style: &RootStyle, screen_size: Size) -> Self {
        let style = ComputedStyle::from_root_style(style, screen_size);

        Node {
            uid: None,
            style,
            text: None,
            min_size: screen_size,
            max_size: screen_size,
            size: Size::default(),
            position: Position::default(),
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
        let style = compute_style(style, parent_style);

        Self {
            uid,
            style,
            text: None,
            min_size,
            max_size,
            size: Size::default(),
            position: Position::default(),
        }
    }
}

const INLINE_NODE_ID_COUNT: usize = (2 * size_of::<usize>()) / size_of::<NodeId>();
type NodeIdVec = SmallVec<[NodeId; INLINE_NODE_ID_COUNT]>;

#[derive(Default)]
pub struct ByorGui {
    nodes: SlotMap<NodeId, Node>,
    children: SecondaryMap<NodeId, NodeIdVec>,
    uid_map: IntMap<Uid, NodeId>,
    ancestor_stack: Vec<NodeId>,

    root_style: RootStyle,
    prev_mouse_state: MouseState,
    mouse_state: MouseState,
    hovered_node_uid: Option<Uid>,
}

pub trait UiContents<R> = FnOnce(&mut ByorGui) -> R;

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
        assert!(self.ancestor_stack.is_empty());

        let root = Node::new_root(self.root_style(), screen_size);
        let root_id = self.nodes.insert(root);
        self.ancestor_stack.push(root_id);

        self.prev_mouse_state = self.mouse_state;
        self.mouse_state = mouse_state;
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
        let hovered = self.hovered_node_uid.zip(uid).is_some_and(|(a, b)| a == b);
        let clicked = hovered && self.mouse_state.button1_pressed;

        NodeResponse {
            hovered,
            clicked,
            result,
        }
    }

    pub fn insert_container_node<R>(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        contents: impl UiContents<R>,
    ) -> NodeResponse<R> {
        let node_id = self.insert_leaf_node(uid, style);

        self.ancestor_stack.push(node_id);
        let result = contents(self);
        assert!(self.ancestor_stack.pop().is_some());

        self.compute_node_response(uid, result)
    }

    pub fn insert_text_node(
        &mut self,
        uid: Option<Uid>,
        style: &Style,
        text: String,
    ) -> NodeResponse<()> {
        let node_id = self.insert_leaf_node(uid, style);
        self.nodes[node_id].text = Some(text);

        self.compute_node_response(uid, ())
    }

    pub fn end_frame<R: rendering::Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let root_id = self.ancestor_stack.pop().unwrap();
        assert!(self.ancestor_stack.is_empty());

        self.layout(root_id);
        // TODO: find hovered element
        self.render(root_id, renderer)
    }
}

#[cfg(feature = "vello")]
mod vello_impls;
