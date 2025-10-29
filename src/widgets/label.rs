use super::*;
use crate::theme::StyleClass;
use crate::*;

#[derive(Default)]
pub struct LabelData<'text> {
    text: &'text str,
}

pub type Label<'text, 'style, 'classes> = Widget<'style, 'classes, LabelData<'text>>;

impl<'style, 'classes> Label<'_, 'style, 'classes> {
    pub const TYPE_CLASS: StyleClass = StyleClass::new_static("###label");

    #[must_use]
    #[inline]
    pub fn text(&self) -> &str {
        self.data().text
    }

    #[must_use]
    #[inline]
    pub fn with_text<'text>(self, text: &'text str) -> Label<'text, 'style, 'classes> {
        self.map_data(|data| LabelData { text, ..data })
    }

    #[must_use]
    #[inline]
    pub fn with_uid_from_text(self) -> Self {
        let uid = Uid::from_slice(self.data.text.as_bytes());
        self.with_uid(uid)
    }
}

impl WidgetData for LabelData<'_> {
    #[inline]
    fn type_class(&self) -> StyleClass {
        Label::TYPE_CLASS
    }
}

impl LeafWidgetData for LabelData<'_> {
    type ShowResult = ();

    fn show(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        gui.insert_text_node(uid.into(), &style, self.text)?;
        Ok(())
    }
}
