use super::*;

use byor_gui_procmacro::StyleBuilder;
use modular_bitfield::prelude::*;
pub use parley::{FontFamily, FontStack, FontStyle, FontWeight, FontWidth, GenericFamily};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Sizing {
    #[default]
    FitContent,
    Grow,
    Fixed(Pixel),
}

impl From<Pixel> for Sizing {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self::Fixed(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub left: Pixel,
    pub right: Pixel,
    pub top: Pixel,
    pub bottom: Pixel,
}

impl Padding {
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    };
}

impl Default for Padding {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
pub enum Direction {
    #[default]
    LeftToRight,
    TopToBottom,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
#[bits = 2]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
#[bits = 3]
pub enum HorizontalTextAlignment {
    /// This is [`HorizontalTextAlignment::Left`] for LTR text and [`HorizontalTextAlignment::Right`] for RTL text.
    #[default]
    Start,
    /// This is [`HorizontalTextAlignment::Right`] for LTR text and [`HorizontalTextAlignment::Left`] for RTL text.
    End,
    /// Align content to the left edge.
    ///
    /// For alignment that should be aware of text direction, use [`HorizontalTextAlignment::Start`] or
    /// [`HorizontalTextAlignment::End`] instead.
    Left,
    /// Align each line centered within the container.
    Center,
    /// Align content to the right edge.
    ///
    /// For alignment that should be aware of text direction, use [`HorizontalTextAlignment::Start`] or
    /// [`HorizontalTextAlignment::End`] instead.
    Right,
    /// Justify each line by spacing out content, except for the last line.
    Justify,
}

impl From<HorizontalTextAlignment> for parley::Alignment {
    #[inline]
    fn from(value: HorizontalTextAlignment) -> Self {
        match value {
            HorizontalTextAlignment::Start => Self::Start,
            HorizontalTextAlignment::End => Self::End,
            HorizontalTextAlignment::Left => Self::Left,
            HorizontalTextAlignment::Center => Self::Center,
            HorizontalTextAlignment::Right => Self::Right,
            HorizontalTextAlignment::Justify => Self::Justify,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
#[bits = 2]
pub enum VerticalTextAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const BLACK: Self = Self::greyscale(0);
    pub const WHITE: Self = Self::greyscale(255);

    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    pub const fn greyscale(value: u8) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
            a: 255,
        }
    }
}

impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property<T> {
    /// A property-specific default value (not necessarily the same as [`Default::default()`])
    Initial,
    /// The same value as its parent, or the initial value in case of the root
    Inherit,
    /// A specific value
    Value(T),
}

impl<T> Property<T> {
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Initial | Self::Inherit => default,
            Self::Value(value) => value,
        }
    }

    #[inline]
    pub fn unwrap_or_else(self, f: impl FnOnce() -> T) -> T {
        match self {
            Self::Initial | Self::Inherit => f(),
            Self::Value(value) => value,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> Property<&T> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(value),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Property<&mut T> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(value),
        }
    }
}

impl<T: Deref> Property<T> {
    #[inline]
    pub fn as_deref(&self) -> Property<&<T as Deref>::Target> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(value.deref()),
        }
    }
}

impl<T: Clone> Property<&T> {
    #[inline]
    pub fn cloned(self) -> Property<T> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(value.clone()),
        }
    }
}

impl<T: Copy> Property<&T> {
    #[inline]
    pub fn copied(self) -> Property<T> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(*value),
        }
    }
}

impl<T> From<T> for Property<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

macro_rules! define_style {
    ($($property_name:ident: $property_type:ty,)*) => {
        #[derive(Debug, Clone, StyleBuilder)]
        pub struct Style {
            $(pub $property_name: Property<$property_type>,)*
        }
    };
}

define_style! {
    width: Sizing,
    height: Sizing,
    min_width: Pixel,
    min_height: Pixel,
    max_width: Pixel,
    max_height: Pixel,
    flex_ratio: f32,
    padding: Padding,
    child_spacing: Pixel,
    layout_direction: Direction,
    child_alignment: Alignment,
    cross_axis_alignment: Alignment,
    background: Color,
    corner_radius: Pixel,
    border_width: Pixel,
    border_color: Color,
    font_family: FontStack<'static>,
    font_size: Pixel,
    font_style: FontStyle,
    font_weight: FontWeight,
    font_width: FontWidth,
    text_underline: bool,
    text_strikethrough: bool,
    text_wrap: bool,
    text_color: Color,
    horizontal_text_alignment: HorizontalTextAlignment,
    vertical_text_alignment: VerticalTextAlignment,
}

impl Style {
    pub const DEFAULT: Self = Self {
        width: Property::Initial,
        height: Property::Initial,
        min_width: Property::Initial,
        min_height: Property::Initial,
        max_width: Property::Initial,
        max_height: Property::Initial,
        flex_ratio: Property::Initial,
        padding: Property::Initial,
        child_spacing: Property::Initial,
        layout_direction: Property::Initial,
        child_alignment: Property::Initial,
        cross_axis_alignment: Property::Initial,
        background: Property::Initial,
        corner_radius: Property::Initial,
        border_width: Property::Initial,
        border_color: Property::Initial,
        font_family: Property::Inherit,
        font_style: Property::Inherit,
        font_size: Property::Inherit,
        font_weight: Property::Inherit,
        font_width: Property::Inherit,
        text_underline: Property::Inherit,
        text_strikethrough: Property::Inherit,
        text_wrap: Property::Inherit,
        text_color: Property::Inherit,
        horizontal_text_alignment: Property::Inherit,
        vertical_text_alignment: Property::Inherit,
    };
}

impl Default for Style {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

slotmap::new_key_type! {
    struct PaddingId;

    struct FontId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
#[bits = 2]
enum ComputedSizing {
    FitContent,
    Grow,
    Fixed,
}

struct ComputedFont {
    family: FontStack<'static>,
    size: Pixel,
    style: FontStyle,
    weight: FontWeight,
    width: FontWidth,
}

impl Default for ComputedFont {
    #[inline]
    fn default() -> Self {
        Self {
            family: FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi)),
            size: 16.0,
            style: FontStyle::Normal,
            weight: FontWeight::NORMAL,
            width: FontWidth::NORMAL,
        }
    }
}

#[bitfield(bits = 17)]
struct ComputedStylePackedFields {
    width: ComputedSizing,
    height: ComputedSizing,
    layout_direction: Direction,
    child_alignment: Alignment,
    cross_axis_alignment: Alignment,
    text_underline: bool,
    text_strikethrough: bool,
    text_wrap: bool,
    horizontal_text_alignment: HorizontalTextAlignment,
    vertical_text_alignment: VerticalTextAlignment,
}

pub struct ComputedStyle {
    packed_fields: ComputedStylePackedFields,

    min_width: Pixel,
    min_height: Pixel,
    max_width: Pixel,
    max_height: Pixel,
    flex_ratio: f32,
    padding: Arc<Padding>,
    child_spacing: Pixel,
    background: Color,
    corner_radius: Pixel,
    border_width: Pixel,
    border_color: Color,
    font: Arc<ComputedFont>,
    text_color: Color,
}

const INITIAL_SIZE: ComputedSizing = ComputedSizing::FitContent;
const INITIAL_MIN_SIZE: Pixel = 0.0;
const INITIAL_MAX_SIZE: Pixel = Pixel::MAX;
const INITIAL_FLEX_RATIO: f32 = 1.0;
const INITIAL_CHILD_SPACING: Pixel = 0.0;
const INITIAL_LAYOUT_DIRECTION: Direction = Direction::LeftToRight;
const INITIAL_ALIGNMENT: Alignment = Alignment::Start;
const INITIAL_BACKGROUND: Color = Color::TRANSPARENT;
const INITIAL_CORNER_RADIUS: Pixel = 0.0;
const INITIAL_BORDER_WIDTH: Pixel = 0.0;
const INITIAL_BORDER_COLOR: Color = Color::TRANSPARENT;
const INITIAL_TEXT_UNDERLINE: bool = false;
const INITIAL_TEXT_STRIKETHROUGH: bool = false;
const INITIAL_TEXT_WRAP: bool = true;
const INITIAL_TEXT_COLOR: Color = Color::BLACK;
const INITIAL_HORIZONTAL_TEXT_ALIGNMENT: HorizontalTextAlignment = HorizontalTextAlignment::Start;
const INITIAL_VERTICAL_TEXT_ALIGNMENT: VerticalTextAlignment = VerticalTextAlignment::Top;

static INITIAL_PADDING: LazyLock<Arc<Padding>> = LazyLock::new(|| Arc::new(Padding::ZERO));

static INITIAL_FONT: LazyLock<Arc<ComputedFont>> = LazyLock::new(|| {
    Arc::new(ComputedFont {
        family: FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi)),
        size: 16.0,
        style: FontStyle::Normal,
        weight: FontWeight::NORMAL,
        width: FontWidth::NORMAL,
    })
});

fn compute_root_font(style: &Style) -> ComputedFont {
    ComputedFont {
        family: match &style.font_family {
            Property::Initial | Property::Inherit => INITIAL_FONT.family.clone(),
            Property::Value(value) => value.clone(),
        },
        size: match style.font_size {
            Property::Initial | Property::Inherit => INITIAL_FONT.size,
            Property::Value(value) => value,
        },
        style: match style.font_style {
            Property::Initial | Property::Inherit => INITIAL_FONT.style,
            Property::Value(value) => value,
        },
        weight: match style.font_weight {
            Property::Initial | Property::Inherit => INITIAL_FONT.weight,
            Property::Value(value) => value,
        },
        width: match style.font_width {
            Property::Initial | Property::Inherit => INITIAL_FONT.width,
            Property::Value(value) => value,
        },
    }
}

fn compute_font(style: &Style, parent_style: &ComputedStyle) -> ComputedFont {
    ComputedFont {
        family: match &style.font_family {
            Property::Initial => INITIAL_FONT.family.clone(),
            Property::Inherit => parent_style.font.family.clone(),
            Property::Value(value) => value.clone(),
        },
        size: match style.font_size {
            Property::Initial => INITIAL_FONT.size,
            Property::Inherit => parent_style.font.size,
            Property::Value(value) => value,
        },
        style: match style.font_style {
            Property::Initial => INITIAL_FONT.style,
            Property::Inherit => parent_style.font.style,
            Property::Value(value) => value,
        },
        weight: match style.font_weight {
            Property::Initial => INITIAL_FONT.weight,
            Property::Inherit => parent_style.font.weight,
            Property::Value(value) => value,
        },
        width: match style.font_width {
            Property::Initial => INITIAL_FONT.width,
            Property::Inherit => parent_style.font.width,
            Property::Value(value) => value,
        },
    }
}

impl Style {
    pub fn compute_root(&self, screen_size: Vec2) -> ComputedStyle {
        macro_rules! compute_property {
            ($property:ident, $initial:expr) => {
                match &self.$property {
                    Property::Initial | Property::Inherit => const { $initial },
                    Property::Value(value) => value.clone(),
                }
            };
        }

        let flex_ratio = compute_property!(flex_ratio, INITIAL_FLEX_RATIO);
        let child_spacing = compute_property!(child_spacing, INITIAL_CHILD_SPACING);
        let layout_direction = compute_property!(layout_direction, INITIAL_LAYOUT_DIRECTION);
        let child_alignment = compute_property!(child_alignment, INITIAL_ALIGNMENT);
        let cross_axis_alignment = compute_property!(cross_axis_alignment, INITIAL_ALIGNMENT);
        let background = compute_property!(background, INITIAL_BACKGROUND);
        let corner_radius = compute_property!(corner_radius, INITIAL_CORNER_RADIUS);
        let border_width = compute_property!(border_width, INITIAL_BORDER_WIDTH);
        let border_color = compute_property!(border_color, INITIAL_BORDER_COLOR);
        let text_underline = compute_property!(text_underline, INITIAL_TEXT_UNDERLINE);
        let text_strikethrough = compute_property!(text_strikethrough, INITIAL_TEXT_STRIKETHROUGH);
        let text_wrap = compute_property!(text_wrap, INITIAL_TEXT_WRAP);
        let text_color = compute_property!(text_color, INITIAL_TEXT_COLOR);
        let horizontal_text_alignment =
            compute_property!(horizontal_text_alignment, INITIAL_HORIZONTAL_TEXT_ALIGNMENT);
        let vertical_text_alignment =
            compute_property!(vertical_text_alignment, INITIAL_VERTICAL_TEXT_ALIGNMENT);

        let padding = match self.padding {
            Property::Initial | Property::Inherit => Arc::clone(&*INITIAL_PADDING),
            Property::Value(value) => Arc::new(value),
        };

        let font = if matches!(self.font_family, Property::Initial | Property::Inherit)
            && matches!(self.font_size, Property::Initial | Property::Inherit)
            && matches!(self.font_style, Property::Initial | Property::Inherit)
            && matches!(self.font_weight, Property::Initial | Property::Inherit)
            && matches!(self.font_width, Property::Initial | Property::Inherit)
        {
            Arc::clone(&*INITIAL_FONT)
        } else {
            Arc::new(compute_root_font(self))
        };

        ComputedStyle {
            packed_fields: ComputedStylePackedFields::new()
                .with_width(ComputedSizing::Fixed)
                .with_height(ComputedSizing::Fixed)
                .with_layout_direction(layout_direction)
                .with_child_alignment(child_alignment)
                .with_cross_axis_alignment(cross_axis_alignment)
                .with_text_underline(text_underline)
                .with_text_strikethrough(text_strikethrough)
                .with_text_wrap(text_wrap)
                .with_horizontal_text_alignment(horizontal_text_alignment)
                .with_vertical_text_alignment(vertical_text_alignment),

            min_width: screen_size.x,
            min_height: screen_size.y,
            max_width: screen_size.x,
            max_height: screen_size.y,
            flex_ratio,
            padding,
            child_spacing,
            background,
            corner_radius,
            border_width,
            border_color,
            font,
            text_color,
        }
    }

    pub fn compute(&self, parent_style: &ComputedStyle) -> ComputedStyle {
        macro_rules! compute_property {
            ($property:ident, $initial:expr) => {
                match self.$property {
                    Property::Initial => const { $initial },
                    Property::Inherit => parent_style.$property,
                    Property::Value(value) => value,
                }
            };
        }

        macro_rules! compute_packed_property {
            ($property:ident, $initial:expr) => {
                match self.$property {
                    Property::Initial => const { $initial },
                    Property::Inherit => parent_style.packed_fields.$property(),
                    Property::Value(value) => value,
                }
            };
        }

        macro_rules! match_size {
            ($size:expr, $min_size:expr, $max_size:expr) => {
                match $size {
                    Sizing::FitContent => ComputedSizing::FitContent,
                    Sizing::Grow => ComputedSizing::Grow,
                    Sizing::Fixed(fixed_size) => {
                        $min_size = fixed_size.clamp($min_size, $max_size);
                        $max_size = $min_size;
                        ComputedSizing::Fixed
                    }
                }
            };
        }

        let mut min_width = compute_property!(min_width, INITIAL_MIN_SIZE);
        let mut min_height = compute_property!(min_height, INITIAL_MIN_SIZE);
        let mut max_width = compute_property!(max_width, INITIAL_MAX_SIZE);
        let mut max_height = compute_property!(max_height, INITIAL_MAX_SIZE);

        let computed_width = match self.width {
            Property::Initial => INITIAL_SIZE,
            Property::Inherit => match_size!(parent_style.width(), min_width, max_width),
            Property::Value(width) => match_size!(width, min_width, max_width),
        };

        let computed_height = match self.height {
            Property::Initial => INITIAL_SIZE,
            Property::Inherit => match_size!(parent_style.height(), min_height, max_height),
            Property::Value(height) => match_size!(height, min_height, max_height),
        };

        let flex_ratio = compute_property!(flex_ratio, INITIAL_FLEX_RATIO);
        let child_spacing = compute_property!(child_spacing, INITIAL_CHILD_SPACING);
        let background = compute_property!(background, INITIAL_BACKGROUND);
        let corner_radius = compute_property!(corner_radius, INITIAL_CORNER_RADIUS);
        let border_width = compute_property!(border_width, INITIAL_BORDER_WIDTH);
        let border_color = compute_property!(border_color, INITIAL_BORDER_COLOR);
        let text_color = compute_property!(text_color, INITIAL_TEXT_COLOR);

        let layout_direction = compute_packed_property!(layout_direction, INITIAL_LAYOUT_DIRECTION);
        let child_alignment = compute_packed_property!(child_alignment, INITIAL_ALIGNMENT);
        let cross_axis_alignment =
            compute_packed_property!(cross_axis_alignment, INITIAL_ALIGNMENT);
        let text_underline = compute_packed_property!(text_underline, INITIAL_TEXT_UNDERLINE);
        let text_strikethrough =
            compute_packed_property!(text_strikethrough, INITIAL_TEXT_STRIKETHROUGH);
        let text_wrap = compute_packed_property!(text_wrap, INITIAL_TEXT_WRAP);
        let horizontal_text_alignment =
            compute_packed_property!(horizontal_text_alignment, INITIAL_HORIZONTAL_TEXT_ALIGNMENT);
        let vertical_text_alignment =
            compute_packed_property!(vertical_text_alignment, INITIAL_VERTICAL_TEXT_ALIGNMENT);

        let padding = match self.padding {
            Property::Initial => Arc::clone(&*INITIAL_PADDING),
            Property::Inherit => Arc::clone(&parent_style.padding),
            Property::Value(value) => Arc::new(value),
        };

        let font = if matches!(self.font_family, Property::Initial)
            && matches!(self.font_size, Property::Initial)
            && matches!(self.font_style, Property::Initial)
            && matches!(self.font_weight, Property::Initial)
            && matches!(self.font_width, Property::Initial)
        {
            Arc::clone(&*INITIAL_FONT)
        } else if matches!(self.font_family, Property::Inherit)
            && matches!(self.font_size, Property::Inherit)
            && matches!(self.font_style, Property::Inherit)
            && matches!(self.font_weight, Property::Inherit)
            && matches!(self.font_width, Property::Inherit)
        {
            Arc::clone(&parent_style.font)
        } else {
            Arc::new(compute_font(self, parent_style))
        };

        ComputedStyle {
            packed_fields: ComputedStylePackedFields::new()
                .with_width(computed_width)
                .with_height(computed_height)
                .with_layout_direction(layout_direction)
                .with_child_alignment(child_alignment)
                .with_cross_axis_alignment(cross_axis_alignment)
                .with_text_underline(text_underline)
                .with_text_strikethrough(text_strikethrough)
                .with_text_wrap(text_wrap)
                .with_horizontal_text_alignment(horizontal_text_alignment)
                .with_vertical_text_alignment(vertical_text_alignment),

            min_width,
            min_height,
            max_width,
            max_height,
            flex_ratio,
            padding,
            child_spacing,
            background,
            corner_radius,
            border_width,
            border_color,
            font,
            text_color,
        }
    }
}

impl ComputedStyle {
    #[inline]
    pub fn width(&self) -> Sizing {
        match self.packed_fields.width() {
            ComputedSizing::FitContent => Sizing::FitContent,
            ComputedSizing::Grow => Sizing::Grow,
            ComputedSizing::Fixed => Sizing::Fixed(self.min_width),
        }
    }

    #[inline]
    pub fn height(&self) -> Sizing {
        match self.packed_fields.height() {
            ComputedSizing::FitContent => Sizing::FitContent,
            ComputedSizing::Grow => Sizing::Grow,
            ComputedSizing::Fixed => Sizing::Fixed(self.min_height),
        }
    }

    #[inline]
    pub fn min_width(&self) -> Pixel {
        self.min_width
    }

    #[inline]
    pub fn min_height(&self) -> Pixel {
        self.min_height
    }

    #[inline]
    pub fn max_width(&self) -> Pixel {
        self.max_width
    }

    #[inline]
    pub fn max_height(&self) -> Pixel {
        self.max_height
    }

    #[inline]
    pub fn flex_ratio(&self) -> f32 {
        self.flex_ratio
    }

    #[inline]
    pub fn padding(&self) -> Padding {
        *self.padding
    }

    #[inline]
    pub fn child_spacing(&self) -> Pixel {
        self.child_spacing
    }

    #[inline]
    pub fn layout_direction(&self) -> Direction {
        self.packed_fields.layout_direction()
    }

    #[inline]
    pub fn child_alignment(&self) -> Alignment {
        self.packed_fields.child_alignment()
    }

    #[inline]
    pub fn cross_axis_alignment(&self) -> Alignment {
        self.packed_fields.cross_axis_alignment()
    }

    #[inline]
    pub fn background(&self) -> Color {
        self.background
    }

    #[inline]
    pub fn corner_radius(&self) -> Pixel {
        self.corner_radius
    }

    #[inline]
    pub fn border_width(&self) -> Pixel {
        self.border_width
    }

    #[inline]
    pub fn border_color(&self) -> Color {
        self.border_color
    }

    #[inline]
    pub fn font_family(&self) -> &FontStack<'static> {
        &self.font.family
    }

    #[inline]
    pub fn font_size(&self) -> Pixel {
        self.font.size
    }

    #[inline]
    pub fn font_style(&self) -> FontStyle {
        self.font.style
    }

    #[inline]
    pub fn font_weight(&self) -> FontWeight {
        self.font.weight
    }

    #[inline]
    pub fn font_width(&self) -> FontWidth {
        self.font.width
    }

    #[inline]
    pub fn text_underline(&self) -> bool {
        self.packed_fields.text_underline()
    }

    #[inline]
    pub fn text_strikethrough(&self) -> bool {
        self.packed_fields.text_strikethrough()
    }

    #[inline]
    pub fn text_wrap(&self) -> bool {
        self.packed_fields.text_wrap()
    }

    #[inline]
    pub fn text_color(&self) -> Color {
        self.text_color
    }

    #[inline]
    pub fn horizontal_text_alignment(&self) -> HorizontalTextAlignment {
        self.packed_fields.horizontal_text_alignment()
    }

    #[inline]
    pub fn vertical_text_alignment(&self) -> VerticalTextAlignment {
        self.packed_fields.vertical_text_alignment()
    }

    pub fn as_style(&self) -> Style {
        Style {
            width: Property::Value(self.width()),
            height: Property::Value(self.height()),
            min_width: Property::Value(self.min_width()),
            min_height: Property::Value(self.min_height()),
            max_width: Property::Value(self.max_width()),
            max_height: Property::Value(self.max_height()),
            flex_ratio: Property::Value(self.flex_ratio()),
            padding: Property::Value(self.padding()),
            child_spacing: Property::Value(self.child_spacing()),
            layout_direction: Property::Value(self.layout_direction()),
            child_alignment: Property::Value(self.child_alignment()),
            cross_axis_alignment: Property::Value(self.cross_axis_alignment()),
            background: Property::Value(self.background()),
            corner_radius: Property::Value(self.corner_radius()),
            border_width: Property::Value(self.border_width()),
            border_color: Property::Value(self.border_color()),
            font_family: Property::Value(self.font_family().clone()),
            font_size: Property::Value(self.font_size()),
            font_style: Property::Value(self.font_style()),
            font_weight: Property::Value(self.font_weight()),
            font_width: Property::Value(self.font_width()),
            text_underline: Property::Value(self.text_underline()),
            text_strikethrough: Property::Value(self.text_strikethrough()),
            text_wrap: Property::Value(self.text_wrap()),
            text_color: Property::Value(self.text_color()),
            horizontal_text_alignment: Property::Value(self.horizontal_text_alignment()),
            vertical_text_alignment: Property::Value(self.vertical_text_alignment()),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __style_recursive {
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %initial, $($t:tt)*) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Initial); $($t)*)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %inherit, $($t:tt)*) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Inherit); $($t)*)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: $value:expr, $($t:tt)*) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Value($value.into())); $($t)*)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %initial) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Initial);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %inherit) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Inherit);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: $value:expr) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Value($value.into()));)
    };
    ($(($name:ident, $property:expr)),*;) => {
        $crate::style::Style {
            $($name: $property,)*
            ..$crate::style::Style::DEFAULT
        }
    };
}

#[macro_export]
macro_rules! style {
    ($($t:tt)*) => {
        $crate::__style_recursive!(; $($t)*)
    };
}
