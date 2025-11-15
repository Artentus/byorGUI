use super::*;

pub use parley::Style as TextStyle;
pub use parley::fontique::Synthesis;
pub use parley::{Cluster, Decoration, FontData, Glyph, GlyphRun, Run, RunMetrics};

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
        color: Color,
    ) -> Result<(), Self::Error>;

    fn draw_poly(
        &mut self,
        vertices: &[Vec2<Pixel>],
        stroke_width: Float<Pixel>,
        color: Color,
    ) -> Result<(), Self::Error>;

    fn fill_poly(&mut self, vertices: &[Vec2<Pixel>], color: Color) -> Result<(), Self::Error>;

    fn draw_text(
        &mut self,
        text: GlyphRun<'_, Color>,
        position: Vec2<Pixel>,
    ) -> Result<(), Self::Error>;
}

pub struct RenderContext<'a, R: Renderer> {
    pub position: Vec2<Pixel>,
    pub size: Vec2<Pixel>,
    pub style: &'a ComputedStyle,
    pub persistent_state: Option<&'a PersistentState>,
    pub renderer: &'a mut R,
}

pub trait NodeRenderer: 'static {
    type Renderer: Renderer;

    fn render(
        &self,
        context: RenderContext<'_, Self::Renderer>,
    ) -> Result<(), <Self::Renderer as Renderer>::Error>;
}

fn draw_tree<R: Renderer>(
    tree: TreeRef<'_, Node, Shared>,
    data: &ByorGuiData<R>,
    renderer: &mut R,
) -> Result<(), R::Error> {
    let TreeRef {
        parent: node,
        descendants,
        ..
    } = tree;

    renderer.fill_rect(
        node.position,
        node.style.fixed_size,
        node.style.corner_radius(),
        node.style.background(),
    )?;

    renderer.draw_rect(
        node.position + node.style.border_width() * 0.5,
        node.style.fixed_size - node.style.border_width(),
        node.style.corner_radius(),
        node.style.border_width(),
        node.style.border_color(),
    )?;

    let (clip_position, clip_size) = node.clip_bounds();
    renderer.push_clip_rect(clip_position, clip_size)?;

    if let Some(node_renderer_id) = node.renderer.expand() {
        let context = RenderContext {
            position: node.position,
            size: node.style.fixed_size,
            style: &node.style,
            persistent_state: node.uid.and_then(|uid| data.persistent_state.get(uid)),
            renderer,
        };

        data.renderers[node_renderer_id].render(context)?;
    }

    if let Some(text_layout_id) = node.text_layout.expand() {
        let text_layout = &data.text_layouts[text_layout_id];

        for line in text_layout.lines() {
            for item in line.items() {
                match item {
                    parley::PositionedLayoutItem::GlyphRun(text) => {
                        let text_position = Vec2 {
                            x: node.position.x + node.style.padding().left,
                            y: node.position.y
                                + node.style.padding().top
                                + node.vertical_text_offset,
                        };
                        renderer.draw_text(text, text_position)?
                    }
                    parley::PositionedLayoutItem::InlineBox(_) => {
                        unreachable!("inline boxes are not generated")
                    }
                }
            }
        }
    }

    iter_subtrees!(descendants => |subtree| {
        if subtree.is_root {
            continue;
        }

        draw_tree(subtree, data, renderer)?;
    });

    renderer.pop_clip_rect()?;
    Ok(())
}

impl<R: Renderer> ByorGui<R> {
    pub fn render(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let mut trees = self.forest.trees();
        while let Some(tree) = trees.next() {
            draw_tree(tree, &self.data, renderer)?;
        }

        Ok(())
    }
}
