use super::*;
use std::marker::PhantomData;

pub trait InlineBoxRenderer {
    type Renderer: Renderer + ?Sized;

    fn render_box(
        &mut self,
        renderer: &mut Self::Renderer,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
        id: u64,
    ) -> Result<(), <Self::Renderer as Renderer>::Error>;
}

pub struct UnimplementedBoxRenderer<R: Renderer> {
    _r: PhantomData<fn(R)>,
}

impl<R: Renderer> Default for UnimplementedBoxRenderer<R> {
    #[inline]
    fn default() -> Self {
        Self { _r: PhantomData }
    }
}

impl<R: Renderer> InlineBoxRenderer for UnimplementedBoxRenderer<R> {
    type Renderer = R;

    #[inline]
    fn render_box(
        &mut self,
        _renderer: &mut Self::Renderer,
        _position: Vec2<Pixel>,
        _size: Vec2<Pixel>,
        _id: u64,
    ) -> Result<(), <Self::Renderer as Renderer>::Error> {
        unimplemented!()
    }
}

pub trait Renderer: 'static {
    type Error;

    fn push_clip_rect(
        &mut self,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
    ) -> Result<(), Self::Error>;

    fn pop_clip_rect(&mut self) -> Result<(), Self::Error>;

    fn draw_rect(
        &mut self,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
        corner_radius: Float<Pixel>,
        stroke_width: Float<Pixel>,
        color: Color,
    ) -> Result<(), Self::Error>;

    fn fill_rect(
        &mut self,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
        corner_radius: Float<Pixel>,
        brush: ComputedBrush,
    ) -> Result<(), Self::Error>;

    fn draw_poly(
        &mut self,
        vertices: &[Vec2<Pixel>],
        stroke_width: Float<Pixel>,
        color: Color,
    ) -> Result<(), Self::Error>;

    fn fill_poly(
        &mut self,
        vertices: &[Vec2<Pixel>],
        brush: ComputedBrush,
    ) -> Result<(), Self::Error>;

    fn draw_text(
        &mut self,
        text: parley::GlyphRun<'_, Color>,
        position: Vec2<Pixel>,
    ) -> Result<(), Self::Error>;

    fn draw_text_layout<B>(
        &mut self,
        layout: &parley::Layout<Color>,
        position: Vec2<Pixel>,
        box_renderer: &mut B,
    ) -> Result<(), Self::Error>
    where
        B: InlineBoxRenderer<Renderer = Self>,
    {
        for line in layout.lines() {
            for item in line.items() {
                match item {
                    parley::PositionedLayoutItem::GlyphRun(text) => {
                        self.draw_text(text, position)?;
                    }
                    parley::PositionedLayoutItem::InlineBox(b) => {
                        let box_position = Vec2 {
                            x: position.x + b.x.px(),
                            y: position.y + b.y.px(),
                        };
                        let box_size = Vec2 {
                            x: b.width.px(),
                            y: b.height.px(),
                        };
                        box_renderer.render_box(self, box_position, box_size, b.id)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct RenderContext<'a, R: Renderer> {
    pub position: Vec2<Pixel>,
    pub size: Vec2<Pixel>,
    pub style: &'a ComputedStyle,
    pub scale_factor: f32,
    pub input_state: NodeInputState,
    pub persistent_state: &'a PersistentState,
    pub renderer: &'a mut R,
}

pub trait NodeRenderer: Send + 'static {
    type Renderer: Renderer;

    fn render(
        &self,
        context: RenderContext<'_, Self::Renderer>,
    ) -> Result<(), <Self::Renderer as Renderer>::Error>;
}

fn draw_drop_shadow<R: Renderer>(node: &Node, renderer: &mut R) -> Result<(), R::Error> {
    const STOP_COUNT: usize = 8;

    let edge_stops = std::array::from_fn::<_, STOP_COUNT, _>(|i| {
        let offset = (i as f32) / (STOP_COUNT as f32);
        let mut color = node.style.drop_shadow_color();
        let t = 1.0 - offset;
        color.a = ((color.a as f32) * t * t * t).round() as u8;

        GradientStop { offset, color }
    });

    renderer.fill_rect(
        Vec2 {
            x: node.position.x - node.style.drop_shadow_width(),
            y: node.position.y + node.style.corner_radius(),
        },
        Vec2 {
            x: node.style.drop_shadow_width(),
            y: node.style.fixed_size.y - 2.0 * node.style.corner_radius(),
        },
        0.px(),
        ComputedBrush::LinearGradient {
            start: Vec2 {
                x: node.position.x,
                y: 0.px(),
            },
            end: Vec2 {
                x: node.position.x - node.style.drop_shadow_width(),
                y: 0.px(),
            },
            stops: &edge_stops,
        },
    )?;

    renderer.fill_rect(
        Vec2 {
            x: node.position.x + node.style.fixed_size.x,
            y: node.position.y + node.style.corner_radius(),
        },
        Vec2 {
            x: node.style.drop_shadow_width(),
            y: node.style.fixed_size.y - 2.0 * node.style.corner_radius(),
        },
        0.px(),
        ComputedBrush::LinearGradient {
            start: Vec2 {
                x: node.position.x + node.style.fixed_size.x,
                y: 0.px(),
            },
            end: Vec2 {
                x: node.position.x + node.style.fixed_size.x + node.style.drop_shadow_width(),
                y: 0.px(),
            },
            stops: &edge_stops,
        },
    )?;

    renderer.fill_rect(
        Vec2 {
            x: node.position.x + node.style.corner_radius(),
            y: node.position.y - node.style.drop_shadow_width(),
        },
        Vec2 {
            x: node.style.fixed_size.x - 2.0 * node.style.corner_radius(),
            y: node.style.drop_shadow_width(),
        },
        0.px(),
        ComputedBrush::LinearGradient {
            start: Vec2 {
                x: 0.px(),
                y: node.position.y,
            },
            end: Vec2 {
                x: 0.px(),
                y: node.position.y - node.style.drop_shadow_width(),
            },
            stops: &edge_stops,
        },
    )?;

    renderer.fill_rect(
        Vec2 {
            x: node.position.x + node.style.corner_radius(),
            y: node.position.y + node.style.fixed_size.y,
        },
        Vec2 {
            x: node.style.fixed_size.x - 2.0 * node.style.corner_radius(),
            y: node.style.drop_shadow_width(),
        },
        0.px(),
        ComputedBrush::LinearGradient {
            start: Vec2 {
                x: 0.px(),
                y: node.position.y + node.style.fixed_size.y,
            },
            end: Vec2 {
                x: 0.px(),
                y: node.position.y + node.style.fixed_size.y + node.style.drop_shadow_width(),
            },
            stops: &edge_stops,
        },
    )?;

    let corner_size = node.style.drop_shadow_width() + node.style.corner_radius();
    let corner_offset = node.style.corner_radius() / corner_size;
    let offset_range = 1.0 - corner_offset;

    let corner_stops = std::array::from_fn::<_, { STOP_COUNT + 2 }, _>(|i| match i {
        0 => GradientStop {
            offset: 0.0,
            color: Color::TRANSPARENT,
        },
        1 => GradientStop {
            offset: corner_offset - 0.00001,
            color: Color::TRANSPARENT,
        },
        _ => {
            let mut stop = edge_stops[i - 2];
            stop.offset = stop.offset * offset_range + corner_offset;
            stop
        }
    });

    renderer.fill_rect(
        node.position - node.style.drop_shadow_width(),
        corner_size.into(),
        0.px(),
        ComputedBrush::RadialGradient {
            center: node.position + node.style.corner_radius(),
            radius: corner_size.into(),
            stops: &corner_stops,
        },
    )?;

    renderer.fill_rect(
        Vec2 {
            x: node.position.x + node.style.fixed_size.x - node.style.corner_radius(),
            y: node.position.y - node.style.drop_shadow_width(),
        },
        corner_size.into(),
        0.px(),
        ComputedBrush::RadialGradient {
            center: Vec2 {
                x: node.position.x + node.style.fixed_size.x - node.style.corner_radius(),
                y: node.position.y + node.style.corner_radius(),
            },
            radius: corner_size.into(),
            stops: &corner_stops,
        },
    )?;

    renderer.fill_rect(
        Vec2 {
            x: node.position.x - node.style.drop_shadow_width(),
            y: node.position.y + node.style.fixed_size.y - node.style.corner_radius(),
        },
        corner_size.into(),
        0.px(),
        ComputedBrush::RadialGradient {
            center: Vec2 {
                x: node.position.x + node.style.corner_radius(),
                y: node.position.y + node.style.fixed_size.y - node.style.corner_radius(),
            },
            radius: corner_size.into(),
            stops: &corner_stops,
        },
    )?;

    renderer.fill_rect(
        node.position + node.style.fixed_size - node.style.corner_radius(),
        corner_size.into(),
        0.px(),
        ComputedBrush::RadialGradient {
            center: node.position + node.style.fixed_size - node.style.corner_radius(),
            radius: corner_size.into(),
            stops: &corner_stops,
        },
    )?;

    Ok(())
}

fn draw_tree<R: Renderer>(
    tree: TreeRef<'_, Node, Shared>,
    data: &ByorGuiData<R>,
    scale_factor: f32,
    renderer: &mut R,
) -> Result<(), R::Error> {
    let TreeRef {
        parent: node,
        descendants,
        ..
    } = tree;

    if node.style.drop_shadow_width() > 0.px() {
        draw_drop_shadow(node, renderer)?;
    }

    renderer.fill_rect(
        node.position,
        node.style.fixed_size,
        node.style.corner_radius(),
        node.style.background().offset(node.position),
    )?;

    if node.style.border_width() > 0.px() {
        renderer.draw_rect(
            node.position + node.style.border_width() * 0.5,
            node.style.fixed_size - node.style.border_width(),
            node.style.corner_radius(),
            node.style.border_width(),
            node.style.border_color(),
        )?;
    }

    let (clip_position, clip_size) = node.clip_bounds();
    renderer.push_clip_rect(clip_position, clip_size)?;

    if let Some(node_renderer_id) = node.renderer.expand() {
        let persistent_state = node
            .uid
            .and_then(|uid| data.persistent_state.get(uid))
            .unwrap_or(&PersistentState::EMPTY);

        let context = RenderContext {
            position: node.position,
            size: node.style.fixed_size,
            style: &node.style,
            scale_factor,
            input_state: data.compute_node_input_state(node.uid),
            persistent_state,
            renderer,
        };

        data.renderers[node_renderer_id].render(context)?;
    }

    if let Some(text_layout_id) = node.text_layout.expand() {
        let text_layout = &data.text_layouts[text_layout_id];
        let text_position = Vec2 {
            x: node.position.x + node.style.padding().left,
            y: node.position.y + node.style.padding().top + node.vertical_text_offset,
        };

        renderer.draw_text_layout(
            text_layout,
            text_position,
            &mut UnimplementedBoxRenderer::default(),
        )?;
    }

    iter_subtrees!(descendants => |subtree| {
        if subtree.is_root {
            continue;
        }

        draw_tree(subtree, data, scale_factor, renderer)?;
    });

    renderer.pop_clip_rect()?;
    Ok(())
}

impl<R: Renderer> ByorGui<R> {
    pub fn render(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let mut trees = self.forest.trees();
        while let Some(tree) = trees.next() {
            draw_tree(tree, &self.data, self.scale_factor(), renderer)?;
        }

        Ok(())
    }
}
