use anyhow::{Result, format_err};
use byor_gui::input::*;
use byor_gui::style::*;
use byor_gui::*;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{Renderer, RendererOptions, Scene};
use winit::event::{ElementState, Modifiers, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

fn main() -> Result<()> {
    use winit::event_loop::EventLoop;

    let event_loop = EventLoop::builder().build()?;
    let mut app = ExampleApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

struct RenderState {
    surface: RenderSurface<'static>,
    renderer: Renderer,
    surface_valid: bool,
}

#[derive(Default)]
struct ExampleAppState {
    show_popup: bool,
}

struct ExampleApp {
    context: RenderContext,
    window: Option<Arc<Window>>,
    state: Option<RenderState>,
    required_redraws: u8,
    gui: ByorGui,
    mouse_state: MouseState,
    modifiers: Modifiers,
    app_state: ExampleAppState,
}

impl ExampleApp {
    fn new() -> Self {
        let mut gui = ByorGui::default();
        *gui.root_style_mut() = style! {
            padding: 5.pt(),
            child_spacing: 5.pt(),
            font_size: 16.pt(),
        };

        Self {
            context: RenderContext::new(),
            window: None,
            state: None,
            required_redraws: 2,
            gui,
            mouse_state: MouseState::default(),
            modifiers: Modifiers::default(),
            app_state: ExampleAppState::default(),
        }
    }
}

macro_rules! infallible {
    ($e:expr) => {
        match $e {
            Ok(result) => result,
        }
    };
}

impl winit::application::ApplicationHandler for ExampleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        use vello::AaSupport;
        use vello::wgpu::PresentMode;

        let window = if let Some(window) = self.window.as_ref() {
            window.clone()
        } else {
            assert!(self.state.is_none());

            let window = event_loop
                .create_window(Window::default_attributes().with_title("byorGUI Demo"))
                .expect("failed to create window");
            let window = Arc::new(window);
            self.window = Some(window.clone());
            window
        };

        if self.state.is_none() {
            let window_size = window.inner_size();
            let surface = pollster::block_on(self.context.create_surface(
                window,
                window_size.width,
                window_size.height,
                PresentMode::AutoNoVsync, // UI feels significantly less responsive when using FiFo modes
            ))
            .expect("failed to create surface");

            let device_handle = &self.context.devices[surface.dev_id];
            let renderer = Renderer::new(
                &device_handle.device,
                RendererOptions {
                    antialiasing_support: AaSupport::area_only(),
                    ..Default::default()
                },
            )
            .map_err(|e| format_err!("{e}"))
            .expect("failed to create renderer");

            self.state = Some(RenderState {
                surface,
                renderer,
                surface_valid: false,
            });
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_deref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ScaleFactorChanged { .. } => {
                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::MouseButton;

                let buttons = match button {
                    MouseButton::Left => MouseButtons::PRIMARY,
                    MouseButton::Right => MouseButtons::SECONDARY,
                    MouseButton::Middle => MouseButtons::MIDDLE,
                    MouseButton::Back => MouseButtons::BACK,
                    MouseButton::Forward => MouseButtons::FORWARD,
                    MouseButton::Other(_) => MouseButtons::empty(),
                };

                match state {
                    ElementState::Pressed => self.mouse_state.pressed_buttons |= buttons,
                    ElementState::Released => self.mouse_state.pressed_buttons &= !buttons,
                }

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        let pixels_per_scroll_line =
                            POINTS_PER_SCROLL_LINE.to_pixel(window.scale_factor() as f32);

                        if self.modifiers.state().contains(ModifiersState::CONTROL) {
                            Vec2 {
                                x: y * pixels_per_scroll_line,
                                y: x * pixels_per_scroll_line,
                            }
                        } else {
                            Vec2 {
                                x: x * pixels_per_scroll_line,
                                y: y * pixels_per_scroll_line,
                            }
                        }
                    }
                    MouseScrollDelta::PixelDelta(physical_position) => Vec2 {
                        x: physical_position.x.px(),
                        y: physical_position.y.px(),
                    },
                };
                self.mouse_state.scroll_delta += delta;

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::CursorEntered { .. } | WindowEvent::CursorLeft { .. } => {
                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_state.position = Vec2 {
                    x: position.x.px(),
                    y: position.y.px(),
                };

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    if (size.width != 0) && (size.height != 0) {
                        self.context
                            .resize_surface(&mut state.surface, size.width, size.height);
                        state.surface_valid = true;

                        self.required_redraws = self.required_redraws.max(2);
                        window.request_redraw();
                    } else {
                        state.surface_valid = false;
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                use vello::wgpu::{CommandEncoderDescriptor, PollType};
                use vello::{AaConfig, RenderParams};

                if let Some(&mut RenderState {
                    ref mut surface,
                    ref mut renderer,
                    surface_valid,
                }) = self.state.as_mut()
                    && surface_valid
                {
                    self.gui.frame(
                        Vec2 {
                            x: surface.config.width.px(),
                            y: surface.config.height.px(),
                        },
                        window.scale_factor() as f32,
                        self.mouse_state,
                        |gui| build_gui(&mut self.app_state, gui),
                    );
                    self.mouse_state.scroll_delta = Vec2::ZERO;

                    let mut scene = Scene::new();
                    infallible!(self.gui.render(&mut scene));

                    let device_handle = &self.context.devices[surface.dev_id];
                    let render_params = RenderParams {
                        base_color: vello::peniko::Color::BLACK,
                        width: surface.config.width,
                        height: surface.config.height,
                        antialiasing_method: AaConfig::Area,
                    };
                    renderer
                        .render_to_texture(
                            &device_handle.device,
                            &device_handle.queue,
                            &scene,
                            &surface.target_view,
                            &render_params,
                        )
                        .expect("failed to render scene");

                    let frame_buffer = surface
                        .surface
                        .get_current_texture()
                        .expect("failed to aquire frame buffer");
                    let mut encoder =
                        device_handle
                            .device
                            .create_command_encoder(&CommandEncoderDescriptor {
                                label: Some("Frame Buffer Blit"),
                            });
                    surface.blitter.copy(
                        &device_handle.device,
                        &mut encoder,
                        &surface.target_view,
                        &frame_buffer.texture.create_view(&Default::default()),
                    );
                    device_handle.queue.submit([encoder.finish()]);
                    frame_buffer.present();

                    device_handle.device.poll(PollType::Poll).unwrap();
                }

                self.required_redraws = self.required_redraws.saturating_sub(1);
                if self.required_redraws > 0 {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn build_gui(app_state: &mut ExampleAppState, mut gui: ByorGuiContext<'_>) {
    gui.vertical_scroll_view(
        &style! {
            height: Sizing::Grow,
            max_height: 600.pt(),
            padding: 5.pt(),
            child_alignment: Alignment::End,
            cross_axis_alignment: Alignment::Center,
            child_spacing: 5.pt(),
            layout_direction: Direction::TopToBottom,
        },
        |mut gui| {
            for _ in 0..5 {
                gui.insert_node(
                    None,
                    &style! {
                        width: 100.pt(),
                        height: 100.pt(),
                    },
                );
            }
        },
    );

    gui.horizontal_scroll_view(
        &style! {
            width: Sizing::Grow,
            max_width: 600.pt(),
            flex_ratio: 2.0,
            padding: 5.pt(),
            child_alignment: Alignment::End,
            cross_axis_alignment: Alignment::Center,
            child_spacing: 5.pt(),
            layout_direction: Direction::LeftToRight,
        },
        |mut gui| {
            for _ in 0..5 {
                gui.insert_node(
                    None,
                    &style! {
                        width: 100.pt(),
                        height: 100.pt(),
                    },
                );
            }
        },
    );

    gui.insert_container_node(
        None,
        &style! {
            width: Sizing::Grow,
            height: Sizing::Grow,
            layout_direction: Direction::TopToBottom,
            padding: 5.pt(),
            child_spacing: 5.pt(),
        },
        |mut gui| {
            gui.insert_node(
                None,
                &style! {
                    width: Sizing::Grow,
                    height: 100.pt(),
                },
            );

            gui.insert_container_node(
                None,
                &style! {
                    width: Sizing::Grow,
                    height: Sizing::Grow,
                    padding: 5.pt(),
                    layout_direction: Direction::TopToBottom,
                    child_alignment: Alignment::Center,
                    child_spacing: 5.pt(),
                },
                |mut gui| {
                    gui.insert_text_node(
                        None,
                        &style! {
                            width: Sizing::Grow,
                            height: 100.pt(),
                            padding: 5.pt(),
                            cross_axis_alignment: Alignment::Center,
                            horizontal_text_alignment: HorizontalTextAlignment::Center,
                            vertical_text_alignment: VerticalTextAlignment::Center,
                        },
                        "Lorem ipsum dolor sit amet",
                    );

                    gui.insert_container_node(
                        const { Some(Uid::from_slice(b"popup_parent")) },
                        &style! {
                            width: 100.pt(),
                            height: 100.pt(),
                            cross_axis_alignment: Alignment::Center,
                        },
                        |mut gui| {
                            if gui.parent_input_state().clicked(MouseButtons::SECONDARY) {
                                app_state.show_popup = true;
                            }

                            gui.popup(
                                &mut app_state.show_popup,
                                FloatPosition::CursorFixed,
                                &style! {
                                    padding: 5.pt(),
                                },
                                |mut gui| {
                                    gui.insert_text_node(
                                        None,
                                        &style! {
                                            max_width: 300.px(),
                                            padding: 5.pt(),
                                            text_wrap: true,
                                            horizontal_text_alignment: HorizontalTextAlignment::Justify,
                                        },
                                        include_str!("lorem_ipsum.txt"),
                                    );
                                },
                            );
                        }
                    );
                },
            );
        },
    );
}
