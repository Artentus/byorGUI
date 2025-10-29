use super::*;
use crate::theme::StyleClass;
use crate::*;

#[derive(Default)]
pub struct FlexPanelData;

pub type FlexPanel<'style, 'classes> = Widget<'style, 'classes, FlexPanelData>;

impl FlexPanel<'_, '_> {
    pub const TYPE_CLASS: StyleClass = StyleClass::new_static("###flex_panel");
}

impl WidgetData for FlexPanelData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        FlexPanel::TYPE_CLASS
    }
}

impl ContainerWidgetData for FlexPanelData {
    type ShowResult<T> = T;

    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>> {
        Ok(gui
            .insert_container_node(uid.into(), &style, contents)?
            .result)
    }
}
