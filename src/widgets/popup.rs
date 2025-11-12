use super::*;
use crate::theme::StyleClass;
use crate::*;

pub struct PopupData<'open> {
    position: FloatPosition,
    open: &'open mut bool,
}

pub type Popup<'open, 'style, 'classes> = Widget<'style, 'classes, PopupData<'open>>;

impl<'open> Popup<'open, '_, '_> {
    pub const TYPE_CLASS: StyleClass = StyleClass::new_static("###popup");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(open: &'open mut bool) -> Self {
        PopupData {
            position: FloatPosition::default(),
            open,
        }
        .into()
    }

    #[must_use]
    #[inline]
    pub fn position(&self) -> FloatPosition {
        self.data().position
    }

    #[must_use]
    #[inline]
    pub fn with_position(self, position: FloatPosition) -> Self {
        self.map_data(|data| PopupData { position, ..data })
    }
}

impl WidgetData for PopupData<'_> {
    #[inline]
    fn type_class(&self) -> StyleClass {
        Popup::TYPE_CLASS
    }
}

impl ContainerWidgetData for PopupData<'_> {
    type ShowResult<T> = Option<T>;

    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>> {
        let uid = uid.produce();

        let result = if *self.open {
            let response = gui.insert_floating_node(uid, self.position, &style, contents)?;

            //  If this is the first frame the popup opened, do not immediately close it
            let previous_open = gui
                .persistent_state(uid)
                .get::<bool>(PersistentStateKey::PreviousPopupState)
                .copied()
                .unwrap_or(false);

            if previous_open
                && !gui.global_input_state().clicked_buttons().is_empty()
                && !response.is_hovered()
            {
                *self.open = false;
            }

            Some(response.result)
        } else {
            None
        };

        gui.persistent_state_mut(uid)
            .insert(PersistentStateKey::PreviousPopupState, *self.open);

        Ok(result)
    }
}
