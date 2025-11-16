use super::*;
use crate::style::axis::*;
use crate::theme::StyleClass;
use crate::*;
use std::marker::PhantomData;

const SCROLL_BAR_UID: Uid = Uid::from_array(b"##scroll_bar");
const SCROLL_BAR_DEC_BUTTON_UID: Uid = Uid::from_array(b"##scroll_bar_dec_button");
const SCROLL_BAR_INC_BUTTON_UID: Uid = Uid::from_array(b"##scroll_bar_inc_button");
const SCROLL_BAR_THUMB_UID: Uid = Uid::from_array(b"##scroll_bar_thumb");

pub struct ScrollBarData {
    axis: Axis,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    thumb_size_ratio: Option<f32>,
}

pub type ScrollBar<'style, 'classes> = Widget<'style, 'classes, ScrollBarData>;

impl ScrollBar<'_, '_> {
    pub const HORIZONTAL_TYPE_CLASS: StyleClass =
        StyleClass::new_static("###horizontal_scroll_bar");
    pub const VERTICAL_TYPE_CLASS: StyleClass = StyleClass::new_static("###vertical_scroll_bar");

    pub const LEFT_BUTTON_CLASS: StyleClass = StyleClass::new_static("###scroll_bar_left_button");
    pub const RIGHT_BUTTON_CLASS: StyleClass = StyleClass::new_static("###scroll_bar_right_button");
    pub const UP_BUTTON_CLASS: StyleClass = StyleClass::new_static("###scroll_bar_up_button");
    pub const DOWN_BUTTON_CLASS: StyleClass = StyleClass::new_static("###scroll_bar_down_button");

    pub const HORIZONTAL_THUMB_CLASS: StyleClass =
        StyleClass::new_static("###horizontal_scroll_bar_thumb");
    pub const VERTICAL_THUMB_CLASS: StyleClass =
        StyleClass::new_static("###vertical_scroll_bar_thumb");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(axis: Axis) -> Self {
        ScrollBarData {
            axis,
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: None,
            thumb_size_ratio: None,
        }
        .into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn horizontal() -> Self {
        Self::new(Axis::X)
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn vertical() -> Self {
        Self::new(Axis::Y)
    }

    #[must_use]
    #[inline]
    pub fn value(&self) -> f32 {
        self.data().value
    }

    #[must_use]
    #[inline]
    pub fn with_value(self, value: f32) -> Self {
        self.map_data(|data| ScrollBarData { value, ..data })
    }

    #[must_use]
    #[inline]
    pub fn min(&self) -> f32 {
        self.data().min
    }

    #[must_use]
    #[inline]
    pub fn with_min(self, min: f32) -> Self {
        self.map_data(|data| ScrollBarData { min, ..data })
    }

    #[must_use]
    #[inline]
    pub fn max(&self) -> f32 {
        self.data().max
    }

    #[must_use]
    #[inline]
    pub fn with_max(self, max: f32) -> Self {
        self.map_data(|data| ScrollBarData { max, ..data })
    }

    #[must_use]
    #[inline]
    pub fn step(&self) -> Option<f32> {
        self.data().step
    }

    #[must_use]
    #[inline]
    pub fn with_step(self, step: f32) -> Self {
        self.map_data(|data| ScrollBarData {
            step: Some(step),
            ..data
        })
    }

    #[must_use]
    #[inline]
    pub fn thumb_size_ratio(&self) -> Option<f32> {
        self.data().thumb_size_ratio
    }

    #[must_use]
    #[inline]
    pub fn with_thumb_size_ratio(self, thumb_size_ratio: f32) -> Self {
        self.map_data(|data| ScrollBarData {
            thumb_size_ratio: Some(thumb_size_ratio.clamp(0.0, 1.0)),
            ..data
        })
    }
}

impl WidgetData for ScrollBarData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        match self.axis {
            Axis::X => ScrollBar::HORIZONTAL_TYPE_CLASS,
            Axis::Y => ScrollBar::VERTICAL_TYPE_CLASS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ButtonDirection {
    Left,
    Right,
    Up,
    Down,
}

impl ButtonDirection {
    #[must_use]
    #[inline]
    const fn dec_along_axis(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::Left,
            Axis::Y => Self::Up,
        }
    }

    #[must_use]
    #[inline]
    const fn inc_along_axis(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::Right,
            Axis::Y => Self::Down,
        }
    }
}

struct ScrollBarButtonRenderer<Renderer: rendering::Renderer> {
    direction: ButtonDirection,
    _renderer: PhantomData<fn(Renderer)>,
}

impl<Renderer: rendering::Renderer> ScrollBarButtonRenderer<Renderer> {
    #[must_use]
    #[inline]
    const fn new(direction: ButtonDirection) -> Self {
        Self {
            direction,
            _renderer: PhantomData,
        }
    }
}

impl<Renderer: rendering::Renderer> rendering::NodeRenderer for ScrollBarButtonRenderer<Renderer> {
    type Renderer = Renderer;

    fn render(
        &self,
        context: rendering::RenderContext<'_, Self::Renderer>,
    ) -> Result<(), <Self::Renderer as rendering::Renderer>::Error> {
        let size = context.size.x.min(context.size.y);
        let arrow_radius = size / 4.0;
        let arrow_center = context.position + context.size / 2.0;

        let mut vertices = match self.direction {
            ButtonDirection::Left => [
                Vec2 {
                    x: 0.4.px(),
                    y: -0.9.px(),
                },
                Vec2 {
                    x: 0.4.px(),
                    y: 0.9.px(),
                },
                Vec2 {
                    x: -0.6.px(),
                    y: 0.px(),
                },
            ],
            ButtonDirection::Right => [
                Vec2 {
                    x: -0.4.px(),
                    y: -0.9.px(),
                },
                Vec2 {
                    x: -0.4.px(),
                    y: 0.9.px(),
                },
                Vec2 {
                    x: 0.6.px(),
                    y: 0.px(),
                },
            ],
            ButtonDirection::Up => [
                Vec2 {
                    x: -0.9.px(),
                    y: 0.4.px(),
                },
                Vec2 {
                    x: 0.9.px(),
                    y: 0.4.px(),
                },
                Vec2 {
                    x: 0.px(),
                    y: -0.6.px(),
                },
            ],
            ButtonDirection::Down => [
                Vec2 {
                    x: -0.9.px(),
                    y: -0.4.px(),
                },
                Vec2 {
                    x: 0.9.px(),
                    y: -0.4.px(),
                },
                Vec2 {
                    x: 0.px(),
                    y: 0.6.px(),
                },
            ],
        };

        for vertex in vertices.iter_mut() {
            *vertex *= arrow_radius.value();
            *vertex += arrow_center;
        }

        context
            .renderer
            .fill_poly(&vertices, context.style.text_color().into())
    }
}

fn scroll_bar_button<Renderer: rendering::Renderer>(
    gui: &mut ByorGuiContext<'_, Renderer>,
    uid: Uid,
    direction: ButtonDirection,
) -> WidgetResult<NodeInputState> {
    let class = match direction {
        ButtonDirection::Left => &[ScrollBar::LEFT_BUTTON_CLASS],
        ButtonDirection::Right => &[ScrollBar::RIGHT_BUTTON_CLASS],
        ButtonDirection::Up => &[ScrollBar::UP_BUTTON_CLASS],
        ButtonDirection::Down => &[ScrollBar::DOWN_BUTTON_CLASS],
    };

    let renderer = ScrollBarButtonRenderer::new(direction);
    let button = CanvasButton::new(renderer)
        .with_uid(uid)
        .with_classes(class);

    gui.show(button)
}

fn scroll_bar_thumb<Renderer: rendering::Renderer>(
    gui: &mut ByorGuiContext<'_, Renderer>,
    uid: Uid,
    axis: Axis,
    size_ratio: Option<f32>,
) -> WidgetResult<NodeInputState> {
    let class = match axis {
        Axis::X => &[ScrollBar::HORIZONTAL_THUMB_CLASS],
        Axis::Y => &[ScrollBar::VERTICAL_THUMB_CLASS],
    };

    let mut thumb = Button::default().with_uid(uid).with_classes(class);

    let mut style = Style::default();
    if let Some(size_ratio) = size_ratio {
        style = style
            .with_flex_ratio(size_ratio)
            .with_size_along_axis(axis, Sizing::Grow)
            .with_max_size_along_axis(axis, INITIAL_MAX_SIZE);
        thumb = thumb.with_style(&style);
    }

    gui.show(thumb)
}

impl<Renderer: rendering::Renderer> LeafWidgetData<Renderer> for ScrollBarData {
    type ShowResult = f32;

    fn show(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        let style = style.with_layout_direction(self.axis.primary_direction());

        let step = self.step.unwrap_or((self.max - self.min) * 0.1);
        let mut value = self.value.clamp(self.min, self.max);
        let mut factor = (value - self.min) / (self.max - self.min);
        let mut opposite_factor = 1.0 - factor;

        if let Some(thumb_size_ratio) = self.thumb_size_ratio {
            factor *= 1.0 - thumb_size_ratio;
            opposite_factor *= 1.0 - thumb_size_ratio;
        }

        let leading_space_style = style! {
            width: Sizing::Grow,
            height: Sizing::Grow,
            flex_ratio: factor,
        };

        let trailing_space_style = style! {
            width: Sizing::Grow,
            height: Sizing::Grow,
            flex_ratio: opposite_factor,
        };

        let uid = uid.produce();
        let dec_button_uid = uid.concat(SCROLL_BAR_DEC_BUTTON_UID);
        let inc_button_uid = uid.concat(SCROLL_BAR_INC_BUTTON_UID);
        let thumb_uid = uid.concat(SCROLL_BAR_THUMB_UID);

        gui.insert_node(
            Some(uid),
            &style,
            NodeContents::builder(|mut gui| {
                let dec_button_response = scroll_bar_button(
                    &mut gui,
                    dec_button_uid,
                    ButtonDirection::dec_along_axis(self.axis),
                )?;
                if dec_button_response.clicked(MouseButtons::PRIMARY) {
                    value -= step;
                }

                gui.insert_node(None, &leading_space_style, NodeContents::EMPTY)?;

                let thumb_response =
                    scroll_bar_thumb(&mut gui, thumb_uid, self.axis, self.thumb_size_ratio)?;
                if thumb_response.clicked(MouseButtons::PRIMARY) {
                    let thumb_pos = gui
                        .previous_state(thumb_uid)
                        .map(|state| state.position.along_axis(self.axis))
                        .unwrap_or_default();
                    let thumb_offset = gui
                        .global_input_state()
                        .mouse_position()
                        .along_axis(self.axis)
                        - thumb_pos;

                    gui.persistent_state_mut(uid)
                        .insert(PersistentStateKey::ScrollBarThumbMouseOffset, thumb_offset);
                } else if thumb_response.pressed(MouseButtons::PRIMARY) {
                    let (scroll_bar_pos, scroll_bar_size) = gui
                        .previous_state(uid)
                        .map(|state| {
                            (
                                state.position.along_axis(self.axis),
                                state.size.along_axis(self.axis),
                            )
                        })
                        .unwrap_or_default();
                    let left_button_size = gui
                        .previous_state(dec_button_uid)
                        .map(|state| state.size.along_axis(self.axis))
                        .unwrap_or_default();
                    let right_button_size = gui
                        .previous_state(inc_button_uid)
                        .map(|state| state.size.along_axis(self.axis))
                        .unwrap_or_default();
                    let thumb_size = gui
                        .previous_state(thumb_uid)
                        .map(|state| state.size.along_axis(self.axis))
                        .unwrap_or_default();
                    let thumb_mouse_offset: Float<Pixel> = gui
                        .persistent_state(uid)
                        .get(PersistentStateKey::ScrollBarThumbMouseOffset)
                        .copied()
                        .unwrap_or(thumb_size / 2.0);

                    let parent_style = gui.computed_parent_style();
                    let padding = parent_style.padding().along_axis(self.axis);
                    let spacing = parent_style.child_spacing();

                    let scroll_space = scroll_bar_size
                        - left_button_size
                        - right_button_size
                        - thumb_size
                        - padding[0]
                        - padding[1]
                        - spacing * 4.0;

                    let scroll_position = gui
                        .global_input_state()
                        .mouse_position()
                        .along_axis(self.axis)
                        - scroll_bar_pos
                        - left_button_size
                        - thumb_mouse_offset
                        - padding[0]
                        - spacing * 2.0;

                    value = (scroll_position / scroll_space) * (self.max - self.min);
                }

                gui.insert_node(None, &trailing_space_style, NodeContents::EMPTY)?;

                let inc_button_response = scroll_bar_button(
                    &mut gui,
                    inc_button_uid,
                    ButtonDirection::inc_along_axis(self.axis),
                )?;
                if inc_button_response.clicked(MouseButtons::PRIMARY) {
                    value += step;
                }

                Ok(value.clamp(self.min, self.max))
            }),
        )?
        .result
    }
}

pub struct ScrollViewData {
    axis: Axis,
}

pub type ScrollView<'style, 'classes> = Widget<'style, 'classes, ScrollViewData>;

impl ScrollView<'_, '_> {
    pub const HORIZONTAL_TYPE_CLASS: StyleClass =
        StyleClass::new_static("###horizontal_scroll_view");
    pub const VERTICAL_TYPE_CLASS: StyleClass = StyleClass::new_static("###vertical_scroll_view");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(axis: Axis) -> Self {
        ScrollViewData { axis }.into()
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn horizontal() -> Self {
        Self::new(Axis::X)
    }

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn vertical() -> Self {
        Self::new(Axis::Y)
    }
}

impl WidgetData for ScrollViewData {
    #[inline]
    fn type_class(&self) -> StyleClass {
        match self.axis {
            Axis::X => ScrollView::HORIZONTAL_TYPE_CLASS,
            Axis::Y => ScrollView::VERTICAL_TYPE_CLASS,
        }
    }
}

impl<Renderer: rendering::Renderer> ContainerWidgetData<Renderer> for ScrollViewData {
    type ShowResult<T> = T;

    fn show<R>(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
        contents: impl FnOnce(ByorGuiContext<'_, Renderer>) -> R,
    ) -> WidgetResult<Self::ShowResult<R>> {
        let uid = uid.produce();

        let parent_style = gui.parent_style().clone();

        // Obtain a sensible spacing from the scrollbar spacing since we do not
        // want to use the spacing of the container
        let scroll_bar_type_class = match self.axis {
            Axis::X => ScrollBar::HORIZONTAL_TYPE_CLASS,
            Axis::Y => ScrollBar::VERTICAL_TYPE_CLASS,
        };
        let scroll_bar_child_spacing = gui
            .theme()
            .build_style_property(
                |style| &style.child_spacing,
                None,
                &[],
                scroll_bar_type_class,
            )
            .cascade(
                &parent_style.child_spacing,
                &parent_style,
                gui.parent_input_state(),
            )
            .unwrap_or(INITIAL_CHILD_SPACING);

        let scroll_view_style = style
            .clone()
            .with_layout_direction(self.axis.cross_direction())
            .with_initial_child_alignment()
            .with_child_spacing(scroll_bar_child_spacing * 2.0);

        let scroll_bar_style = Style::default().with_size_along_axis(self.axis, Sizing::Grow);

        gui.insert_node(
            None,
            &scroll_view_style,
            NodeContents::builder(|mut gui| {
                let cascaded_style = style.cascade(&parent_style, gui.parent_input_state());
                let scroll_container_style = cascaded_style
                    .as_style()
                    .with_width(Sizing::Grow)
                    .with_height(Sizing::Grow)
                    .with_initial_min_width()
                    .with_initial_min_height()
                    .with_initial_max_width()
                    .with_initial_max_height()
                    .with_initial_flex_ratio()
                    .with_padding(Padding::ZERO)
                    .with_initial_cross_axis_alignment()
                    .with_border_width(0.0.px())
                    .with_background(Color::TRANSPARENT);

                let mut scroll: Float<Pixel> = gui
                    .persistent_state(uid)
                    .get(self.axis.persistent_state_scroll_key())
                    .copied()
                    .unwrap_or_default();
                let mut thumb_size_ratio = 0.5;
                let mut max_scroll = 0.px();

                let response = gui.insert_node(
                    Some(uid),
                    &scroll_container_style,
                    NodeContents::builder(|gui| {
                        if let Some(previous_state) = gui.previous_state(uid) {
                            let padding =
                                gui.computed_parent_style().padding().along_axis(self.axis);
                            let container_size =
                                previous_state.size.along_axis(self.axis) - padding[0] - padding[1];
                            let content_size = previous_state.content_size.along_axis(self.axis);
                            let available_size = container_size - content_size;
                            thumb_size_ratio = container_size / content_size;
                            max_scroll = (-available_size).max(0.px());
                        }

                        contents(gui)
                    }),
                )?;

                if max_scroll > 0.px() {
                    if response.is_hovered() {
                        // Scroll is subtractive in layouting, so we need to subtract here as well
                        scroll -= gui
                            .global_input_state()
                            .scroll_delta()
                            .along_axis(self.axis);
                    }

                    let scroll_bar = ScrollBar::new(self.axis)
                        .with_uid(uid.concat(SCROLL_BAR_UID))
                        .with_value(scroll.value())
                        .with_min(0.0)
                        .with_max(max_scroll.value())
                        .with_step((POINTS_PER_SCROLL_LINE * gui.scale_factor()).value())
                        .with_thumb_size_ratio(thumb_size_ratio)
                        .with_style(&scroll_bar_style);
                    scroll = gui.show(scroll_bar)?.px();
                }

                gui.persistent_state_mut(uid)
                    .insert(self.axis.persistent_state_scroll_key(), scroll);

                Ok(response.result)
            }),
        )?
        .result
    }
}
