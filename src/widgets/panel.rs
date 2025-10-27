use super::{Widget, WidgetResult};
use crate::*;

pub struct FlexPanel<'style> {
    uid: Option<Uid>,
    style: &'style Style,
}

impl FlexPanel<'_> {
    #[inline]
    pub const fn uid(&self) -> Option<Uid> {
        self.uid
    }

    #[inline]
    pub const fn style(&self) -> &Style {
        self.style
    }
}

impl Widget for FlexPanel<'_> {
    #[inline]
    fn with_uid(uid: Uid) -> Self {
        Self {
            uid: Some(uid),
            style: &Style::DEFAULT,
        }
    }

    #[inline]
    fn new() -> Self {
        Self {
            uid: None,
            style: &Style::DEFAULT,
        }
    }
}

impl FlexPanel<'_> {
    #[inline]
    pub const fn with_style<'style>(self, style: &'style Style) -> FlexPanel<'style> {
        FlexPanel { style, ..self }
    }
}

impl FlexPanel<'_> {
    #[track_caller]
    pub fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_>,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> WidgetResult<R> {
        Ok(gui
            .insert_container_node(self.uid, self.style, contents)?
            .result)
    }
}
