use crate::style::axis::*;
use crate::*;

const SCROLL_BAR_SIZE: Float<Point> = Float::new(20.0);
const SCROLL_BAR_THUMB_SIZE: Float<Point> = Float::new(50.0);
const SCROLL_BAR_SPACING: Float<Point> = Float::new(2.0);

const SCROLL_BAR_UID: Uid = Uid::new(b"##scroll_bar");
const SCROLL_BAR_DEC_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_dec_button");
const SCROLL_BAR_INC_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_inc_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::new(b"##scroll_bar_thumb");

impl ByorGuiContext<'_> {
    pub fn button(&mut self, text: &str, uid: Uid, style: &Style) -> NodeResponse<()> {
        self.insert_text_node(Some(uid), style, text)
    }

    pub fn scroll_bar(
        &mut self,
        axis: Axis,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
        uid: Uid,
        style: &Style,
    ) {
        let style = style
            .clone()
            .with_layout_direction(axis.primary_direction())
            .with_child_spacing(SCROLL_BAR_SPACING / 2.0);

        *value = (*value).clamp(min, max);
        let factor = (*value - min) / (max - min);

        let leading_space_style = style! { flex_ratio: factor }
            .with_size_along_axis(axis, Sizing::Grow)
            .with_size_along_axis(!axis, SCROLL_BAR_SIZE);
        let trailing_space_style = style! { flex_ratio: 1.0 - factor }
            .with_size_along_axis(axis, Sizing::Grow)
            .with_size_along_axis(!axis, SCROLL_BAR_SIZE);
        let thumb_style = Style::default()
            .with_size_along_axis(axis, Sizing::Grow)
            .with_size_along_axis(!axis, SCROLL_BAR_SIZE)
            .with_min_size_along_axis(axis, SCROLL_BAR_SIZE)
            .with_max_size_along_axis(axis, SCROLL_BAR_THUMB_SIZE);

        let button_style = style! {
            width: SCROLL_BAR_SIZE,
            height: SCROLL_BAR_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        let dec_button_uid = uid.concat(SCROLL_BAR_DEC_BUTTON_UID);
        let inc_button_uid = uid.concat(SCROLL_BAR_INC_BUTTON_UID);
        let thumb_uid = uid.concat(SCROLL_BAR_THUMB_UID);

        self.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .button("<", dec_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value -= step;
            }

            gui.insert_node(None, &leading_space_style);

            let response = gui.insert_node(Some(thumb_uid), &thumb_style);
            if response.clicked(MouseButtons::PRIMARY) {
                let thumb_pos = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.position.along_axis(axis))
                    .unwrap_or_default();

                gui.insert_persistent_state(
                    uid,
                    PersistentStateKey::ScrollBarThumbMouseOffset,
                    gui.input_state().mouse_position().along_axis(axis) - thumb_pos,
                );
            } else if response.pressed(MouseButtons::PRIMARY) {
                let (scroll_bar_pos, scroll_bar_size) = gui
                    .get_previous_state(uid)
                    .map(|state| (state.position.along_axis(axis), state.size.along_axis(axis)))
                    .unwrap_or_default();
                let left_button_size = gui
                    .get_previous_state(dec_button_uid)
                    .map(|state| state.size.along_axis(axis))
                    .unwrap_or_default();
                let right_button_size = gui
                    .get_previous_state(inc_button_uid)
                    .map(|state| state.size.along_axis(axis))
                    .unwrap_or_default();
                let thumb_size = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.size.along_axis(axis))
                    .unwrap_or_default();
                let thumb_mouse_offset: Float<Pixel> = gui
                    .get_persistent_state(uid, PersistentStateKey::ScrollBarThumbMouseOffset)
                    .copied()
                    .unwrap_or(thumb_size / 2.0);

                let parent_style = gui.computed_parent_style();
                let padding = parent_style.padding().along_axis(axis);
                let spacing = parent_style.child_spacing();

                let scroll_space = scroll_bar_size
                    - left_button_size
                    - right_button_size
                    - thumb_size
                    - padding[0]
                    - padding[1]
                    - spacing * 4.0;

                let scroll_position = gui.input_state().mouse_position().along_axis(axis)
                    - scroll_bar_pos
                    - left_button_size
                    - thumb_mouse_offset
                    - padding[0]
                    - spacing * 2.0;

                *value = (scroll_position / scroll_space) * (max - min);
            }

            gui.insert_node(None, &trailing_space_style);

            if gui
                .button(">", inc_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value += step;
            }
        });

        *value = (*value).clamp(min, max);
    }

    #[inline]
    pub fn horizontal_scroll_bar(
        &mut self,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
        uid: Uid,
        style: &Style,
    ) {
        self.scroll_bar(Axis::X, value, min, max, step, uid, style);
    }

    #[inline]
    pub fn vertical_scroll_bar(
        &mut self,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
        uid: Uid,
        style: &Style,
    ) {
        self.scroll_bar(Axis::Y, value, min, max, step, uid, style);
    }

    pub fn scroll_view<R>(
        &mut self,
        axis: Axis,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let scroll_view_style = style
            .clone()
            .with_layout_direction(axis.cross_direction())
            .with_initial_child_alignment()
            .with_child_spacing(SCROLL_BAR_SPACING);

        let parent_style = self.parent_style();
        let cascaded_style = style.cascade(parent_style);
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

        let scroll_bar_style = Style::default().with_size_along_axis(axis, Sizing::Grow);

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let mut scroll: Float<Pixel> = gui
                .get_persistent_state(uid, axis.persistent_state_scroll_key())
                .copied()
                .unwrap_or_default();
            let mut max_scroll = 0.px();

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, |gui| {
                max_scroll = if let Some(previous_state) = gui.get_previous_state(uid) {
                    let padding = gui.computed_parent_style().padding().along_axis(axis);
                    let available_size = previous_state.size.along_axis(axis)
                        - padding[0]
                        - padding[1]
                        - previous_state.content_size.along_axis(axis);
                    (-available_size).max(0.px())
                } else {
                    0.px()
                };

                contents(gui)
            });

            if max_scroll > 0.px() {
                if response.is_hovered() {
                    // scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui.input_state().scroll_delta().along_axis(axis);
                }

                // scroll bar
                let mut scroll_value = scroll.value();
                gui.scroll_bar(
                    axis,
                    &mut scroll_value,
                    0.0,
                    max_scroll.value(),
                    PIXELS_PER_SCROLL_LINE.value(),
                    uid.concat(SCROLL_BAR_UID),
                    &scroll_bar_style,
                );
                scroll = scroll_value.px();
            }

            gui.insert_persistent_state(uid, axis.persistent_state_scroll_key(), scroll);

            response.result
        })
        .result
    }

    #[inline]
    pub fn horizontal_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        self.scroll_view(Axis::X, uid, style, contents)
    }

    #[inline]
    pub fn vertical_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        self.scroll_view(Axis::Y, uid, style, contents)
    }
}
