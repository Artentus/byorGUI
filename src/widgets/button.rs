use super::*;
use crate::theme::StyleClass;
use crate::*;

#[derive(Default)]
pub struct ButtonData<'text> {
    text: &'text str,
}

pub type Button<'text, 'style, 'classes> = Widget<'style, 'classes, ButtonData<'text>>;

impl<'style, 'classes> Button<'_, 'style, 'classes> {
    pub const TYPE_CLASS: StyleClass = StyleClass::new_static("###button");

    #[must_use]
    #[inline]
    pub fn text(&self) -> &str {
        self.data().text
    }

    #[must_use]
    #[inline]
    pub fn with_text<'text>(self, text: &'text str) -> Button<'text, 'style, 'classes> {
        self.map_data(|data| ButtonData { text, ..data })
    }

    #[must_use]
    #[inline]
    pub fn with_uid_from_text(self) -> Self {
        let uid = Uid::from_slice(self.data.text.as_bytes());
        self.with_uid(uid)
    }
}

impl WidgetData for ButtonData<'_> {
    #[inline]
    fn type_class(&self) -> StyleClass {
        Button::TYPE_CLASS
    }
}

impl LeafWidgetData for ButtonData<'_> {
    type ShowResult = NodeInputState;

    fn show(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        Ok(gui
            .insert_text_node(Some(uid.produce()), &style, self.text)?
            .input_state)
    }
}
