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

impl<Renderer: rendering::Renderer> LeafWidgetData<Renderer> for ButtonData<'_> {
    type ShowResult = NodeInputState;

    fn show(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        Ok(gui
            .insert_node(Some(uid.produce()), &style, NodeContents::text(self.text))?
            .input_state)
    }
}

#[derive(Default)]
pub struct ContentButtonData;

pub type ContentButton<'style, 'classes> = Widget<'style, 'classes, ContentButtonData>;

impl<'style, 'classes> ContentButton<'style, 'classes> {
    pub const TYPE_CLASS: StyleClass = Button::TYPE_CLASS;
}

impl WidgetData for ContentButtonData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        Button::TYPE_CLASS
    }
}

impl<Renderer: rendering::Renderer> ContainerWidgetData<Renderer> for ContentButtonData {
    type ShowResult<T> = NodeResponse<T>;

    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>> {
        gui.insert_node(Some(uid.produce()), &style, NodeContents::builder(contents))
    }
}

pub struct CanvasButtonData<NR: rendering::NodeRenderer> {
    renderer: NR,
}

pub type CanvasButton<'style, 'classes, NR> = Widget<'style, 'classes, CanvasButtonData<NR>>;

impl<'style, 'classes, NR: rendering::NodeRenderer> CanvasButton<'style, 'classes, NR> {
    pub const TYPE_CLASS: StyleClass = Button::TYPE_CLASS;

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(renderer: NR) -> Self {
        CanvasButtonData { renderer }.into()
    }
}

impl<NR: rendering::NodeRenderer> WidgetData for CanvasButtonData<NR> {
    #[inline]
    fn type_class(&self) -> StyleClass {
        Button::TYPE_CLASS
    }
}

impl<Renderer, NR> LeafWidgetData<Renderer> for CanvasButtonData<NR>
where
    Renderer: rendering::Renderer,
    NR: rendering::NodeRenderer<Renderer = Renderer>,
{
    type ShowResult = NodeInputState;

    fn show(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        Ok(gui
            .insert_node(
                Some(uid.produce()),
                &style,
                NodeContents::renderer(self.renderer),
            )?
            .input_state)
    }
}
