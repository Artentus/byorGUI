use super::*;

const SCROLL_BAR_SIZE: Float<Point> = Float::new(20.0);
const SCROLL_BAR_THUMB_SIZE: Float<Point> = Float::new(50.0);
const SCROLL_BAR_SPACING: Float<Point> = Float::new(2.0);

const SCROLL_BAR_UID: Uid = Uid::new(b"##scroll_bar");
const SCROLL_BAR_LEFT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_left_button");
const SCROLL_BAR_RIGHT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_right_button");
const SCROLL_BAR_UP_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_up_button");
const SCROLL_BAR_DOWN_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_down_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::new(b"##scroll_bar_thumb");

impl ByorGuiContext<'_> {
    pub fn button(&mut self, text: &str, uid: Uid, style: &Style) -> NodeResponse<()> {
        self.insert_text_node(Some(uid), style, text)
    }

    pub fn horizontal_scroll_bar(
        &mut self,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
        uid: Uid,
        style: &Style,
    ) {
        let style = style
            .clone()
            .with_layout_direction(Direction::LeftToRight)
            .with_child_spacing((SCROLL_BAR_SPACING / 2.0).into());

        *value = (*value).clamp(min, max);
        let factor = (*value - min) / (max - min);

        let leading_space_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            flex_ratio: factor,
        };
        let trailing_space_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            flex_ratio: 1.0 - factor,
        };

        let thumb_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            min_width: SCROLL_BAR_SIZE,
            max_width: SCROLL_BAR_THUMB_SIZE,
        };

        let button_style = style! {
            width: SCROLL_BAR_SIZE,
            height: SCROLL_BAR_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        let left_button_uid = uid.concat(SCROLL_BAR_LEFT_BUTTON_UID);
        let right_button_uid = uid.concat(SCROLL_BAR_RIGHT_BUTTON_UID);
        let thumb_uid = uid.concat(SCROLL_BAR_THUMB_UID);

        self.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .button("🞀", left_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value -= step;
            }

            gui.insert_node(None, &leading_space_style);

            let response = gui.insert_node(Some(thumb_uid), &thumb_style);
            if response.clicked(MouseButtons::PRIMARY) {
                let thumb_x = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.position.x)
                    .unwrap_or_default();

                gui.insert_persistent_state(
                    uid,
                    PersistentStateKey::ScrollBarThumbMouseOffset,
                    gui.input_state().mouse_position().x - thumb_x,
                );
            } else if response.pressed(MouseButtons::PRIMARY) {
                let (scroll_bar_x, scroll_bar_width) = gui
                    .get_previous_state(uid)
                    .map(|state| (state.position.x, state.size.x))
                    .unwrap_or_default();
                let left_button_width = gui
                    .get_previous_state(left_button_uid)
                    .map(|state| state.size.x)
                    .unwrap_or_default();
                let right_button_width = gui
                    .get_previous_state(right_button_uid)
                    .map(|state| state.size.x)
                    .unwrap_or_default();
                let thumb_width = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.size.x)
                    .unwrap_or_default();
                let thumb_mouse_offset: Float<Pixel> = gui
                    .get_persistent_state(uid, PersistentStateKey::ScrollBarThumbMouseOffset)
                    .copied()
                    .unwrap_or(thumb_width / 2.0);

                let parent_style = gui.computed_parent_style();
                let padding = parent_style.padding();
                let spacing = parent_style.child_spacing();

                let scroll_space = scroll_bar_width
                    - left_button_width
                    - right_button_width
                    - thumb_width
                    - padding.left
                    - padding.right
                    - spacing * 4.0;

                let scroll_position = gui.input_state().mouse_position().x
                    - scroll_bar_x
                    - left_button_width
                    - thumb_mouse_offset
                    - padding.left
                    - spacing * 2.0;

                *value = (scroll_position / scroll_space) * (max - min);
            }

            gui.insert_node(None, &trailing_space_style);

            if gui
                .button("🞂", right_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value += step;
            }
        });

        *value = (*value).clamp(min, max);
    }

    pub fn vertical_scroll_bar(
        &mut self,
        value: &mut f32,
        min: f32,
        max: f32,
        step: f32,
        uid: Uid,
        style: &Style,
    ) {
        let style = style
            .clone()
            .with_layout_direction(Direction::TopToBottom)
            .with_child_spacing((SCROLL_BAR_SPACING / 2.0).into());

        *value = (*value).clamp(min, max);
        let factor = (*value - min) / (max - min);

        let leading_space_style = style! {
            width: SCROLL_BAR_SIZE,
            height: Sizing::Grow,
            flex_ratio: factor,
        };
        let trailing_space_style = style! {
            width: SCROLL_BAR_SIZE,
            height: Sizing::Grow,
            flex_ratio: 1.0 - factor,
        };

        let thumb_style = style! {
            width: SCROLL_BAR_SIZE,
            height: Sizing::Grow,
            min_height: SCROLL_BAR_SIZE,
            max_height: SCROLL_BAR_THUMB_SIZE,
        };

        let button_style = style! {
            width: SCROLL_BAR_SIZE,
            height: SCROLL_BAR_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        let up_button_uid = uid.concat(SCROLL_BAR_UP_BUTTON_UID);
        let down_button_uid = uid.concat(SCROLL_BAR_DOWN_BUTTON_UID);
        let thumb_uid = uid.concat(SCROLL_BAR_THUMB_UID);

        self.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .button("🞁", up_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value -= step;
            }

            gui.insert_node(None, &leading_space_style);

            let response = gui.insert_node(Some(thumb_uid), &thumb_style);
            if response.clicked(MouseButtons::PRIMARY) {
                let thumb_y = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.position.y)
                    .unwrap_or_default();

                gui.insert_persistent_state(
                    uid,
                    PersistentStateKey::ScrollBarThumbMouseOffset,
                    gui.input_state().mouse_position().y - thumb_y,
                );
            } else if response.pressed(MouseButtons::PRIMARY) {
                let (scroll_bar_y, scroll_bar_height) = gui
                    .get_previous_state(uid)
                    .map(|state| (state.position.y, state.size.y))
                    .unwrap_or_default();
                let up_button_height = gui
                    .get_previous_state(up_button_uid)
                    .map(|state| state.size.y)
                    .unwrap_or_default();
                let down_button_height = gui
                    .get_previous_state(down_button_uid)
                    .map(|state| state.size.y)
                    .unwrap_or_default();
                let thumb_height = gui
                    .get_previous_state(thumb_uid)
                    .map(|state| state.size.y)
                    .unwrap_or_default();
                let thumb_mouse_offset: Float<Pixel> = gui
                    .get_persistent_state(uid, PersistentStateKey::ScrollBarThumbMouseOffset)
                    .copied()
                    .unwrap_or(thumb_height / 2.0);

                let parent_style = gui.computed_parent_style();
                let padding = parent_style.padding();
                let spacing = parent_style.child_spacing();

                let scroll_space = scroll_bar_height
                    - up_button_height
                    - down_button_height
                    - thumb_height
                    - padding.top
                    - padding.bottom
                    - spacing * 4.0;

                let scroll_position = gui.input_state().mouse_position().y
                    - scroll_bar_y
                    - up_button_height
                    - thumb_mouse_offset
                    - padding.top
                    - spacing * 2.0;

                *value = (scroll_position / scroll_space) * (max - min);
            }

            gui.insert_node(None, &trailing_space_style);

            if gui
                .button("🞃", down_button_uid, &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value += step;
            }
        });

        *value = (*value).clamp(min, max);
    }

    pub fn horizontal_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let scroll_view_style = style
            .clone()
            .with_layout_direction(Direction::TopToBottom)
            .with_initial_child_alignment()
            .with_child_spacing(SCROLL_BAR_SPACING.into());

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

        let scroll_bar_style = style! { width: Sizing::Grow };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let mut scroll: Float<Pixel> = gui
                .get_persistent_state(uid, PersistentStateKey::HorizontalScroll)
                .copied()
                .unwrap_or_default();
            let mut max_scroll = 0.px();

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, |gui| {
                max_scroll = if let Some(previous_state) = gui.get_previous_state(uid) {
                    let padding = gui.computed_parent_style().padding();
                    let available_width = previous_state.size.x
                        - padding.left
                        - padding.right
                        - previous_state.content_size.x;
                    (-available_width).max(0.px())
                } else {
                    0.px()
                };

                contents(gui)
            });

            if max_scroll > 0.px() {
                if response.is_hovered() {
                    // scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui.input_state().scroll_delta().x;
                }

                // scroll bar
                let mut scroll_value = scroll.value();
                gui.horizontal_scroll_bar(
                    &mut scroll_value,
                    0.0,
                    max_scroll.value(),
                    PIXELS_PER_SCROLL_LINE.value(),
                    uid.concat(SCROLL_BAR_UID),
                    &scroll_bar_style,
                );
                scroll = scroll_value.px();
            }

            gui.insert_persistent_state(uid, PersistentStateKey::HorizontalScroll, scroll);

            response.result
        })
        .result
    }

    pub fn vertical_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let scroll_view_style = style
            .clone()
            .with_layout_direction(Direction::LeftToRight)
            .with_initial_child_alignment()
            .with_child_spacing(SCROLL_BAR_SPACING.into());

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

        let scroll_bar_style = style! { height: Sizing::Grow };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let mut scroll: Float<Pixel> = gui
                .get_persistent_state(uid, PersistentStateKey::VerticalScroll)
                .copied()
                .unwrap_or_default();
            let mut max_scroll = 0.px();

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, |gui| {
                max_scroll = if let Some(previous_state) = gui.get_previous_state(uid) {
                    let padding = gui.computed_parent_style().padding();
                    let available_height = previous_state.size.y
                        - padding.top
                        - padding.bottom
                        - previous_state.content_size.y;
                    (-available_height).max(0.px())
                } else {
                    0.px()
                };

                contents(gui)
            });

            if max_scroll > 0.px() {
                if response.is_hovered() {
                    // scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui.input_state().scroll_delta().y;
                }

                // scroll bar
                let mut scroll_value = scroll.value();
                gui.vertical_scroll_bar(
                    &mut scroll_value,
                    0.0,
                    max_scroll.value(),
                    PIXELS_PER_SCROLL_LINE.value(),
                    uid.concat(SCROLL_BAR_UID),
                    &scroll_bar_style,
                );
                scroll = scroll_value.px();
            }

            gui.insert_persistent_state(uid, PersistentStateKey::VerticalScroll, scroll);

            response.result
        })
        .result
    }
}
