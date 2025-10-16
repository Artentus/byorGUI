use super::*;

const SCROLL_BAR_SIZE: Pixel = 20.0;
const SCROLL_BAR_THUMB_SIZE: Pixel = 50.0;

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
        let style = style.clone().with_layout_direction(Direction::LeftToRight);

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

        self.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .button("🞀", uid.concat(SCROLL_BAR_LEFT_BUTTON_UID), &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value -= step;
            }

            gui.insert_node(None, &leading_space_style);
            gui.insert_node(Some(uid.concat(SCROLL_BAR_THUMB_UID)), &thumb_style);
            gui.insert_node(None, &trailing_space_style);

            if gui
                .button("🞂", uid.concat(SCROLL_BAR_RIGHT_BUTTON_UID), &button_style)
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
        let style = style.clone().with_layout_direction(Direction::TopToBottom);

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

        self.insert_container_node(Some(uid), &style, |mut gui| {
            if gui
                .button("🞁", uid.concat(SCROLL_BAR_UP_BUTTON_UID), &button_style)
                .clicked(MouseButtons::PRIMARY)
            {
                *value -= step;
            }

            gui.insert_node(None, &leading_space_style);
            gui.insert_node(Some(uid.concat(SCROLL_BAR_THUMB_UID)), &thumb_style);
            gui.insert_node(None, &trailing_space_style);

            if gui
                .button("🞃", uid.concat(SCROLL_BAR_DOWN_BUTTON_UID), &button_style)
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
            .with_child_spacing(2.0);

        let parent_style = self.parent_style();
        let computed_style = style.compute(parent_style);
        let scroll_container_style = computed_style
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

        let scroll_bar_style = style! {
            width: Sizing::Grow,
            child_spacing: 1.0,
        };

        let max_scroll = if let Some(previous_state) = self.get_previous_state(uid) {
            let available_width = previous_state.inner_size.x - previous_state.content_size.x;
            (-available_width).max(0.0)
        } else {
            0.0
        };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let persistent_state = gui.get_persistent_state(uid);
            let mut scroll = persistent_state.horizontal_scroll.unwrap_or_default();

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, contents);

            if max_scroll > 0.0 {
                if response.is_hovered() {
                    // scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui.input_state().scroll_delta().x;
                }

                // scroll bar
                gui.horizontal_scroll_bar(
                    &mut scroll,
                    0.0,
                    max_scroll,
                    PIXELS_PER_SCROLL_LINE,
                    uid.concat(SCROLL_BAR_UID),
                    &scroll_bar_style,
                );
            }

            let persistent_state = gui.get_persistent_state_mut(uid);
            persistent_state.horizontal_scroll = Some(scroll);

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
            .with_child_spacing(2.0);

        let parent_style = self.parent_style();
        let computed_style = style.compute(parent_style);
        let scroll_container_style = computed_style
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

        let scroll_bar_style = style! {
            height: Sizing::Grow,
            child_spacing: 1.0,
        };

        let max_scroll = if let Some(previous_state) = self.get_previous_state(uid) {
            let available_width = previous_state.inner_size.y - previous_state.content_size.y;
            (-available_width).max(0.0)
        } else {
            0.0
        };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let persistent_state = gui.get_persistent_state(uid);
            let mut scroll = persistent_state.vertical_scroll.unwrap_or_default();

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, contents);

            if max_scroll > 0.0 {
                if response.is_hovered() {
                    // scroll is subtractive in layouting, so we need to subtract here as well
                    scroll -= gui.input_state().scroll_delta().y;
                }

                // scroll bar
                gui.vertical_scroll_bar(
                    &mut scroll,
                    0.0,
                    max_scroll,
                    PIXELS_PER_SCROLL_LINE,
                    uid.concat(SCROLL_BAR_UID),
                    &scroll_bar_style,
                );
            }

            let persistent_state = gui.get_persistent_state_mut(uid);
            persistent_state.vertical_scroll = Some(scroll);

            response.result
        })
        .result
    }
}
