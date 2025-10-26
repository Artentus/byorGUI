pub mod popup;
pub mod scroll;

use crate::style::axis::*;
use crate::*;

#[track_caller]
pub fn generate_uid() -> Uid {
    let location = std::panic::Location::caller();
    Uid::new(location)
}

pub trait Widget: Sized {
    fn with_uid(uid: Uid) -> Self;

    #[track_caller]
    #[inline]
    fn new() -> Self {
        Self::with_uid(generate_uid())
    }
}

impl ByorGuiContext<'_> {
    #[track_caller]
    pub fn button(&mut self, text: &str, uid: Uid, style: &Style) -> NodeResponse<()> {
        self.insert_text_node(Some(uid), style, text)
    }

    #[track_caller]
    #[inline]
    pub fn horizontal_scroll_bar(
        &mut self,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        style: &Style,
    ) -> f32 {
        scroll::ScrollBar::new()
            .with_axis(Axis::X)
            .with_value(value)
            .with_min(min)
            .with_max(max)
            .with_step(step)
            .with_style(style)
            .show(self)
    }

    #[track_caller]
    #[inline]
    pub fn vertical_scroll_bar(
        &mut self,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        style: &Style,
    ) -> f32 {
        scroll::ScrollBar::new()
            .with_axis(Axis::Y)
            .with_value(value)
            .with_min(min)
            .with_max(max)
            .with_step(step)
            .with_style(style)
            .show(self)
    }

    #[track_caller]
    #[inline]
    pub fn horizontal_scroll_view<R>(
        &mut self,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        scroll::ScrollView::new()
            .with_axis(Axis::X)
            .with_style(style)
            .show(self, contents)
    }

    #[track_caller]
    #[inline]
    pub fn vertical_scroll_view<R>(
        &mut self,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> R {
        scroll::ScrollView::new()
            .with_axis(Axis::Y)
            .with_style(style)
            .show(self, contents)
    }

    #[track_caller]
    #[inline]
    pub fn popup<R>(
        &mut self,
        open: &mut bool,
        position: FloatPosition,
        style: &Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> Option<R> {
        popup::Popup::new()
            .with_position(position)
            .with_style(style)
            .show(self, open, contents)
    }
}
