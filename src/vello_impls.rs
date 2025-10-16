use crate::rendering::*;
use crate::*;
use vello::kurbo::{self, Affine, Line, Rect, Stroke};
use vello::peniko::color::{AlphaColor, Srgb};
use vello::peniko::{self, Fill};

impl From<Vec2> for kurbo::Point {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Vec2> for kurbo::Vec2 {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Vec2> for kurbo::Size {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self {
            width: value.x.into(),
            height: value.y.into(),
        }
    }
}

impl From<Color> for AlphaColor<Srgb> {
    #[inline]
    fn from(value: Color) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}

impl Renderer for vello::Scene {
    type Error = std::convert::Infallible;

    fn push_clip_rect(&mut self, position: Vec2, size: Vec2) -> Result<(), Self::Error> {
        let rect = Rect::from_origin_size(position, size);
        self.push_clip_layer(Affine::IDENTITY, &rect);

        Ok(())
    }

    fn pop_clip_rect(&mut self) -> Result<(), Self::Error> {
        self.pop_layer();

        Ok(())
    }

    fn draw_rect(
        &mut self,
        position: Vec2,
        size: Vec2,
        corner_radius: f32,
        stroke_width: f32,
        color: Color,
    ) -> Result<(), Self::Error> {
        let rect = Rect::from_origin_size(position, size);
        let brush = peniko::Brush::Solid(color.into());

        if corner_radius == 0.0 {
            self.stroke(
                &Stroke::new(stroke_width as f64),
                Affine::IDENTITY,
                &brush,
                None,
                &rect,
            );
        } else {
            self.stroke(
                &Stroke::new(stroke_width as f64),
                Affine::IDENTITY,
                &brush,
                None,
                &rect.to_rounded_rect(corner_radius as f64),
            );
        }

        Ok(())
    }

    fn fill_rect(
        &mut self,
        position: Vec2,
        size: Vec2,
        corner_radius: f32,
        color: Color,
    ) -> Result<(), Self::Error> {
        let rect = Rect::from_origin_size(position, size);
        let brush = peniko::Brush::Solid(color.into());

        if corner_radius == 0.0 {
            self.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &rect);
        } else {
            self.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                None,
                &rect.to_rounded_rect(corner_radius as f64),
            );
        }

        Ok(())
    }

    fn draw_text(&mut self, text: GlyphRun<'_, Color>, position: Vec2) -> Result<(), Self::Error> {
        let style = text.style();
        let transform = Affine::translate(position);

        if let Some(underline) = &style.underline {
            let brush = peniko::Brush::Solid(underline.brush.into());

            let run_metrics = text.run().metrics();
            let offset = match underline.offset {
                Some(offset) => offset,
                None => run_metrics.underline_offset,
            };
            let width = match underline.size {
                Some(size) => size,
                None => run_metrics.underline_size,
            };

            let y = text.baseline() - offset + width / 2.0;

            let line = Line::new(
                (text.offset() as f64, y as f64),
                ((text.offset() + text.advance()) as f64, y as f64),
            );
            self.stroke(&Stroke::new(width.into()), transform, brush, None, &line);
        }

        {
            let brush = peniko::Brush::Solid(style.brush.into());

            let run = text.run();
            let font = run.font();
            let font_size = run.font_size();
            let synthesis = run.synthesis();
            let glyph_xform = synthesis
                .skew()
                .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));

            self.draw_glyphs(font)
                .brush(&brush)
                .hint(true)
                .transform(transform)
                .glyph_transform(glyph_xform)
                .font_size(font_size)
                .normalized_coords(run.normalized_coords())
                .draw(
                    Fill::NonZero,
                    text.positioned_glyphs().map(|glyph| vello::Glyph {
                        id: glyph.id,
                        x: glyph.x,
                        y: glyph.y,
                    }),
                );
        }

        if let Some(strikethrough) = &style.strikethrough {
            let brush = peniko::Brush::Solid(strikethrough.brush.into());

            let run_metrics = text.run().metrics();
            let offset = match strikethrough.offset {
                Some(offset) => offset,
                None => run_metrics.strikethrough_offset,
            };
            let width = match strikethrough.size {
                Some(size) => size,
                None => run_metrics.strikethrough_size,
            };

            let y = text.baseline() - offset + run_metrics.strikethrough_size / 2.0;

            let line = Line::new(
                (text.offset() as f64, y as f64),
                ((text.offset() + text.advance()) as f64, y as f64),
            );
            self.stroke(&Stroke::new(width.into()), transform, brush, None, &line);
        }

        Ok(())
    }
}
