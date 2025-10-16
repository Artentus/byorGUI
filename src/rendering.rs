use super::*;

pub use parley::Style as TextStyle;
pub use parley::fontique::Synthesis;
pub use parley::{Cluster, Decoration, FontData, Glyph, GlyphRun, Run, RunMetrics};

pub trait Renderer {
    type Error;

    fn push_clip_rect(&mut self, position: Vec2, size: Vec2) -> Result<(), Self::Error>;

    fn pop_clip_rect(&mut self) -> Result<(), Self::Error>;

    fn draw_rect(
        &mut self,
        position: Vec2,
        size: Vec2,
        corner_radius: f32,
        stroke_width: f32,
        color: Color,
    ) -> Result<(), Self::Error>;

    fn fill_rect(
        &mut self,
        position: Vec2,
        size: Vec2,
        corner_radius: f32,
        color: Color,
    ) -> Result<(), Self::Error>;

    fn draw_text(&mut self, text: GlyphRun<'_, Color>, position: Vec2) -> Result<(), Self::Error>;
}

impl ByorGui {
    fn draw_node<R: Renderer>(
        &self,
        node_id: NodeId,
        depth: usize,
        renderer: &mut R,
    ) -> Result<(), R::Error> {
        const LAYER_COLORS: &[Color] = &[
            Color::rgb(10, 110, 137),
            Color::rgb(253, 147, 141),
            Color::rgb(128, 73, 254),
            Color::rgb(254, 216, 77),
        ];

        let node = &self.nodes[node_id];
        renderer.fill_rect(node.position, node.size, 5.0, LAYER_COLORS[depth])?;

        let (clip_position, clip_size) = node.clip_bounds();
        renderer.push_clip_rect(clip_position, clip_size)?;

        if let Some(&text_layout_id) = node.text_layout.as_ref() {
            let text_layout = &self.text_layouts[text_layout_id];

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

        for &child_id in self.children.get(node_id).into_iter().flatten() {
            self.draw_node(child_id, depth + 1, renderer)?;
        }

        renderer.pop_clip_rect()?;
        Ok(())
    }

    pub(crate) fn render_impl<R: Renderer>(
        &mut self,
        root_id: NodeId,
        renderer: &mut R,
    ) -> Result<(), R::Error> {
        self.draw_node(root_id, 0, renderer)
    }
}
