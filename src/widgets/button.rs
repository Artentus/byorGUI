use super::Widget;
use crate::*;

pub struct Button<'text, 'style> {
    uid: Uid,
    text: &'text str,
    style: &'style Style,
}

impl Button<'_, '_> {
    #[inline]
    pub const fn uid(&self) -> Uid {
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

impl Widget for Button<'_, '_> {
    #[inline]
    fn with_uid(uid: Uid) -> Self {
        Self {
            uid,
            text: "",
            style: &Style::DEFAULT,
        }
    }
}

impl<'style> Button<'_, 'style> {
    #[inline]
    pub const fn with_text<'text>(self, text: &'text str) -> Button<'text, 'style> {
        Button { text, ..self }
    }
}

impl<'text> Button<'text, '_> {
    #[inline]
    pub const fn with_style<'style>(self, style: &'style Style) -> Button<'text, 'style> {
        Button { style, ..self }
    }
}

impl Button<'_, '_> {
    #[inline]
    pub const fn with_text_as_uid(mut self) -> Self {
        self.uid = Uid::from_slice(self.text.as_bytes());
        self
    }
}

impl Button<'_, '_> {
    pub fn show(self, gui: &mut ByorGuiContext<'_>) -> bool {
        gui.insert_text_node(Some(self.uid), self.style, self.text)
            .clicked(MouseButtons::PRIMARY)
    }
}
