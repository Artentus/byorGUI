use super::*;

use byor_gui_procmacro::StyleBuilder;
pub use parley::Alignment as HorizontalTextAlignment;
pub use parley::style::{FontFamily, FontStack, FontStyle, FontWidth, GenericFamily};

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
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    SemiLight,
    #[default]
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
}

impl From<FontWeight> for parley::style::FontWeight {
    fn from(value: FontWeight) -> Self {
        match value {
            FontWeight::Thin => parley::style::FontWeight::THIN,
            FontWeight::ExtraLight => parley::style::FontWeight::EXTRA_LIGHT,
            FontWeight::Light => parley::style::FontWeight::LIGHT,
            FontWeight::SemiLight => parley::style::FontWeight::SEMI_LIGHT,
            FontWeight::Normal => parley::style::FontWeight::NORMAL,
            FontWeight::Medium => parley::style::FontWeight::MEDIUM,
            FontWeight::SemiBold => parley::style::FontWeight::SEMI_BOLD,
            FontWeight::Bold => parley::style::FontWeight::BOLD,
            FontWeight::ExtraBold => parley::style::FontWeight::EXTRA_BOLD,
            FontWeight::Black => parley::style::FontWeight::BLACK,
            FontWeight::ExtraBlack => parley::style::FontWeight::EXTRA_BLACK,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
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
    ($($name:ident: $t:ty,)*) => {
        #[derive(Debug, Clone, StyleBuilder)]
        pub struct Style {
            $(pub $name: Property<$t>,)*
        }

        #[derive(Debug, Clone)]
        pub struct ComputedStyle {
            $(pub $name: $t,)*
        }

        impl Style {
            pub fn compute_root(&self, screen_size: Size) -> ComputedStyle {
                let mut style = ComputedStyle {
                    $(
                        $name: match &self.$name {
                            Property::Initial | Property::Inherit => ComputedStyle::INITIAL.$name,
                            Property::Value(value) => value.clone(),
                        },
                    )*
                };

                style.width = Sizing::Fixed(screen_size.width);
                style.height = Sizing::Fixed(screen_size.height);
                style.min_width = screen_size.width;
                style.min_height = screen_size.height;
                style.max_width = screen_size.width;
                style.max_height = screen_size.height;

                style
            }

            pub fn compute(&self, parent_style: &ComputedStyle) -> ComputedStyle {
                ComputedStyle {
                    $(
                        $name: match &self.$name {
                            Property::Initial => ComputedStyle::INITIAL.$name,
                            Property::Inherit => parent_style.$name.clone(),
                            Property::Value(value) => value.clone(),
                        },
                    )*
                }
            }
        }

        impl ComputedStyle {
            pub fn into_style(self) -> Style {
                Style {
                    $($name: Property::Value(self.$name),)*
                }
            }
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
    font: FontStack<'static>,
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
        font: Property::Inherit,
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

impl ComputedStyle {
    pub const INITIAL: Self = Self {
        width: Sizing::FitContent,
        height: Sizing::FitContent,
        min_width: 0.0,
        min_height: 0.0,
        max_width: Pixel::MAX,
        max_height: Pixel::MAX,
        flex_ratio: 1.0,
        padding: Padding::ZERO,
        child_spacing: 0.0,
        layout_direction: Direction::LeftToRight,
        child_alignment: Alignment::Start,
        cross_axis_alignment: Alignment::Start,
        background: Color::TRANSPARENT,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
        font: FontStack::Single(FontFamily::Generic(GenericFamily::SystemUi)),
        font_size: 16.0,
        font_style: FontStyle::Normal,
        font_weight: FontWeight::Normal,
        font_width: FontWidth::NORMAL,
        text_underline: false,
        text_strikethrough: false,
        text_wrap: true,
        text_color: Color::BLACK,
        horizontal_text_alignment: HorizontalTextAlignment::Start,
        vertical_text_alignment: VerticalTextAlignment::Top,
    };
}

impl Default for ComputedStyle {
    #[inline]
    fn default() -> Self {
        Self::INITIAL
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
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Value($value)); $($t)*)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %initial) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Initial);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: %inherit) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Inherit);)
    };
    ($(($parsed_name:ident, $parsed_property:expr)),*; $name:ident: $value:expr) => {
        $crate::__style_recursive!($(($parsed_name, $parsed_property),)* ($name, $crate::style::Property::Value($value));)
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
