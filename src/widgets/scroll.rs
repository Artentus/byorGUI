use super::*;
use crate::style::axis::*;
use crate::theme::StyleClass;
use crate::*;

const SCROLL_BAR_SIZE: Float<Point> = Float::new(20.0);
const SCROLL_BAR_THUMB_SIZE: Float<Point> = Float::new(50.0);
const SCROLL_BAR_SPACING: Float<Point> = Float::new(2.0);

const SCROLL_BAR_UID: Uid = Uid::from_array(b"##scroll_bar");
const SCROLL_BAR_DEC_BUTTON_UID: Uid = Uid::from_array(b"##scroll_bar_dec_button");
const SCROLL_BAR_INC_BUTTON_UID: Uid = Uid::from_array(b"##scroll_bar_inc_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::from_array(b"##scroll_bar_thumb");

pub struct ScrollBarData {
    axis: Axis,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
}

pub type ScrollBar<'style, 'classes> = Widget<'style, 'classes, ScrollBarData>;

impl ScrollBar<'_, '_> {
    pub const HORIZONTAL_TYPE_CLASS: StyleClass =
        StyleClass::new_static("###horizontal_scroll_bar");
    pub const VERTICAL_TYPE_CLASS: StyleClass = StyleClass::new_static("###vertical_scroll_bar");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(axis: Axis) -> Self {
        ScrollBarData {
            axis,
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: None,
        }
        .into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn horizontal() -> Self {
        Self::new(Axis::X)
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn vertical() -> Self {
        Self::new(Axis::Y)
    }

    #[must_use]
    #[inline]
    pub fn value(&self) -> f32 {
        self.data().value
    }

    #[must_use]
    #[inline]
    pub fn with_value(self, value: f32) -> Self {
        self.map_data(|data| ScrollBarData { value, ..data })
    }

    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.data().min
    }

    #[must_use]
    #[inline]
    pub fn with_min(self, min: f32) -> Self {
        self.map_data(|data| ScrollBarData { min, ..data })
    }

    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.data().max
    }

    #[must_use]
    #[inline]
    pub fn with_max(self, max: f32) -> Self {
        self.map_data(|data| ScrollBarData { max, ..data })
    }

    #[must_use]
    #[inline]
    pub fn step(&self) -> Option<f32> {
        self.data().step
    }

    #[must_use]
    #[inline]
    pub fn with_step(self, step: f32) -> Self {
        self.map_data(|data| ScrollBarData {
            step: Some(step),
            ..data
        })
    }
}

impl WidgetData for ScrollBarData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        match self.axis {
            Axis::X => ScrollBar::HORIZONTAL_TYPE_CLASS,
            Axis::Y => ScrollBar::VERTICAL_TYPE_CLASS,
        }
    }
}

impl LeafWidgetData for ScrollBarData {
    type ShowResult = f32;

    fn show(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        let style = style
            .with_layout_direction(self.axis.primary_direction())
            .with_child_spacing(SCROLL_BAR_SPACING / 2.0);

        let step = self.step.unwrap_or((self.max - self.min) * 0.1);
        let mut value = self.value.clamp(self.min, self.max);
        let factor = (value - self.min) / (self.max - self.min);

        let leading_space_style = style! { flex_ratio: factor }
            .with_size_along_axis(self.axis, Sizing::Grow)
            .with_size_along_axis(!self.axis, SCROLL_BAR_SIZE);
        let trailing_space_style = style! { flex_ratio: 1.0 - factor }
            .with_size_along_axis(self.axis, Sizing::Grow)
            .with_size_along_axis(!self.axis, SCROLL_BAR_SIZE);
        let thumb_style = Style::default()
            .with_size_along_axis(self.axis, Sizing::Grow)
            .with_size_along_axis(!self.axis, SCROLL_BAR_SIZE)
            .with_min_size_along_axis(self.axis, SCROLL_BAR_SIZE)
            .with_max_size_along_axis(self.axis, SCROLL_BAR_THUMB_SIZE);

        let button_style = style! {
            width: SCROLL_BAR_SIZE,
            height: SCROLL_BAR_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        let uid = uid.produce();
        let dec_button_uid = uid.concat(SCROLL_BAR_DEC_BUTTON_UID);
        let inc_button_uid = uid.concat(SCROLL_BAR_INC_BUTTON_UID);
        let thumb_uid = uid.concat(SCROLL_BAR_THUMB_UID);

        gui.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .insert_text_node(Some(dec_button_uid), &button_style, "<")?
                .clicked(MouseButtons::PRIMARY)
            {
                value -= step;
            }

            gui.insert_node(None, &leading_space_style)?;

            let response = gui.insert_node(Some(thumb_uid), &thumb_style)?;
            if response.clicked(MouseButtons::PRIMARY) {
                let thumb_pos = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.position.along_axis(self.axis))
                    .unwrap_or_default();

                gui.insert_persistent_state(
                    uid,
                    PersistentStateKey::ScrollBarThumbMouseOffset,
                    gui.global_input_state()
                        .mouse_position()
                        .along_axis(self.axis)
                        - thumb_pos,
                );
            } else if response.pressed(MouseButtons::PRIMARY) {
                let (scroll_bar_pos, scroll_bar_size) = gui
                    .get_previous_state(uid)
                    .map(|state| {
                        (
                            state.position.along_axis(self.axis),
                            state.size.along_axis(self.axis),
                        )
                    })
                    .unwrap_or_default();
                let left_button_size = gui
                    .get_previous_state(dec_button_uid)
                    .map(|state| state.size.along_axis(self.axis))
                    .unwrap_or_default();
                let right_button_size = gui
                    .get_previous_state(inc_button_uid)
                    .map(|state| state.size.along_axis(self.axis))
                    .unwrap_or_default();
                let thumb_size = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.size.along_axis(self.axis))
                    .unwrap_or_default();
                let thumb_mouse_offset: Float<Pixel> = gui
                    .get_persistent_state(uid, PersistentStateKey::ScrollBarThumbMouseOffset)
                    .copied()
                    .unwrap_or(thumb_size / 2.0);

                let parent_style = gui.computed_parent_style();
                let padding = parent_style.padding().along_axis(self.axis);
                let spacing = parent_style.child_spacing();

                let scroll_space = scroll_bar_size
                    - left_button_size
                    - right_button_size
                    - thumb_size
                    - padding[0]
                    - padding[1]
                    - spacing * 4.0;

                let scroll_position = gui
                    .global_input_state()
                    .mouse_position()
                    .along_axis(self.axis)
                    - scroll_bar_pos
                    - left_button_size
                    - thumb_mouse_offset
                    - padding[0]
                    - spacing * 2.0;

                value = (scroll_position / scroll_space) * (self.max - self.min);
            }

            gui.insert_node(None, &trailing_space_style)?;

            if gui
                .insert_text_node(Some(inc_button_uid), &button_style, ">")?
                .clicked(MouseButtons::PRIMARY)
            {
                value += step;
            }

            Ok(value.clamp(self.min, self.max))
        })?
        .result
    }
}

pub struct ScrollViewData {
    axis: Axis,
}

pub type ScrollView<'style, 'classes> = Widget<'style, 'classes, ScrollViewData>;

impl ScrollView<'_, '_> {
    pub const HORIZONTAL_TYPE_CLASS: StyleClass =
        StyleClass::new_static("###horizontal_scroll_view");
    pub const VERTICAL_TYPE_CLASS: StyleClass = StyleClass::new_static("###vertical_scroll_view");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(axis: Axis) -> Self {
        ScrollViewData { axis }.into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn horizontal() -> Self {
        Self::new(Axis::X)
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn vertical() -> Self {
        Self::new(Axis::Y)
    }
}

impl WidgetData for ScrollViewData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        match self.axis {
            Axis::X => ScrollView::HORIZONTAL_TYPE_CLASS,
            Axis::Y => ScrollView::VERTICAL_TYPE_CLASS,
        }
    }
}

impl ContainerWidgetData for ScrollViewData {
    type ShowResult<T> = T;

    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>> {
        let uid = uid.produce();

        let parent_style = gui.parent_style();
        let cascaded_style = style.cascade(parent_style);

        let scroll_view_style = style
            .with_layout_direction(self.axis.cross_direction())
            .with_initial_child_alignment()
            .with_child_spacing(SCROLL_BAR_SPACING);

        let scroll_container_style = cascaded_style
            .as_style()
            .with_width(Sizing::Grow)
            .with_height(Sizing::Grow)
            .with_initial_min_width()
            .with_initial_min_height()
            .with_initial_max_width()
            .with_initial_max_height()
            .with_initial_flex_ratio()
            .with_padding(Padding::ZERO)
            .with_initial_cross_axis_alignment();

        let scroll_bar_style = Style::default().with_size_along_axis(self.axis, Sizing::Grow);

        gui.insert_container_node(None, &scroll_view_style, |mut gui| {
            let mut scroll: Float<Pixel> = gui
                .get_persistent_state(uid, self.axis.persistent_state_scroll_key())
                .copied()
                .unwrap_or_default();
            let mut max_scroll = 0.px();

            let response =
                gui.insert_container_node(Some(uid), &scroll_container_style, |gui| {
                    max_scroll = if let Some(previous_state) = gui.get_previous_state(uid) {
                        let padding = gui.computed_parent_style().padding().along_axis(self.axis);
                        let available_size = previous_state.size.along_axis(self.axis)
                            - padding[0]
                            - padding[1]
                            - previous_state.content_size.along_axis(self.axis);
                        (-available_size).max(0.px())
                    } else {
                        0.px()
                    };

                    contents(gui)
                })?;

            if max_scroll > 0.px() {
                if response.is_hovered() {
                    // Scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui
                        .global_input_state()
                        .scroll_delta()
                        .along_axis(self.axis);
                }

                let scroll_bar = ScrollBar::new(self.axis)
                    .with_uid(uid.concat(SCROLL_BAR_UID))
                    .with_value(scroll.value())
                    .with_min(0.0)
                    .with_max(max_scroll.value())
                    .with_step((POINTS_PER_SCROLL_LINE * gui.scale_factor()).value())
                    .with_style(&scroll_bar_style);
                scroll = gui.show(scroll_bar)?.px();
            }

            gui.insert_persistent_state(uid, self.axis.persistent_state_scroll_key(), scroll);

            Ok(response.result)
        })?
        .result
    }
}
