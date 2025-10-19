use super::*;

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
        &self.padding
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
