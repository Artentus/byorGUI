use crate::Vec2;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct MouseButtons: u8 {
        const PRIMARY = 0x01;
        const SECONDARY = 0x02;
        const MIDDLE = 0x04;
        const BACK = 0x08;
        const FORWARD = 0x10;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState {
    pub position: Vec2,
    pub pressed_buttons: MouseButtons,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    prev_position: Vec2,
    position: Vec2,

    prev_pressed_buttons: MouseButtons,
    pressed_buttons: MouseButtons,
}

impl InputState {
    pub(crate) fn update(&mut self, mouse_state: MouseState) {
        self.prev_position = self.position;
        self.position = mouse_state.position;

        self.prev_pressed_buttons = self.pressed_buttons;
        self.pressed_buttons = mouse_state.pressed_buttons;
    }

    #[inline]
    pub fn mouse_position(&self) -> Vec2 {
        self.position
    }

    #[inline]
    pub fn mouse_delta(&self) -> Vec2 {
        self.position - self.prev_position
    }

    #[inline]
    pub fn pressed_buttons(&self) -> MouseButtons {
        self.pressed_buttons
    }

    #[inline]
    pub fn clicked_buttons(&self) -> MouseButtons {
        self.pressed_buttons & !self.prev_pressed_buttons
    }

    #[inline]
    pub fn released_buttons(&self) -> MouseButtons {
        self.prev_pressed_buttons & !self.pressed_buttons
    }
}
