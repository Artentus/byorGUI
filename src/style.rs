pub mod axis;
pub mod computed;

use super::*;
use byor_gui_procmacro::StyleBuilder;
use modular_bitfield::prelude::*;
use std::fmt;
use std::ops::{Div, DivAssign, Mul, MulAssign};
use std::sync::{Arc, LazyLock};

pub use parley::{FontFamily, FontStack, FontStyle, FontWeight, FontWidth, GenericFamily};
pub use smallvec::{SmallVec, smallvec};

macro_rules! def_measurement {
    ($name:ident[$($unit:ident),+ $(,)?]) => {
        #[derive(Clone, Copy, PartialEq)]
        pub enum $name {
            $($unit(Float<$unit>),)+
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$unit(value) => fmt::Debug::fmt(value, f),)+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$unit(value) => fmt::Display::fmt(value, f),)+
                }
            }
        }

        $(
            impl From<Float<$unit>> for $name {
                #[inline]
                fn from(value: Float<$unit>) -> Self {
                    Self::$unit(value)
                }
            }
        )*

        impl Mul<f32> for $name {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: f32) -> Self::Output {
                match self {
                    $(Self::$unit(value) => Self::$unit(value * rhs),)+
                }
            }
        }

        impl MulAssign<f32> for $name {
            #[inline]
            fn mul_assign(&mut self, rhs: f32) {
                *self = *self * rhs;
            }
        }

        impl Div<f32> for $name {
            type Output = Self;

            #[inline]
            fn div(self, rhs: f32) -> Self::Output {
                match self {
                    $(Self::$unit(value) => Self::$unit(value / rhs),)+
                }
            }
        }

        impl DivAssign<f32> for $name {
            #[inline]
            fn div_assign(&mut self, rhs: f32) {
                *self = *self / rhs;
            }
        }
    };
}

def_measurement! {
    AbsoluteMeasurement[Pixel, Point, EM]
}

impl AbsoluteMeasurement {
    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_point: f32, pixel_per_em: f32) -> Float<Pixel> {
        match self {
            Self::Pixel(value) => value,
            Self::Point(value) => value.to_pixel(pixel_per_point),
            Self::EM(value) => value.to_pixel(pixel_per_em),
        }
    }
}

def_measurement! {
    RelativeMeasurement[Pixel, Point, EM, Percent]
}

impl RelativeMeasurement {
    #[must_use]
    #[inline]
    pub fn to_pixel(
        self,
        pixel_per_point: f32,
        pixel_per_em: f32,
        one_hundred_percent_value: Float<Pixel>,
    ) -> Float<Pixel> {
        match self {
            Self::Pixel(value) => value,
            Self::Point(value) => value.to_pixel(pixel_per_point),
            Self::EM(value) => value.to_pixel(pixel_per_em),
            Self::Percent(value) => value.to_pixel(one_hundred_percent_value),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Sizing {
    #[default]
    FitContent,
    Grow,
    Fixed(AbsoluteMeasurement),
}

impl<T: Into<AbsoluteMeasurement>> From<T> for Sizing {
    #[inline]
    fn from(value: T) -> Self {
        Self::Fixed(value.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Padding {
    pub left: AbsoluteMeasurement,
    pub right: AbsoluteMeasurement,
    pub top: AbsoluteMeasurement,
    pub bottom: AbsoluteMeasurement,
}

impl Padding {
    pub const ZERO: Self = Self {
        left: AbsoluteMeasurement::Pixel(Float::px(0.0)),
        right: AbsoluteMeasurement::Pixel(Float::px(0.0)),
        top: AbsoluteMeasurement::Pixel(Float::px(0.0)),
        bottom: AbsoluteMeasurement::Pixel(Float::px(0.0)),
    };
}

impl Default for Padding {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<T: Into<AbsoluteMeasurement>> From<T> for Padding {
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

impl<T: Into<AbsoluteMeasurement>> From<[T; 2]> for Padding {
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

impl<T: Into<AbsoluteMeasurement>> From<[T; 4]> for Padding {
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
    T1: Into<AbsoluteMeasurement>,
    T2: Into<AbsoluteMeasurement>,
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
    T1: Into<AbsoluteMeasurement>,
    T2: Into<AbsoluteMeasurement>,
    T3: Into<AbsoluteMeasurement>,
    T4: Into<AbsoluteMeasurement>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Brush {
    Solid(Color),
    LinearGradient {
        start_x: RelativeMeasurement,
        start_y: RelativeMeasurement,
        end_x: RelativeMeasurement,
        end_y: RelativeMeasurement,
        stops: SmallVec<[GradientStop; 4]>,
    },
    RadialGradient {
        center_x: RelativeMeasurement,
        center_y: RelativeMeasurement,
        radius_x: RelativeMeasurement,
        radius_y: RelativeMeasurement,
        stops: SmallVec<[GradientStop; 4]>,
    },
}

impl From<Color> for Brush {
    #[inline]
    fn from(color: Color) -> Self {
        Self::Solid(color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupPosition {
    BeforeParent,
    ParentStart,
    ParentEnd,
    AfterParent,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum FloatPosition {
    #[default]
    Cursor,
    CursorFixed,
    Fixed {
        x: AbsoluteMeasurement,
        y: AbsoluteMeasurement,
    },
    Popup {
        x: PopupPosition,
        y: PopupPosition,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PersistentFloatPosition {
    Cursor {
        referenced: bool,
        x: Float<Pixel>,
        y: Float<Pixel>,
    },
    CursorFixed {
        referenced: bool,
        x: Float<Pixel>,
        y: Float<Pixel>,
    },
    Fixed {
        referenced: bool,
        x: Float<Pixel>,
        y: Float<Pixel>,
    },
    Popup {
        referenced: bool,
        x: PopupPosition,
        y: PopupPosition,
    },
}

impl PersistentFloatPosition {
    #[inline]
    pub(crate) const fn referenced(&self) -> bool {
        match self {
            &Self::Cursor { referenced, .. }
            | &Self::CursorFixed { referenced, .. }
            | &Self::Fixed { referenced, .. }
            | &Self::Popup { referenced, .. } => referenced,
        }
    }

    #[inline]
    pub(crate) const fn reset_referenced(&mut self) {
        match self {
            Self::Cursor { referenced, .. }
            | Self::CursorFixed { referenced, .. }
            | Self::Fixed { referenced, .. }
            | Self::Popup { referenced, .. } => *referenced = false,
        }
    }
}

impl Default for PersistentFloatPosition {
    #[inline]
    fn default() -> Self {
        Self::Fixed {
            referenced: true,
            x: 0.px(),
            y: 0.px(),
        }
    }
}

pub type PropertyFn<T> = fn(&CascadedStyle, NodeInputState) -> T;

#[derive(Debug, Default, Clone, Copy)]
pub enum Property<T, const INHERIT_FALLBACK: bool> {
    /// The property is not specified
    #[default]
    Unspecified,
    /// A property-specific default value (not necessarily the same as [`Default::default()`])
    Initial,
    /// The same value as its parent, or the initial value in case of the root
    Inherit,
    /// A specific value
    Value(T),
    /// Compute the value using a custom function
    Compute(PropertyFn<T>),
}

impl<T: Clone, const INHERIT_FALLBACK: bool> Property<T, INHERIT_FALLBACK> {
    #[must_use]
    #[inline]
    pub fn or_else(self, other: &Self) -> Self {
        if matches!(self, Self::Unspecified) {
            other.clone()
        } else {
            self
        }
    }

    #[must_use]
    #[inline]
    pub fn cascade(
        self,
        parent_value: &T,
        parent_style: &CascadedStyle,
        input_state: NodeInputState,
    ) -> Option<T> {
        match self {
            Self::Unspecified => match INHERIT_FALLBACK {
                false => None,
                true => Some(parent_value.clone()),
            },
            Self::Initial => None,
            Self::Inherit => Some(parent_value.clone()),
            Self::Value(value) => Some(value),
            Self::Compute(f) => Some(f(parent_style, input_state)),
        }
    }
}

impl<T, const INHERIT_FALLBACK: bool> From<T> for Property<T, INHERIT_FALLBACK> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T, const INHERIT_FALLBACK: bool> From<PropertyFn<T>> for Property<T, INHERIT_FALLBACK> {
    #[inline]
    fn from(f: PropertyFn<T>) -> Self {
        Self::Compute(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PropertyFallback {
    Initial,
    Inherit,
}

impl PropertyFallback {
    #[must_use]
    #[inline]
    const fn is_inherit(self) -> bool {
        match self {
            Self::Initial => false,
            Self::Inherit => true,
        }
    }
}

macro_rules! define_style {
    ($([$fallback_value:ident] $property_name:ident: $property_type:ty { $initial_value:expr },)*) => {
        #[derive(Debug, Clone, StyleBuilder)]
        pub struct Style {
            $(pub $property_name: Property<$property_type, { PropertyFallback::$fallback_value.is_inherit() }>,)*
        }

        #[derive(Debug, Clone)]
        pub struct CascadedStyle {
            $(pub $property_name: $property_type,)*
        }

        impl Style {
            pub const DEFAULT: Self = Self {
                $($property_name: Property::Unspecified,)*
            };
        }

        impl Default for Style {
            #[inline]
            fn default() -> Self {
                Self::DEFAULT
            }
        }

        impl CascadedStyle {
            pub const INITIAL: Self = Self {
                $($property_name: $initial_value,)*
            };
        }

        impl Style {
            #[must_use]
            pub fn or_else(&self, other: &Self) -> Self {
                Self {
                    $(
                        $property_name: self.$property_name.clone().or_else(&other.$property_name),
                    )*
                }
            }

            #[must_use]
            pub fn cascade_root(&self, screen_size: Vec2<Pixel>, input_state: NodeInputState) -> CascadedStyle {
                let mut style = CascadedStyle {
                    $(
                        $property_name: match &self.$property_name {
                            Property::Unspecified | Property::Initial | Property::Inherit => $initial_value,
                            Property::Value(value) => value.clone(),
                            Property::Compute(f) => f(&CascadedStyle::INITIAL, input_state),
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

            #[must_use]
            pub fn cascade(&self, parent_style: &CascadedStyle, input_state: NodeInputState) -> CascadedStyle {
                CascadedStyle {
                    $(
                        $property_name: self
                            .$property_name
                            .clone()
                            .cascade(&parent_style.$property_name, &parent_style, input_state)
                            .unwrap_or($initial_value),
                    )*
                }
            }
        }

        impl CascadedStyle {
            #[must_use]
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

pub const INITIAL_SIZE: Sizing = Sizing::FitContent;
pub const INITIAL_MIN_SIZE: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(Float::px(0.0));
pub const INITIAL_MAX_SIZE: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(Float::px(f32::MAX));
pub const INITIAL_FLEX_RATIO: f32 = 1.0;
pub const INITIAL_PADDING: Padding = Padding::ZERO;
pub const INITIAL_CHILD_SPACING: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(Float::px(0.0));
pub const INITIAL_LAYOUT_DIRECTION: Direction = Direction::LeftToRight;
pub const INITIAL_ALIGNMENT: Alignment = Alignment::Start;
pub const INITIAL_BACKGROUND: Brush = Brush::Solid(Color::TRANSPARENT);
pub const INITIAL_CORNER_RADIUS: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(Float::px(0.0));
pub const INITIAL_BORDER_WIDTH: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(Float::px(0.0));
pub const INITIAL_BORDER_COLOR: Color = Color::TRANSPARENT;
pub const INITIAL_DROP_SHADOW_WIDTH: AbsoluteMeasurement =
    AbsoluteMeasurement::Pixel(Float::px(0.0));
pub const INITIAL_DROP_SHADOW_COLOR: Color = Color::TRANSPARENT;
pub const INITIAL_FONT_FAMILY: FontStack<'static> =
    FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi));
pub const INITIAL_FONT_SIZE: AbsoluteMeasurement = AbsoluteMeasurement::Pixel(ROOT_FONT_SIZE);
pub const INITIAL_FONT_STYLE: FontStyle = FontStyle::Normal;
pub const INITIAL_FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
pub const INITIAL_FONT_WIDTH: FontWidth = FontWidth::NORMAL;
pub const INITIAL_TEXT_UNDERLINE: bool = false;
pub const INITIAL_TEXT_STRIKETHROUGH: bool = false;
pub const INITIAL_TEXT_WRAP: bool = true;
pub const INITIAL_TEXT_COLOR: Color = Color::BLACK;
pub const INITIAL_HORIZONTAL_TEXT_ALIGNMENT: HorizontalTextAlignment =
    HorizontalTextAlignment::Start;
pub const INITIAL_VERTICAL_TEXT_ALIGNMENT: VerticalTextAlignment = VerticalTextAlignment::Top;

define_style! {
    [Initial] width: Sizing { INITIAL_SIZE },
    [Initial] height: Sizing { INITIAL_SIZE },
    [Initial] min_width: AbsoluteMeasurement { INITIAL_MIN_SIZE },
    [Initial] min_height: AbsoluteMeasurement { INITIAL_MIN_SIZE },
    [Initial] max_width: AbsoluteMeasurement { INITIAL_MAX_SIZE },
    [Initial] max_height: AbsoluteMeasurement { INITIAL_MAX_SIZE },
    [Initial] flex_ratio: f32 { INITIAL_FLEX_RATIO },
    [Initial] padding: Padding { INITIAL_PADDING },
    [Initial] child_spacing: AbsoluteMeasurement { INITIAL_CHILD_SPACING },
    [Initial] layout_direction: Direction { INITIAL_LAYOUT_DIRECTION },
    [Initial] child_alignment: Alignment { INITIAL_ALIGNMENT },
    [Initial] cross_axis_alignment: Alignment { INITIAL_ALIGNMENT },
    [Initial] background: Brush { INITIAL_BACKGROUND },
    [Initial] corner_radius: AbsoluteMeasurement { INITIAL_CORNER_RADIUS },
    [Initial] border_width: AbsoluteMeasurement { INITIAL_BORDER_WIDTH },
    [Initial] border_color: Color { INITIAL_BORDER_COLOR },
    [Initial] drop_shadow_width: AbsoluteMeasurement { INITIAL_DROP_SHADOW_WIDTH },
    [Initial] drop_shadow_color: Color { INITIAL_DROP_SHADOW_COLOR },
    [Inherit] font_family: FontStack<'static> { INITIAL_FONT_FAMILY },
    [Inherit] font_size: AbsoluteMeasurement { INITIAL_FONT_SIZE },
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

/// This type is a hack to help the compiler perform double type conversions in the style macro.
#[doc(hidden)]
pub enum _PropertyValue<T, I: Into<T>> {
    Value(I),
    Compute(PropertyFn<T>),
}

impl<T, I: Into<T>> From<I> for _PropertyValue<T, I> {
    #[inline]
    fn from(value: I) -> Self {
        Self::Value(value)
    }
}

impl<T> From<PropertyFn<T>> for _PropertyValue<T, T> {
    #[inline]
    fn from(f: PropertyFn<T>) -> Self {
        Self::Compute(f)
    }
}

impl<T, const INHERIT_FALLBACK: bool> Property<T, INHERIT_FALLBACK> {
    #[doc(hidden)]
    #[inline]
    pub fn _from_value<I: Into<T>>(value: _PropertyValue<T, I>) -> Self {
        match value {
            _PropertyValue::Value(value) => Self::Value(value.into()),
            _PropertyValue::Compute(f) => Self::Compute(f),
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
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::_from_value($value.into())); $($t)*)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %initial) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Initial);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %inherit) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Inherit);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: $value:expr) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::_from_value($value.into()));)
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
