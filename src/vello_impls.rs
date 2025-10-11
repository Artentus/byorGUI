use crate::rendering::*;
use crate::*;
use vello::kurbo::{self, Affine, Line, Rect, Stroke};
use vello::peniko::{self, Fill};

impl From<Position> for kurbo::Point {
    #[inline]
    fn from(value: Position) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Position> for kurbo::Vec2 {
    #[inline]
    fn from(value: Position) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Size> for kurbo::Size {
    #[inline]
    fn from(value: Size) -> Self {
        Self {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

impl From<Size> for kurbo::Vec2 {
    #[inline]
    fn from(value: Size) -> Self {
        Self {
            x: value.width.into(),
            y: value.height.into(),
        }
    }
}

impl Renderer for vello::Scene {
    type Error = std::convert::Infallible;

    fn draw_rect(
        &mut self,
        position: Position,
        size: Size,
        corner_radius: f32,
        stroke_width: f32,
        brush: Brush,
    ) -> Result<(), Self::Error> {
        let rect = Rect::from_origin_size(position, size);
        let brush = match brush {
            Brush::None => return Ok(()),
            Brush::Solid(color) => peniko::Brush::Solid(color),
        };

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
        position: Position,
        size: Size,
        corner_radius: f32,
        brush: Brush,
    ) -> Result<(), Self::Error> {
        let rect = Rect::from_origin_size(position, size);
        let brush = match brush {
            Brush::None => return Ok(()),
            Brush::Solid(color) => peniko::Brush::Solid(color),
        };

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

    fn draw_text(
        &mut self,
        text: GlyphRun<'_, Brush>,
        position: Position,
    ) -> Result<(), Self::Error> {
        let style = text.style();
        let transform = Affine::translate(position);

        'draw_underline: {
            if let Some(underline) = &style.underline {
                let brush = match &underline.brush {
                    Brush::None => break 'draw_underline,
                    &Brush::Solid(color) => peniko::Brush::Solid(color),
                };

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
        }

        'draw_glyphs: {
            let brush = match &style.brush {
                Brush::None => break 'draw_glyphs,
                &Brush::Solid(color) => peniko::Brush::Solid(color),
            };

            let mut x = text.offset();
            let y = text.baseline();
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
                    text.glyphs().map(|glyph| {
                        let gx = x + glyph.x;
                        let gy = y - glyph.y;
                        x += glyph.advance;
                        vello::Glyph {
                            id: glyph.id,
                            x: gx,
                            y: gy,
                        }
                    }),
                );
        }

        'draw_strikethrough: {
            if let Some(strikethrough) = &style.strikethrough {
                let brush = match &strikethrough.brush {
                    Brush::None => break 'draw_strikethrough,
                    &Brush::Solid(color) => peniko::Brush::Solid(color),
                };

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
        }

        Ok(())
    }
}
