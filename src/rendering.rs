use super::*;

pub use parley::Style as TextStyle;
pub use parley::fontique::Synthesis;
pub use parley::{Cluster, Decoration, FontData, Glyph, GlyphRun, Run, RunMetrics};

pub trait Renderer {
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

    fn draw_text(
        &mut self,
        text: GlyphRun<'_, Color>,
        position: Vec2<Pixel>,
    ) -> Result<(), Self::Error>;
}

fn draw_tree<R: Renderer>(
    tree: TreeRef<'_, Node, Shared>,
    data: &ByorGuiData,
    depth: usize,
    renderer: &mut R,
) -> Result<(), R::Error> {
    const LAYER_COLORS: &[Color] = &[
        Color::rgb(10, 110, 137),
        Color::rgb(253, 147, 141),
        Color::rgb(128, 73, 254),
        Color::rgb(254, 216, 77),
    ];

    let TreeRef {
        parent: node,
        descendants,
        ..
    } = tree;

    renderer.fill_rect(
        node.position,
        node.style.fixed_size,
        5.px(),
        LAYER_COLORS[depth],
    )?;

    let (clip_position, clip_size) = node.clip_bounds();
    renderer.push_clip_rect(clip_position, clip_size)?;

    if let Some(&text_layout_id) = node.text_layout.as_ref() {
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

        draw_tree(subtree, data, depth + 1, renderer)?;
    });

    renderer.pop_clip_rect()?;
    Ok(())
}

impl ByorGui {
    pub fn render<R: Renderer>(&mut self, renderer: &mut R) -> Result<(), R::Error> {
        let mut trees = self.forest.trees();
        while let Some(tree) = trees.next() {
            draw_tree(tree, &self.data, 0, renderer)?;
        }

        Ok(())
    }
}
