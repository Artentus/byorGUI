use anyhow::{Result, format_err};
use byor_gui::style::*;
use byor_gui::widgets::*;
use byor_gui::*;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{Renderer, RendererOptions, Scene};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
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

struct ExampleApp {
    context: RenderContext,
    window: Option<Arc<Window>>,
    state: Option<RenderState>,
    gui: ByorGui,
    mouse_state: MouseState,
}

impl ExampleApp {
    fn new() -> Self {
        let mut gui = ByorGui::default();
        *gui.root_style_mut() = style! {
            padding: 5.0,
            child_spacing: 5.0,
        };

        Self {
            context: RenderContext::new(),
            window: None,
            state: None,
            gui,
            mouse_state: MouseState::default(),
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
                PresentMode::Fifo,
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
                window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::MouseButton;

                match button {
                    MouseButton::Left => self.mouse_state.button1_pressed = state.is_pressed(),
                    MouseButton::Right => self.mouse_state.button2_pressed = state.is_pressed(),
                    MouseButton::Middle => self.mouse_state.button3_pressed = state.is_pressed(),
                    _ => (),
                }

                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                window.request_redraw();
            }
            WindowEvent::CursorEntered { .. } | WindowEvent::CursorLeft { .. } => {
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_state.position = Position {
                    x: position.x as Pixel,
                    y: position.y as Pixel,
                };

                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    if (size.width != 0) && (size.height != 0) {
                        self.context
                            .resize_surface(&mut state.surface, size.width, size.height);
                        state.surface_valid = true;
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
                    self.gui.begin_frame(
                        Size {
                            width: surface.config.width as Pixel,
                            height: surface.config.height as Pixel,
                        },
                        self.mouse_state,
                    );

                    gui(&mut self.gui);

                    let mut scene = Scene::new();
                    infallible!(self.gui.end_frame(&mut scene));

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
            }
            _ => (),
        }
    }
}

fn gui(gui: &mut ByorGui) {
    gui.insert_node(
        None,
        &style! {
            width: 100.0,
            height: Sizing::Grow,
        },
    );

    gui.horizontal_scroll_view(
        const { Uid::new(b"scroll_view") },
        &style! {
            width: Sizing::Grow,
            max_width: 600.0,
            flex_ratio: 2.0,
            padding: 5.0,
            child_alignment: Alignment::End,
            cross_axis_alignment: Alignment::Center,
            child_spacing: 5.0,
        },
        |mut gui| {
            for _ in 0..5 {
                gui.insert_node(
                    None,
                    &style! {
                        width: 100.0,
                        height: 100.0,
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
            padding: 5.0,
            child_spacing: 5.0,
        },
        |mut gui| {
            gui.insert_node(
                None,
                &style! {
                    width: Sizing::Grow,
                    height: 100.0,
                },
            );

            gui.insert_container_node(
                None,
                &style! {
                    width: Sizing::Grow,
                    height: Sizing::Grow,
                    padding: 5.0,
                    layout_direction: Direction::TopToBottom,
                    child_alignment: Alignment::Center,
                    child_spacing: 5.0,
                },
                |mut gui| {
                    gui.insert_node(
                        None,
                        &style! {
                            width: 100.0,
                            height: 100.0,
                            cross_axis_alignment: Alignment::Center,
                        },
                    );

                    gui.insert_text_node(
                        None,
                        &style! {
                            width: Sizing::Grow,
                            height: 100.0,
                            padding: 5.0,
                            cross_axis_alignment: Alignment::Center,
                            horizontal_text_alignment: HorizontalTextAlignment::Center,
                            vertical_text_alignment: VerticalTextAlignment::Center,
                        },
                        "lorem ipsum dolor sit amet",
                    );
                },
            );
        },
    );
}
