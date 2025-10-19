use crate::{Float, Pixel, Vec2};
use bitflags::bitflags;

pub const PIXELS_PER_SCROLL_LINE: Float<Pixel> = Float::new(40.0);

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
    pub position: Vec2<Pixel>,
    pub pressed_buttons: MouseButtons,
    pub scroll_delta: Vec2<Pixel>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    prev_position: Vec2<Pixel>,
    position: Vec2<Pixel>,

    prev_pressed_buttons: MouseButtons,
    pressed_buttons: MouseButtons,

    scroll_delta: Vec2<Pixel>,
}

impl InputState {
    pub(crate) fn update(&mut self, mouse_state: MouseState) {
        self.prev_position = self.position;
        self.position = mouse_state.position;

        self.prev_pressed_buttons = self.pressed_buttons;
        self.pressed_buttons = mouse_state.pressed_buttons;

        self.scroll_delta = mouse_state.scroll_delta;
    }

    #[inline]
    pub fn mouse_position(&self) -> Vec2<Pixel> {
        self.position
    }

    #[inline]
    pub fn mouse_delta(&self) -> Vec2<Pixel> {
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

    #[inline]
    pub fn scroll_delta(&self) -> Vec2<Pixel> {
        self.scroll_delta
    }
}
