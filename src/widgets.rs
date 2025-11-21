pub mod button;
pub mod label;
pub mod panel;
pub mod popup;
pub mod scroll;
pub mod text_box;

use crate::theme::StyleClass;
use crate::*;

pub use button::{Button, CanvasButton, ContentButton};
pub use label::Label;
pub use panel::FlexPanel;
pub use popup::Popup;
pub use scroll::{ScrollBar, ScrollView};
pub use text_box::TextBox;

#[derive(Debug, Clone, Copy)]
pub enum MaybeUid {
    Some(Uid),
    None(&'static std::panic::Location<'static>),
}

impl From<Uid> for MaybeUid {
    #[inline]
    fn from(uid: Uid) -> Self {
        Self::Some(uid)
    }
}

impl From<MaybeUid> for Option<Uid> {
    #[inline]
    fn from(value: MaybeUid) -> Self {
        match value {
            MaybeUid::Some(uid) => Some(uid),
            MaybeUid::None(_) => None,
        }
    }
}

impl MaybeUid {
    #[track_caller]
    #[must_use]
    #[inline]
    pub fn for_caller_location() -> Self {
        Self::None(std::panic::Location::caller())
    }

    #[must_use]
    #[inline]
    pub fn produce(self) -> Uid {
        match self {
            Self::Some(uid) => uid,
            Self::None(location) => Uid::new(location),
        }
    }
}

pub type WidgetResult<T> = Result<T, DuplicateUidError>;

pub trait WidgetData: Sized {
    fn type_class(&self) -> StyleClass;
}

pub trait LeafWidgetData<Renderer: rendering::Renderer>: WidgetData {
    type ShowResult;

    #[track_caller]
    fn show(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult>;
}

pub trait ContainerWidgetData<Renderer: rendering::Renderer>: WidgetData {
    type ShowResult<T>;

    #[track_caller]
    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>>;
}

#[derive(Debug)]
pub struct Widget<'style, 'classes, Data: WidgetData> {
    uid: MaybeUid,
    style: Option<&'style Style>,
    classes: &'classes [StyleClass],
    data: Data,
}

impl<Data: WidgetData + Default> Default for Widget<'_, '_, Data> {
    #[track_caller]
    #[inline]
    fn default() -> Self {
        Self {
            uid: MaybeUid::for_caller_location(),
            style: None,
            classes: &[],
            data: Data::default(),
        }
    }
}

impl<Data: WidgetData> From<Data> for Widget<'_, '_, Data> {
    #[track_caller]
    #[inline]
    fn from(data: Data) -> Self {
        Self {
            uid: MaybeUid::for_caller_location(),
            style: None,
            classes: &[],
            data,
        }
    }
}

impl<'style, 'classes, Data: WidgetData> Widget<'style, 'classes, Data> {
    #[must_use]
    #[inline]
    pub fn type_class(&self) -> StyleClass {
        self.data.type_class()
    }

    #[must_use]
    #[inline]
    pub fn uid(&self) -> Option<Uid> {
        self.uid.into()
    }

    #[must_use]
    #[inline]
    pub fn with_uid(self, uid: Uid) -> Self {
        Self {
            uid: uid.into(),
            ..self
        }
    }

    #[must_use]
    #[inline]
    pub fn style(&self) -> Option<&Style> {
        self.style
    }

    #[must_use]
    #[inline]
    pub fn with_style<'new_style>(
        self,
        style: &'new_style Style,
    ) -> Widget<'new_style, 'classes, Data> {
        Widget {
            style: Some(style),
            ..self
        }
    }

    #[must_use]
    #[inline]
    pub fn classes(&self) -> &[StyleClass] {
        self.classes
    }

    #[must_use]
    #[inline]
    pub fn with_classes<'new_classes>(
        self,
        classes: &'new_classes [StyleClass],
    ) -> Widget<'style, 'new_classes, Data> {
        Widget { classes, ..self }
    }

    #[must_use]
    #[inline]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    #[inline]
    pub fn map_data<NewData: WidgetData>(
        self,
        f: impl FnOnce(Data) -> NewData,
    ) -> Widget<'style, 'classes, NewData> {
        Widget {
            uid: self.uid,
            style: self.style,
            classes: self.classes,
            data: f(self.data),
        }
    }
}

impl<Renderer: rendering::Renderer> ByorGuiContext<'_, Renderer> {
    #[track_caller]
    pub fn show<Data: LeafWidgetData<Renderer>>(
        &mut self,
        widget: Widget<Data>,
    ) -> WidgetResult<Data::ShowResult> {
        let style = self
            .theme()
            .build_style(widget.style, widget.classes, widget.type_class());

        widget.data.show(self, widget.uid, style)
    }

    #[track_caller]
    pub fn show_container<Data: ContainerWidgetData<Renderer>, R>(
        &mut self,
        widget: Widget<Data>,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<Data::ShowResult<R>> {
        let style = self
            .theme()
            .build_style(widget.style, widget.classes, widget.type_class());

        widget.data.show(self, widget.uid, style, contents)
    }

    #[track_caller]
    #[inline]
    pub fn label(&mut self, text: &str) -> WidgetResult<()> {
        self.show(Label::default().with_text(text))
    }

    #[track_caller]
    #[inline]
    pub fn button(&mut self, text: &str) -> WidgetResult<NodeInputState> {
        self.show(Button::default().with_text(text).with_uid_from_text())
    }

    #[track_caller]
    #[inline]
    pub fn content_button<R>(
        &mut self,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>),
    ) -> WidgetResult<NodeInputState> {
        self.show_container(ContentButton::default(), contents)
            .map(|response| response.input_state)
    }

    #[track_caller]
    #[inline]
    pub fn canvas_button(
        &mut self,
        renderer: impl rendering::NodeRenderer<Renderer = Renderer>,
    ) -> WidgetResult<NodeInputState> {
        self.show(CanvasButton::new(renderer))
    }

    #[track_caller]
    #[inline]
    pub fn flex_panel<R>(
        &mut self,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<R> {
        self.show_container(FlexPanel::default(), contents)
    }

    #[track_caller]
    #[inline]
    pub fn horizontal_scroll_bar(&mut self, value: f32, min: f32, max: f32) -> WidgetResult<f32> {
        let scroll_bar = ScrollBar::horizontal()
            .with_value(value)
            .with_min(min)
            .with_max(max);
        self.show(scroll_bar)
    }

    #[track_caller]
    #[inline]
    pub fn vertical_scroll_bar(&mut self, value: f32, min: f32, max: f32) -> WidgetResult<f32> {
        let scroll_bar = ScrollBar::vertical()
            .with_value(value)
            .with_min(min)
            .with_max(max);
        self.show(scroll_bar)
    }

    #[track_caller]
    #[inline]
    pub fn horizontal_scroll_view<R>(
        &mut self,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<R> {
        self.show_container(ScrollView::horizontal(), contents)
    }

    #[track_caller]
    #[inline]
    pub fn vertical_scroll_view<R>(
        &mut self,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<R> {
        self.show_container(ScrollView::vertical(), contents)
    }

    #[track_caller]
    #[inline]
    pub fn popup<R>(
        &mut self,
        open: &mut bool,
        position: FloatPosition,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<Option<R>> {
        self.show_container(Popup::new(open).with_position(position), contents)
    }

    #[track_caller]
    #[inline]
    pub fn text_box(&mut self, text: &mut String) -> WidgetResult<()> {
        self.show(TextBox::new(text))
    }
}
