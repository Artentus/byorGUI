use super::*;

use byor_gui_procmacro::StyleBuilder;
use modular_bitfield::prelude::*;
pub use parley::{FontFamily, FontStack, FontStyle, FontWeight, FontWidth, GenericFamily};
use std::fmt;
use std::sync::{Arc, LazyLock};

#[derive(Clone, Copy, PartialEq)]
pub enum Measurement {
    Pixel(Float<Pixel>),
    Point(Float<Point>),
    EM(Float<EM>),
}

impl fmt::Debug for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pixel(value) => fmt::Debug::fmt(value, f),
            Self::Point(value) => fmt::Debug::fmt(value, f),
            Self::EM(value) => fmt::Debug::fmt(value, f),
        }
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pixel(value) => fmt::Display::fmt(value, f),
            Self::Point(value) => fmt::Display::fmt(value, f),
            Self::EM(value) => fmt::Display::fmt(value, f),
        }
    }
}

macro_rules! impl_measurement_from_float {
    ($($unit:ident),* $(,)?) => {
        $(
            impl From<Float<$unit>> for Measurement {
                #[inline]
                fn from(value: Float<$unit>) -> Self {
                    Self::$unit(value)
                }
            }
        )*
    };
}

impl_measurement_from_float!(Pixel, Point, EM);

impl Measurement {
    #[inline]
    pub fn to_pixel(self, pixel_per_point: f32, pixel_per_em: f32) -> Float<Pixel> {
        match self {
            Measurement::Pixel(value) => value.to_pixel(pixel_per_point, pixel_per_em),
            Measurement::Point(value) => value.to_pixel(pixel_per_point, pixel_per_em),
            Measurement::EM(value) => value.to_pixel(pixel_per_point, pixel_per_em),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Sizing {
    #[default]
    FitContent,
    Grow,
    Fixed(Measurement),
}

impl<T: Into<Measurement>> From<T> for Sizing {
    #[inline]
    fn from(value: T) -> Self {
        Self::Fixed(value.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Padding {
    pub left: Measurement,
    pub right: Measurement,
    pub top: Measurement,
    pub bottom: Measurement,
}

impl Padding {
    pub const ZERO: Self = Self {
        left: Measurement::Pixel(Float::px(0.0)),
        right: Measurement::Pixel(Float::px(0.0)),
        top: Measurement::Pixel(Float::px(0.0)),
        bottom: Measurement::Pixel(Float::px(0.0)),
    };
}

impl Default for Padding {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<T: Into<Measurement>> From<T> for Padding {
    #[inline]
    fn from(value: T) -> Self {
        let value = value.into();

        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

impl<T: Into<Measurement>> From<[T; 2]> for Padding {
    #[inline]
    fn from(value: [T; 2]) -> Self {
        let value = value.map(Into::into);

        Self {
            left: value[0],
            right: value[0],
            top: value[1],
            bottom: value[1],
        }
    }
}

impl<T: Into<Measurement>> From<[T; 4]> for Padding {
    #[inline]
    fn from(value: [T; 4]) -> Self {
        let value = value.map(Into::into);

        Self {
            left: value[0],
            right: value[1],
            top: value[2],
            bottom: value[3],
        }
    }
}

impl<T1, T2> From<(T1, T2)> for Padding
where
    T1: Into<Measurement>,
    T2: Into<Measurement>,
{
    #[inline]
    fn from(value: (T1, T2)) -> Self {
        let value0 = value.0.into();
        let value1 = value.1.into();

        Self {
            left: value0,
            right: value0,
            top: value1,
            bottom: value1,
        }
    }
}

impl<T1, T2, T3, T4> From<(T1, T2, T3, T4)> for Padding
where
    T1: Into<Measurement>,
    T2: Into<Measurement>,
    T3: Into<Measurement>,
    T4: Into<Measurement>,
{
    #[inline]
    fn from(value: (T1, T2, T3, T4)) -> Self {
        let value0 = value.0.into();
        let value1 = value.1.into();
        let value2 = value.2.into();
        let value3 = value.3.into();

        Self {
            left: value0,
            right: value1,
            top: value2,
            bottom: value3,
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
    ($([$default_value:ident] $property_name:ident: $property_type:ty { $initial_value:expr },)*) => {
        #[derive(Debug, Clone, StyleBuilder)]
        pub struct Style {
            $(pub $property_name: Property<$property_type>,)*
        }

        #[derive(Debug, Clone)]
        pub struct CascadedStyle {
            $(pub $property_name: $property_type,)*
        }

        impl Style {
            pub const DEFAULT: Self = Self {
                $($property_name: Property::$default_value,)*
            };
        }

        impl Default for Style {
            #[inline]
            fn default() -> Self {
                Self::DEFAULT
            }
        }

        impl Style {
            pub fn cascade_root(&self, screen_size: Vec2<Pixel>) -> CascadedStyle {
                let mut style = CascadedStyle {
                    $(
                        $property_name: match &self.$property_name {
                            Property::Initial | Property::Inherit => $initial_value,
                            Property::Value(value) => value.clone(),
                        },
                    )*
                };

                style.width = screen_size.x.into();
                style.height = screen_size.y.into();
                style.min_width = screen_size.x.into();
                style.min_height = screen_size.y.into();
                style.max_width = screen_size.x.into();
                style.max_height = screen_size.y.into();

                style
            }

            pub fn cascade(&self, parent_style: &CascadedStyle) -> CascadedStyle {
                CascadedStyle {
                    $(
                        $property_name: match &self.$property_name {
                            Property::Initial => $initial_value,
                            Property::Inherit => parent_style.$property_name.clone(),
                            Property::Value(value) => value.clone(),
                        },
                    )*
                }
            }
        }

        impl CascadedStyle {
            pub fn as_style(&self) -> Style {
                Style {
                    $(
                        $property_name: Property::Value(self.$property_name.clone()),
                    )*
                }
            }
        }
    };
}

const ROOT_FONT_SIZE: Float<Pixel> = Float::px(16.0);

const INITIAL_SIZE: Sizing = Sizing::FitContent;
const INITIAL_MIN_SIZE: Measurement = Measurement::Pixel(Float::px(0.0));
const INITIAL_MAX_SIZE: Measurement = Measurement::Pixel(Float::px(f32::MAX));
const INITIAL_FLEX_RATIO: f32 = 1.0;
const INITIAL_PADDING: Padding = Padding::ZERO;
const INITIAL_CHILD_SPACING: Measurement = Measurement::Pixel(Float::px(0.0));
const INITIAL_LAYOUT_DIRECTION: Direction = Direction::LeftToRight;
const INITIAL_ALIGNMENT: Alignment = Alignment::Start;
const INITIAL_BACKGROUND: Color = Color::TRANSPARENT;
const INITIAL_CORNER_RADIUS: Measurement = Measurement::Pixel(Float::px(0.0));
const INITIAL_BORDER_WIDTH: Measurement = Measurement::Pixel(Float::px(0.0));
const INITIAL_BORDER_COLOR: Color = Color::TRANSPARENT;
const INITIAL_FONT_FAMILY: FontStack<'static> =
    FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi));
const INITIAL_FONT_SIZE: Measurement = Measurement::Pixel(ROOT_FONT_SIZE);
const INITIAL_FONT_STYLE: FontStyle = FontStyle::Normal;
const INITIAL_FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
const INITIAL_FONT_WIDTH: FontWidth = FontWidth::NORMAL;
const INITIAL_TEXT_UNDERLINE: bool = false;
const INITIAL_TEXT_STRIKETHROUGH: bool = false;
const INITIAL_TEXT_WRAP: bool = true;
const INITIAL_TEXT_COLOR: Color = Color::BLACK;
const INITIAL_HORIZONTAL_TEXT_ALIGNMENT: HorizontalTextAlignment = HorizontalTextAlignment::Start;
const INITIAL_VERTICAL_TEXT_ALIGNMENT: VerticalTextAlignment = VerticalTextAlignment::Top;

define_style! {
    [Initial] width: Sizing { INITIAL_SIZE },
    [Initial] height: Sizing { INITIAL_SIZE },
    [Initial] min_width: Measurement { INITIAL_MIN_SIZE },
    [Initial] min_height: Measurement { INITIAL_MIN_SIZE },
    [Initial] max_width: Measurement { INITIAL_MAX_SIZE },
    [Initial] max_height: Measurement { INITIAL_MAX_SIZE },
    [Initial] flex_ratio: f32 { INITIAL_FLEX_RATIO },
    [Initial] padding: Padding { INITIAL_PADDING },
    [Initial] child_spacing: Measurement { INITIAL_CHILD_SPACING },
    [Initial] layout_direction: Direction { INITIAL_LAYOUT_DIRECTION },
    [Initial] child_alignment: Alignment { INITIAL_ALIGNMENT },
    [Initial] cross_axis_alignment: Alignment { INITIAL_ALIGNMENT },
    [Initial] background: Color { INITIAL_BACKGROUND },
    [Initial] corner_radius: Measurement { INITIAL_CORNER_RADIUS },
    [Initial] border_width: Measurement { INITIAL_BORDER_WIDTH },
    [Initial] border_color: Color { INITIAL_BORDER_COLOR },
    [Inherit] font_family: FontStack<'static> { INITIAL_FONT_FAMILY },
    [Inherit] font_size: Measurement { INITIAL_FONT_SIZE },
    [Inherit] font_style: FontStyle { INITIAL_FONT_STYLE },
    [Inherit] font_weight: FontWeight { INITIAL_FONT_WEIGHT },
    [Inherit] font_width: FontWidth { INITIAL_FONT_WIDTH },
    [Inherit] text_underline: bool { INITIAL_TEXT_UNDERLINE },
    [Inherit] text_strikethrough: bool { INITIAL_TEXT_STRIKETHROUGH },
    [Inherit] text_wrap: bool { INITIAL_TEXT_WRAP },
    [Inherit] text_color: Color { INITIAL_TEXT_COLOR },
    [Inherit] horizontal_text_alignment: HorizontalTextAlignment { INITIAL_HORIZONTAL_TEXT_ALIGNMENT },
    [Inherit] vertical_text_alignment: VerticalTextAlignment { INITIAL_VERTICAL_TEXT_ALIGNMENT },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Specifier)]
#[bits = 2]
pub(crate) enum ComputedSizing {
    FitContent,
    Grow,
    Fixed,
}

impl Sizing {
    #[inline]
    fn compute(self, pixel_per_point: f32, pixel_per_em: f32) -> (ComputedSizing, Float<Pixel>) {
        match self {
            Self::FitContent => (ComputedSizing::FitContent, 0.px()),
            Self::Grow => (ComputedSizing::Grow, 0.px()),
            Self::Fixed(fixed_size) => (
                ComputedSizing::Fixed,
                fixed_size.to_pixel(pixel_per_point, pixel_per_em).round(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedPadding {
    pub left: Float<Pixel>,
    pub right: Float<Pixel>,
    pub top: Float<Pixel>,
    pub bottom: Float<Pixel>,
}

impl ComputedPadding {
    pub const ZERO: Self = Self {
        left: Float::px(0.0),
        right: Float::px(0.0),
        top: Float::px(0.0),
        bottom: Float::px(0.0),
    };
}

impl Default for ComputedPadding {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl Padding {
    #[inline]
    fn compute(&self, pixel_per_point: f32, pixel_per_em: f32) -> ComputedPadding {
        ComputedPadding {
            left: self.left.to_pixel(pixel_per_point, pixel_per_em).round(),
            right: self.right.to_pixel(pixel_per_point, pixel_per_em).round(),
            top: self.top.to_pixel(pixel_per_point, pixel_per_em).round(),
            bottom: self.bottom.to_pixel(pixel_per_point, pixel_per_em).round(),
        }
    }
}

struct ComputedFont {
    family: FontStack<'static>,
    size: Float<Pixel>,
    style: FontStyle,
    weight: FontWeight,
    width: FontWidth,
}

impl ComputedFont {
    const INITIAL: Self = Self {
        family: INITIAL_FONT_FAMILY,
        size: ROOT_FONT_SIZE,
        style: INITIAL_FONT_STYLE,
        weight: INITIAL_FONT_WEIGHT,
        width: INITIAL_FONT_WIDTH,
    };
}

impl Default for ComputedFont {
    #[inline]
    fn default() -> Self {
        Self::INITIAL
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

    flex_ratio: f32,
    padding: Arc<ComputedPadding>,
    child_spacing: Float<Pixel>,
    background: Color,
    corner_radius: Float<Pixel>,
    border_width: Float<Pixel>,
    border_color: Color,
    font: Arc<ComputedFont>,
    text_color: Color,

    pub(crate) fixed_size: Vec2<Pixel>,
    pub(crate) min_size: Vec2<Pixel>,
    pub(crate) max_size: Vec2<Pixel>,
}

impl ComputedStyle {
    // values that differ from the cascaded style
    // ------------------------------------------------------

    #[inline]
    pub fn padding(&self) -> &ComputedPadding {
        &*self.padding
    }

    #[inline]
    pub fn child_spacing(&self) -> Float<Pixel> {
        self.child_spacing
    }

    #[inline]
    pub fn corner_radius(&self) -> Float<Pixel> {
        self.corner_radius
    }

    #[inline]
    pub fn border_width(&self) -> Float<Pixel> {
        self.border_width
    }

    #[inline]
    pub fn font_size(&self) -> Float<Pixel> {
        self.font.size
    }

    // values that don't
    // ------------------------------------------------------

    #[inline]
    pub(crate) fn width(&self) -> ComputedSizing {
        self.packed_fields.width()
    }

    #[inline]
    pub(crate) fn height(&self) -> ComputedSizing {
        self.packed_fields.height()
    }

    #[inline]
    pub(crate) fn layout_direction(&self) -> Direction {
        self.packed_fields.layout_direction()
    }

    #[inline]
    pub(crate) fn child_alignment(&self) -> Alignment {
        self.packed_fields.child_alignment()
    }

    #[inline]
    pub(crate) fn cross_axis_alignment(&self) -> Alignment {
        self.packed_fields.cross_axis_alignment()
    }

    #[inline]
    pub(crate) fn text_underline(&self) -> bool {
        self.packed_fields.text_underline()
    }

    #[inline]
    pub(crate) fn text_strikethrough(&self) -> bool {
        self.packed_fields.text_strikethrough()
    }

    #[inline]
    pub(crate) fn text_wrap(&self) -> bool {
        self.packed_fields.text_wrap()
    }

    #[inline]
    pub(crate) fn horizontal_text_alignment(&self) -> HorizontalTextAlignment {
        self.packed_fields.horizontal_text_alignment()
    }

    #[inline]
    pub(crate) fn vertical_text_alignment(&self) -> VerticalTextAlignment {
        self.packed_fields.vertical_text_alignment()
    }

    #[inline]
    pub(crate) fn flex_ratio(&self) -> f32 {
        self.flex_ratio
    }

    #[inline]
    pub(crate) fn background(&self) -> Color {
        self.background
    }

    #[inline]
    pub(crate) fn border_color(&self) -> Color {
        self.border_color
    }

    #[inline]
    pub(crate) fn font_family(&self) -> &FontStack<'static> {
        &self.font.family
    }

    #[inline]
    pub(crate) fn font_style(&self) -> FontStyle {
        self.font.style
    }

    #[inline]
    pub(crate) fn font_weight(&self) -> FontWeight {
        self.font.weight
    }

    #[inline]
    pub(crate) fn font_width(&self) -> FontWidth {
        self.font.width
    }

    #[inline]
    pub(crate) fn text_color(&self) -> Color {
        self.text_color
    }
}

macro_rules! all_match {
    ([$($property:expr),* $(,)?], $pattern:pat) => {
        true $(&& matches!($property, $pattern))*
    };
}

static INITIAL_COMPUTED_PADDING: LazyLock<Arc<ComputedPadding>> =
    LazyLock::new(|| Arc::new(ComputedPadding::default()));
static INITIAL_COMPUTED_FONT: LazyLock<Arc<ComputedFont>> =
    LazyLock::new(|| Arc::new(ComputedFont::default()));

pub(crate) fn compute_style(
    style: &Style,
    cascaded_style: &CascadedStyle,
    parent_style: Option<&ComputedStyle>,
    scale_factor: f32,
) -> ComputedStyle {
    let parent_font_size = parent_style
        .map(ComputedStyle::font_size)
        .unwrap_or(ROOT_FONT_SIZE);
    let font_size = cascaded_style
        .font_size
        .to_pixel(scale_factor, parent_font_size.value());

    let (width, fixed_width) = cascaded_style
        .width
        .compute(scale_factor, font_size.value());
    let (height, fixed_height) = cascaded_style
        .height
        .compute(scale_factor, font_size.value());

    let min_width = cascaded_style
        .min_width
        .to_pixel(scale_factor, font_size.value())
        .round();
    let min_height = cascaded_style
        .min_height
        .to_pixel(scale_factor, font_size.value())
        .round();
    let max_width = cascaded_style
        .max_width
        .to_pixel(scale_factor, font_size.value())
        .round();
    let max_height = cascaded_style
        .max_height
        .to_pixel(scale_factor, font_size.value())
        .round();
    let child_spacing = cascaded_style
        .child_spacing
        .to_pixel(scale_factor, font_size.value())
        .round();
    let corner_radius = cascaded_style
        .corner_radius
        .to_pixel(scale_factor, font_size.value());
    let border_width = cascaded_style
        .border_width
        .to_pixel(scale_factor, font_size.value());

    let min_size = Vec2 {
        x: min_width,
        y: min_height,
    };
    let max_size = Vec2 {
        x: max_width,
        y: max_height,
    };
    let fixed_size = Vec2 {
        x: fixed_width,
        y: fixed_height,
    }
    .clamp(min_size, max_size);

    let padding = match &style.padding {
        Property::Initial => Arc::clone(&*INITIAL_COMPUTED_PADDING),
        Property::Inherit => {
            if let Some(parent_style) = parent_style {
                Arc::clone(&parent_style.padding)
            } else {
                Arc::clone(&*INITIAL_COMPUTED_PADDING)
            }
        }
        Property::Value(_) => Arc::new(
            cascaded_style
                .padding
                .compute(scale_factor, font_size.value()),
        ),
    };

    let font = if all_match!(
        [
            style.font_family,
            style.font_size,
            style.font_style,
            style.font_weight,
            style.font_width,
        ],
        Property::Initial
    ) {
        Arc::clone(&*INITIAL_COMPUTED_FONT)
    } else if all_match!(
        [
            style.font_family,
            style.font_size,
            style.font_style,
            style.font_weight,
            style.font_width,
        ],
        Property::Inherit
    ) {
        if let Some(parent_style) = parent_style {
            Arc::clone(&parent_style.font)
        } else {
            Arc::clone(&*INITIAL_COMPUTED_FONT)
        }
    } else {
        Arc::new(ComputedFont {
            family: cascaded_style.font_family.clone(),
            size: font_size,
            style: cascaded_style.font_style,
            weight: cascaded_style.font_weight,
            width: cascaded_style.font_width,
        })
    };

    ComputedStyle {
        packed_fields: ComputedStylePackedFields::new()
            .with_width(width)
            .with_height(height)
            .with_layout_direction(cascaded_style.layout_direction)
            .with_child_alignment(cascaded_style.child_alignment)
            .with_cross_axis_alignment(cascaded_style.cross_axis_alignment)
            .with_text_underline(cascaded_style.text_underline)
            .with_text_strikethrough(cascaded_style.text_strikethrough)
            .with_text_wrap(cascaded_style.text_wrap)
            .with_horizontal_text_alignment(cascaded_style.horizontal_text_alignment)
            .with_vertical_text_alignment(cascaded_style.vertical_text_alignment),

        flex_ratio: cascaded_style.flex_ratio,
        padding,
        child_spacing,
        background: cascaded_style.background,
        corner_radius,
        border_width,
        border_color: cascaded_style.border_color,
        font,
        text_color: cascaded_style.text_color,

        fixed_size,
        min_size,
        max_size,
    }
}
