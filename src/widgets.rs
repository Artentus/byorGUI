use super::*;

const SCROLL_BAR_SIZE: Pixel = 20.0;
const SCROLL_BAR_THUMB_SIZE: Pixel = 50.0;

const SCROLL_BAR_LEFT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_left_button");
const SCROLL_BAR_RIGHT_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_right_button");
const SCROLL_BAR_UP_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_up_button");
const SCROLL_BAR_DOWN_BUTTON_UID: Uid = Uid::new(b"##scroll_bar_down_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::new(b"##scroll_bar_thumb");

pub trait WidgetBuilder: GuiBuilder {
    fn button(&mut self, text: &str, uid: Uid, style: &Style) -> NodeResponse<()> {
        self.insert_text_node(Some(uid), style, text)
    }

    fn horizontal_scroll_view<R>(
        &mut self,
        uid: Uid,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        let parent_style = self.parent_style();
        let computed_style = style.compute(parent_style);

        let scroll_view_style = Style {
            flex_ratio: None,
            layout_direction: Direction::TopToBottom.into(),
            child_alignment: Alignment::default().into(),
            child_spacing: 2.0.into(),
            ..style.clone()
        };

        let scroll_container_style = Style {
            width: Sizing::Grow,
            height: Sizing::Grow,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            padding: Padding::default().into(),
            cross_axis_alignment: Alignment::default().into(),
            allow_horizontal_scoll: true,
            ..computed_style.into_style()
        };

        let scroll_bar_style = Style {
            width: Sizing::Grow,
            height: Sizing::FitContent,
            padding: Padding::default().into(),
            child_spacing: 1.0.into(),
            layout_direction: Direction::LeftToRight.into(),
            ..Default::default()
        };

        let scroll_bar_button_style = Style {
            width: Sizing::Fixed(SCROLL_BAR_SIZE),
            height: Sizing::Fixed(SCROLL_BAR_SIZE),
            allow_text_wrap: false.into(),
            horizontal_text_alignment: HorizontalTextAlignment::Center.into(),
            vertical_text_alignment: VerticalTextAlignment::Center.into(),
            ..Default::default()
        };

        let scroll_factor = if let Some(persistent_state) = self.get_persistent_state(uid) {
            let scroll = persistent_state.horizontal_scroll.unwrap_or_default();
            let available_width =
                persistent_state.inner_size().width - persistent_state.content_size().width;
            let max_scroll = (-available_width).max(0.0);
            (max_scroll > 0.0).then_some(scroll / max_scroll)
        } else {
            None
        };
        let opposite_scroll_factor = scroll_factor.map(|scroll_factor| 1.0 - scroll_factor);

        let scroll_bar_leading_space_style = Style {
            width: Sizing::Grow,
            height: Sizing::Fixed(SCROLL_BAR_SIZE),
            flex_ratio: Some(scroll_factor.unwrap_or_default()),
            ..Default::default()
        };
        let scroll_bar_trailing_space_style = Style {
            width: Sizing::Grow,
            height: Sizing::Fixed(SCROLL_BAR_SIZE),
            flex_ratio: Some(opposite_scroll_factor.unwrap_or_default()),
            ..Default::default()
        };

        let scroll_bar_thumb_style = Style {
            width: Sizing::Grow,
            height: Sizing::Fixed(SCROLL_BAR_SIZE),
            min_width: SCROLL_BAR_SIZE.into(),
            max_width: SCROLL_BAR_THUMB_SIZE.into(),
            flex_ratio: Some(1.0),
            allow_text_wrap: false.into(),
            horizontal_text_alignment: HorizontalTextAlignment::Center.into(),
            vertical_text_alignment: VerticalTextAlignment::Center.into(),
            ..Default::default()
        };

        self.insert_container_node(None, &scroll_view_style, |mut gui| {
            // scroll container
            let result = gui
                .insert_container_node(Some(uid), &scroll_container_style, contents)
                .result;

            // scroll bar
            if scroll_factor.is_some() || opposite_scroll_factor.is_some() {
                gui.insert_container_node(None, &scroll_bar_style, |mut gui| {
                    let mut scroll_delta = 0.0;

                    if gui
                        .button(
                            "<",
                            uid.concat(SCROLL_BAR_LEFT_BUTTON_UID),
                            &scroll_bar_button_style,
                        )
                        .clicked
                    {
                        scroll_delta -= 10.0;
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
                        .clicked
                    {
                        scroll_delta += 10.0;
                    }

                    let persistent_state = gui.get_persistent_state_mut(uid);
                    let available_width =
                        persistent_state.inner_size().width - persistent_state.content_size().width;
                    let max_scroll = (-available_width).max(0.0);
                    let scroll = persistent_state
                        .horizontal_scroll
                        .as_mut()
                        .expect("scroll area is not scrollable");
                    *scroll = (*scroll + scroll_delta).clamp(0.0, max_scroll);
                });
            }

            result
        })
        .result
    }
}

impl<T: GuiBuilder> WidgetBuilder for T {}
