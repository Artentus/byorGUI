use super::{Widget, WidgetResult};
use crate::*;

pub struct Label<'text, 'style> {
    uid: Option<Uid>,
    text: &'text str,
    style: &'style Style,
}

impl Label<'_, '_> {
    #[inline]
    pub const fn uid(&self) -> Option<Uid> {
        self.uid
    }

    #[inline]
    pub const fn text(&self) -> &str {
        self.text
    }

    #[inline]
    pub const fn style(&self) -> &Style {
        self.style
    }
}

impl Widget for Label<'_, '_> {
    #[inline]
    fn with_uid(uid: Uid) -> Self {
        Self {
            uid: Some(uid),
            text: "",
            style: &Style::DEFAULT,
        }
    }

    #[inline]
    fn new() -> Self {
        Self {
            uid: None,
            text: "",
            style: &Style::DEFAULT,
        }
    }
}

impl<'style> Label<'_, 'style> {
    #[inline]
    pub const fn with_text<'text>(self, text: &'text str) -> Label<'text, 'style> {
        Label { text, ..self }
    }
}

impl<'text> Label<'text, '_> {
    #[inline]
    pub const fn with_style<'style>(self, style: &'style Style) -> Label<'text, 'style> {
        Label { style, ..self }
    }
}

impl Label<'_, '_> {
    #[inline]
    pub const fn with_text_as_uid(mut self) -> Self {
        self.uid = Some(Uid::from_slice(self.text.as_bytes()));
        self
    }
}

impl Label<'_, '_> {
    #[track_caller]
    pub fn show(self, gui: &mut ByorGuiContext<'_>) -> WidgetResult<()> {
        gui.insert_text_node(self.uid, self.style, self.text)?;
        Ok(())
    }
}
