use crate::rendering::*;
use crate::*;
use vello::kurbo::{self, Affine, Line, PathEl, Rect, Shape, Stroke};
use vello::peniko::color::{AlphaColor, DynamicColor, Srgb};
use vello::peniko::{self, Fill, Gradient};

impl From<Vec2<Pixel>> for kurbo::Point {
    #[inline]
    fn from(value: Vec2<Pixel>) -> Self {
        Self {
            x: value.x.value().into(),
            y: value.y.value().into(),
        }
    }
}

impl From<Vec2<Pixel>> for kurbo::Vec2 {
    #[inline]
    fn from(value: Vec2<Pixel>) -> Self {
        Self {
            x: value.x.value().into(),
            y: value.y.value().into(),
        }
    }
}

impl From<Vec2<Pixel>> for kurbo::Size {
    #[inline]
    fn from(value: Vec2<Pixel>) -> Self {
        Self {
            width: value.x.value().into(),
            height: value.y.value().into(),
        }
    }
}

impl From<Color> for AlphaColor<Srgb> {
    #[inline]
    fn from(color: Color) -> Self {
        Self::from_rgba8(color.r, color.g, color.b, color.a)
    }
}

impl From<Color> for DynamicColor {
    #[inline]
    fn from(color: Color) -> Self {
        Self::from_alpha_color::<Srgb>(color.into())
    }
}

impl From<GradientStop> for peniko::ColorStop {
    #[inline]
    fn from(stop: GradientStop) -> Self {
        peniko::ColorStop {
            offset: stop.offset,
            color: stop.color.into(),
        }
    }
}

fn convert_brush(brush: ComputedBrush) -> (peniko::Brush, Option<Affine>) {
    match brush {
        ComputedBrush::Solid(color) => (peniko::Brush::Solid(color.into()), None),
        ComputedBrush::LinearGradient { start, end, stops } => (
            peniko::Brush::Gradient(Gradient::new_linear(start, end).with_stops(stops)),
            None,
        ),
        ComputedBrush::RadialGradient {
            center,
            radius,
            stops,
        } => {
            let gradient = Gradient::new_radial(kurbo::Point::ZERO, 1.0).with_stops(stops);
            let transform =
                Affine::scale_non_uniform(radius.x.value() as f64, radius.y.value() as f64)
                    .then_translate((center).into());

            (peniko::Brush::Gradient(gradient), Some(transform))
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Polygon<'a> {
    vertices: &'a [Vec2<Pixel>],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PolygonIterState {
    Open,
    Vertex,
    Finished,
}

struct PolygonIter<'a> {
    state: PolygonIterState,
    vertices: &'a [Vec2<Pixel>],
}

impl Iterator for PolygonIter<'_> {
    type Item = PathEl;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == PolygonIterState::Finished {
            None
        } else if let Some((&vertex, remaining)) = self.vertices.split_first() {
            self.vertices = remaining;

            if self.state == PolygonIterState::Open {
                self.state = PolygonIterState::Vertex;
                Some(PathEl::MoveTo(vertex.into()))
            } else {
                Some(PathEl::LineTo(vertex.into()))
            }
        } else {
            self.state = PolygonIterState::Finished;
            Some(PathEl::ClosePath)
        }
    }
}

impl Shape for Polygon<'_> {
    type PathElementsIter<'iter>
        = PolygonIter<'iter>
    where
        Self: 'iter;

    fn path_elements(&self, _tolerance: f64) -> Self::PathElementsIter<'_> {
        PolygonIter {
            state: PolygonIterState::Open,
            vertices: self.vertices,
        }
    }

    // All of these are unnecessary for drawing
    fn area(&self) -> f64 {
        unimplemented!()
    }

    fn perimeter(&self, _accuracy: f64) -> f64 {
        unimplemented!()
    }

    fn winding(&self, _pt: kurbo::Point) -> i32 {
        unimplemented!()
    }

    fn bounding_box(&self) -> Rect {
        unimplemented!()
    }
}

impl Renderer for vello::Scene {
    type Error = std::convert::Infallible;

    fn push_clip_rect(
        &mut self,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
    ) -> Result<(), Self::Error> {
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
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
        corner_radius: Float<Pixel>,
        stroke_width: Float<Pixel>,
        color: Color,
    ) -> Result<(), Self::Error> {
        if color.a > 0 {
            let rect = Rect::from_origin_size(position, size);
            let brush = peniko::Brush::Solid(color.into());

            if corner_radius == 0.px() {
                self.stroke(
                    &Stroke::new(stroke_width.value() as f64),
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &rect,
                );
            } else {
                self.stroke(
                    &Stroke::new(stroke_width.value() as f64),
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &rect.to_rounded_rect(corner_radius.value() as f64),
                );
            }
        }

        Ok(())
    }

    fn fill_rect(
        &mut self,
        position: Vec2<Pixel>,
        size: Vec2<Pixel>,
        corner_radius: Float<Pixel>,
        brush: ComputedBrush,
    ) -> Result<(), Self::Error> {
        if let ComputedBrush::Solid(Color { a: 0, .. }) = brush {
            return Ok(());
        };

        let rect = Rect::from_origin_size(position, size);
        let (brush, brush_transform) = convert_brush(brush);

        if corner_radius == 0.px() {
            self.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                brush_transform,
                &rect,
            );
        } else {
            self.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                brush_transform,
                &rect.to_rounded_rect(corner_radius.value() as f64),
            );
        }

        Ok(())
    }

    fn draw_poly(
        &mut self,
        vertices: &[Vec2<Pixel>],
        stroke_width: Float<Pixel>,
        color: Color,
    ) -> Result<(), Self::Error> {
        if color.a > 0 {
            let poly = Polygon { vertices };
            let brush = peniko::Brush::Solid(color.into());

            self.stroke(
                &Stroke::new(stroke_width.value() as f64),
                Affine::IDENTITY,
                &brush,
                None,
                &poly,
            );
        }

        Ok(())
    }

    fn fill_poly(
        &mut self,
        vertices: &[Vec2<Pixel>],
        brush: ComputedBrush,
    ) -> Result<(), Self::Error> {
        if let ComputedBrush::Solid(Color { a: 0, .. }) = brush {
            return Ok(());
        };

        let poly = Polygon { vertices };
        let (brush, brush_transform) = convert_brush(brush);

        self.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &brush,
            brush_transform,
            &poly,
        );

        Ok(())
    }

    fn draw_text(
        &mut self,
        text: GlyphRun<'_, Color>,
        position: Vec2<Pixel>,
    ) -> Result<(), Self::Error> {
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
