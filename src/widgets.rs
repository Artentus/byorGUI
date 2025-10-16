use super::*;

const SCROLL_BAR_SIZE: Pixel = 20.0;
const SCROLL_BAR_THUMB_SIZE: Pixel = 50.0;

const SCROLL_BAR_LEFT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_left_button");
const SCROLL_BAR_RIGHT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_right_button");
const SCROLL_BAR_UP_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_up_button");
const SCROLL_BAR_DOWN_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_down_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::new(b"##scroll_bar_thumb");

impl ByorGuiContext<'_> {
    pub fn button(&mut self, text: &str, uid: Uid, style: &Style) -> NodeResponse<()> {
        self.insert_text_node(Some(uid), style, text)
    }

    pub fn horizontal_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let parent_style = self.parent_style();
        let computed_style = style.compute(parent_style);

        let scroll_view_style = style
            .clone()
            .with_layout_direction(Direction::TopToBottom)
            .with_initial_child_alignment()
            .with_child_spacing(2.0);

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
            .with_cross_axis_alignment(Alignment::Start);

        let scroll_bar_style = style! {
            width: Sizing::Grow,
            child_spacing: 1.0,
        };

        let scroll_bar_button_style = style! {
            width: SCROLL_BAR_SIZE,
            height: SCROLL_BAR_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        let max_scroll = if let Some(previous_state) = self.get_previous_state(uid) {
            let available_width = previous_state.inner_size.x - previous_state.content_size.x;
            (-available_width).max(0.0)
        } else {
            0.0
        };

        let persistent_state = self.get_persistent_state_mut(uid);
        let scroll = persistent_state.horizontal_scroll.get_or_insert_default();
        *scroll = (*scroll).min(max_scroll);

        let scroll_factor = (max_scroll > 0.0).then_some(*scroll / max_scroll);
        let opposite_scroll_factor = scroll_factor.map(|scroll_factor| 1.0 - scroll_factor);

        let scroll_bar_leading_space_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            flex_ratio: scroll_factor.unwrap_or_default(),
        };
        let scroll_bar_trailing_space_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            flex_ratio: opposite_scroll_factor.unwrap_or_default(),
        };

        let scroll_bar_thumb_style = style! {
            width: Sizing::Grow,
            height: SCROLL_BAR_SIZE,
            min_width: SCROLL_BAR_SIZE,
            max_width: SCROLL_BAR_THUMB_SIZE,
            text_wrap: false,
            horizontal_text_alignment: HorizontalTextAlignment::Center,
            vertical_text_alignment: VerticalTextAlignment::Center,
        };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            let mut scroll_delta = 0.0;

            // scroll container
            let response = gui.insert_container_node(Some(uid), &scroll_container_style, contents);

            if response.is_hovered() {
                // scroll is subtractive in layouting, so we need to subtract here as well
                scroll_delta -= gui.input_state().scroll_delta().x;
            }

            // scroll bar
            if scroll_factor.is_some() || opposite_scroll_factor.is_some() {
                gui.insert_container_node(None, &scroll_bar_style, |mut gui| {
                    if gui
                        .button(
                            "<",
                            uid.concat(SCROLL_BAR_LEFT_BUTTON_UID),
                            &scroll_bar_button_style,
                        )
                        .clicked(MouseButtons::PRIMARY)
                    {
                        scroll_delta -= PIXELS_PER_SCROLL_LINE;
                    }

                    gui.insert_node(None, &scroll_bar_leading_space_style);
                    gui.button(
                        "::",
                        uid.concat(SCROLL_BAR_THUMB_UID),
                        &scroll_bar_thumb_style,
                    );
                    gui.insert_node(None, &scroll_bar_trailing_space_style);

                    if gui
                        .button(
                            ">",
                            uid.concat(SCROLL_BAR_RIGHT_BUTTON_UID),
                            &scroll_bar_button_style,
                        )
                        .clicked(MouseButtons::PRIMARY)
                    {
                        scroll_delta += PIXELS_PER_SCROLL_LINE;
                    }
                });
            }

            if scroll_delta != 0.0 {
                let persistent_state = gui.get_persistent_state_mut(uid);
                let scroll = persistent_state.horizontal_scroll.get_or_insert_default();
                *scroll = (*scroll + scroll_delta).clamp(0.0, max_scroll);
            }

            response.result
        })
        .result
    }
}
