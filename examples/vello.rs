use anyhow::{Result, format_err};
use byor_gui::input::*;
use byor_gui::style::*;
use byor_gui::theme::*;
use byor_gui::widgets::*;
use byor_gui::*;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{Renderer, RendererOptions, Scene};
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
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

#[derive(Default)]
struct ExampleAppState {
    show_popup: bool,
}

struct ExampleApp {
    context: RenderContext,
    window: Option<Arc<Window>>,
    state: Option<RenderState>,
    required_redraws: u8,
    gui: ByorGui<Scene>,
    app_state: ExampleAppState,
}

impl ExampleApp {
    fn new() -> Self {
        let mut gui = ByorGui::default();
        create_theme(gui.theme_mut());

        Self {
            context: RenderContext::new(),
            window: None,
            state: None,
            required_redraws: 2,
            gui,
            app_state: ExampleAppState::default(),
        }
    }
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.gui.set_scale_factor(scale_factor as f32);

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.gui.on_input_event(event.into());

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Ok(button) = button.try_into() {
                    match state {
                        ElementState::Pressed => self
                            .gui
                            .on_input_event(InputEvent::ButtonPressed { button }),
                        ElementState::Released => self
                            .gui
                            .on_input_event(InputEvent::ButtonReleased { button }),
                    }
                }

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        let delta = if self
                            .gui
                            .input_state()
                            .modifiers()
                            .contains(Modifiers::CONTROL)
                        {
                            ScrollDelta::Point(Vec2 {
                                x: y * POINTS_PER_SCROLL_LINE,
                                y: x * POINTS_PER_SCROLL_LINE,
                            })
                        } else {
                            ScrollDelta::Point(Vec2 {
                                x: x * POINTS_PER_SCROLL_LINE,
                                y: y * POINTS_PER_SCROLL_LINE,
                            })
                        };

                        self.gui.on_input_event(InputEvent::Scrolled { delta });
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.gui.on_input_event(InputEvent::Scrolled {
                            delta: ScrollDelta::Pixel(delta.into()),
                        });
                    }
                }

                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::CursorEntered { .. } | WindowEvent::CursorLeft { .. } => {
                self.required_redraws = self.required_redraws.max(2);
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.gui.on_input_event(InputEvent::CursorMoved {
                    position: position.into(),
                });

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
                    self.gui
                        .frame(
                            Vec2 {
                                x: surface.config.width.px(),
                                y: surface.config.height.px(),
                            },
                            |gui| build_gui(&mut self.app_state, gui),
                        )
                        .map_err(|e| format_err!("{e}"))
                        .expect("error building GUI");

                    let mut scene = Scene::new();
                    self.gui.render(&mut scene).unwrap();

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

fn create_theme(theme: &mut Theme) {
    theme.insert_style(
        Theme::UNIVERSAL_CLASS,
        &style! {
            padding: 5.pt(),
            child_spacing: 5.pt(),
            border_color: Color { r: 192, g: 192, b: 192, a: 255 },
            border_width: 1.0.pt(),
            corner_radius: 5.0.pt(),
        },
    );

    theme.insert_style(
        Theme::ROOT_TYPE_CLASS,
        &style! {
            font_size: 16.pt(),
            background: Color { r: 48, g: 48, b: 48, a: 255 },
            border_width: 0.0.pt(),
            corner_radius: 0.0.pt(),
            text_color: Color { r: 224, g: 224, b: 224, a: 255 },
        },
    );

    theme.insert_style(
        Label::TYPE_CLASS,
        &style! {
            border_width: 0.0.pt(),
            corner_radius: 0.0.pt(),
        },
    );

    let button_background: PropertyFn<Brush> = |_, input_state| {
        if input_state.pressed(MouseButtons::PRIMARY) {
            Color::greyscale(96).into()
        } else if input_state.is_hovered() {
            Color::greyscale(80).into()
        } else {
            Color::greyscale(64).into()
        }
    };

    theme.insert_style(
        Button::TYPE_CLASS,
        &style! {
            background: button_background,
        },
    );

    theme.insert_style(
        FlexPanel::TYPE_CLASS,
        &style! {
            width: Sizing::Grow,
            height: Sizing::Grow,
        },
    );

    theme.insert_style(
        ScrollBar::HORIZONTAL_TYPE_CLASS,
        &style! {
            width: Sizing::Grow,
            height: 20.pt(),
            padding: 0.px(),
            child_spacing: 1.pt(),
            border_width: 0.0.px(),
            background: Color { r: 32, g: 32, b: 32, a: 255 },
        },
    );

    theme.insert_style(
        ScrollBar::VERTICAL_TYPE_CLASS,
        &style! {
            width: 20.pt(),
            height: Sizing::Grow,
            padding: 0.px(),
            child_spacing: 1.pt(),
            border_width: 0.0.px(),
            background: Color { r: 32, g: 32, b: 32, a: 255 },
        },
    );

    let scroll_bar_button_style = style! {
        width: 20.pt(),
        height: 20.pt(),
        background: button_background,
    };
    theme.insert_style(ScrollBar::LEFT_BUTTON_CLASS, &scroll_bar_button_style);
    theme.insert_style(ScrollBar::RIGHT_BUTTON_CLASS, &scroll_bar_button_style);
    theme.insert_style(ScrollBar::UP_BUTTON_CLASS, &scroll_bar_button_style);
    theme.insert_style(ScrollBar::DOWN_BUTTON_CLASS, &scroll_bar_button_style);

    theme.insert_style(
        ScrollBar::HORIZONTAL_THUMB_CLASS,
        &style! {
            width: Sizing::Grow,
            height: 20.pt(),
            min_width: 20.pt(),
            max_width: 60.pt(),
            background: button_background,
        },
    );

    theme.insert_style(
        ScrollBar::VERTICAL_THUMB_CLASS,
        &style! {
            width: 20.pt(),
            height: Sizing::Grow,
            min_height: 20.pt(),
            max_height: 60.pt(),
            background: button_background,
        },
    );

    theme.insert_style(
        ScrollView::HORIZONTAL_TYPE_CLASS,
        &style! {
            width: Sizing::Grow,
            max_width: 600.pt(),
            flex_ratio: 2.0,
            child_alignment: Alignment::End,
            cross_axis_alignment: Alignment::Center,
            layout_direction: Direction::LeftToRight,
        },
    );

    theme.insert_style(
        ScrollView::VERTICAL_TYPE_CLASS,
        &style! {
            height: Sizing::Grow,
            max_height: 600.pt(),
            child_alignment: Alignment::End,
            cross_axis_alignment: Alignment::Center,
            layout_direction: Direction::TopToBottom,
        },
    );

    theme.insert_style(
        Popup::TYPE_CLASS,
        &style! {
            background: Color { r: 40, g: 40, b: 40, a: 255 },
            drop_shadow_width: 20.pt(),
            drop_shadow_color: Color { r: 0, g: 0, b: 0, a: 196 },
        },
    );
}

fn build_gui(
    app_state: &mut ExampleAppState,
    mut gui: ByorGuiContext<'_, Scene>,
) -> WidgetResult<()> {
    gui.vertical_scroll_view(|mut gui| {
        for i in 0..5 {
            gui.uid_scope(Uid::new(i), |gui| {
                gui.insert_node(
                    Some(Uid::from_array(b"test")),
                    &style! {
                        width: 100.pt(),
                        height: 100.pt(),
                        border_width: 1.0.pt(),
                        border_color: %inherit,
                        corner_radius: 5.0.pt(),
                    },
                    NodeContents::EMPTY,
                )?;

                Ok(())
            })?;
        }

        Ok(())
    })??;

    gui.horizontal_scroll_view(|mut gui| {
        for _ in 0..5 {
            gui.insert_node(
                None,
                &style! {
                    width: 100.pt(),
                    height: 100.pt(),
                    border_width: 1.0.pt(),
                    border_color: %inherit,
                    corner_radius: 5.0.pt(),
                },
                NodeContents::EMPTY,
            )?;
        }

        Ok(())
    })??;

    let style = style! {
        layout_direction: Direction::TopToBottom,
    };
    let panel = FlexPanel::default().with_style(&style);
    gui.show_container(panel, |mut gui| {
        gui.insert_node(
            None,
            &style! {
                width: Sizing::Grow,
                height: 100.pt(),
            },
            NodeContents::EMPTY,
        )?;

        let style = style! {
            layout_direction: Direction::TopToBottom,
            child_alignment: Alignment::Center,
        };
        let panel = FlexPanel::default().with_style(&style);
        gui.show_container(panel, |mut gui| {
            let style = style! {
                width: Sizing::Grow,
                height: 100.pt(),
                cross_axis_alignment: Alignment::Center,
                horizontal_text_alignment: HorizontalTextAlignment::Center,
                vertical_text_alignment: VerticalTextAlignment::Center,
            };
            let label = Label::default()
                .with_text("Lorem ipsum dolor sit amet")
                .with_style(&style);
            gui.show(label)?;

            gui.insert_node(
                const { Some(Uid::from_slice(b"popup_parent")) },
                &style! {
                    width: 100.pt(),
                    height: 100.pt(),
                    cross_axis_alignment: Alignment::Center,
                    border_width: 1.0.pt(),
                    border_color: %inherit,
                    corner_radius: 5.0.pt(),
                },
                NodeContents::builder(|mut gui| {
                    if gui.parent_input_state().clicked(MouseButtons::SECONDARY) {
                        app_state.show_popup = true;
                    }

                    gui.popup(
                        &mut app_state.show_popup,
                        FloatPosition::CursorFixed,
                        |mut gui| {
                            let style = style! {
                                max_width: 300.px(),
                                padding: 5.pt(),
                                horizontal_text_alignment: HorizontalTextAlignment::Justify,
                            };
                            let label = Label::default()
                                .with_text(include_str!("lorem_ipsum.txt"))
                                .with_style(&style);
                            gui.show(label)?;

                            Ok(())
                        },
                    )?
                    .transpose()?;

                    Ok(())
                }),
            )?
            .result?;

            Ok(())
        })??;

        Ok(())
    })??;

    Ok(())
}
