use super::Widget;
use crate::*;

pub struct Popup<'style> {
    uid: Uid,
    position: FloatPosition,
    style: &'style Style,
}

impl Popup<'_> {
    #[inline]
    pub const fn uid(&self) -> Uid {
        self.uid
    }

    #[inline]
    pub const fn position(&self) -> FloatPosition {
        self.position
    }

    #[inline]
    pub const fn style(&self) -> &Style {
        self.style
    }
}

impl Widget for Popup<'_> {
    #[inline]
    fn with_uid(uid: Uid) -> Self {
        Self {
            uid,
            position: FloatPosition::default(),
            style: &Style::DEFAULT,
        }
    }
}

impl Popup<'_> {
    #[inline]
    pub const fn with_position(self, position: FloatPosition) -> Self {
        Self { position, ..self }
    }

    #[inline]
    pub const fn with_style<'style>(self, style: &'style Style) -> Popup<'style> {
        Popup { style, ..self }
    }
}

impl Popup<'_> {
    pub fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_>,
        open: &mut bool,
        contents: impl FnOnce(ByorGuiContext<'_>) -> R,
    ) -> Option<R> {
        let result = if *open {
            let response = gui.insert_floating_node(self.uid, self.position, self.style, contents);

            //  if this is the first frame the popup opened, do not immediately close it
            let previous_open = gui
                .get_persistent_state::<bool>(self.uid, PersistentStateKey::PreviousPopupState)
                .copied()
                .unwrap_or(false);

            if previous_open
                && !gui.global_input_state().clicked_buttons().is_empty()
                && !response.is_hovered()
            {
                *open = false;
            }

            Some(response.result)
        } else {
            None
        };

        gui.insert_persistent_state(self.uid, PersistentStateKey::PreviousPopupState, *open);

        result
    }
}
