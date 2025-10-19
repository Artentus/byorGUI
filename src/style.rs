pub mod axis;
pub mod computed;

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
    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_point: f32, pixel_per_em: f32) -> Float<Pixel> {
        match self {
            Measurement::Pixel(value) => value,
            Measurement::Point(value) => value.to_pixel(pixel_per_point),
            Measurement::EM(value) => value.to_pixel(pixel_per_em),
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
    #[must_use]
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Initial | Self::Inherit => default,
            Self::Value(value) => value,
        }
    }

    #[must_use]
    #[inline]
    pub fn unwrap_or_else(self, f: impl FnOnce() -> T) -> T {
        match self {
            Self::Initial | Self::Inherit => f(),
            Self::Value(value) => value,
        }
    }

    #[must_use]
    #[inline]
    pub fn as_ref(&self) -> Property<&T> {
        match self {
            Self::Initial => Property::Initial,
            Self::Inherit => Property::Inherit,
            Self::Value(value) => Property::Value(value),
        }
    }

    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
            #[must_use]
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

            #[must_use]
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
