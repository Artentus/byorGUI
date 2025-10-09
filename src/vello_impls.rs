use crate::rendering::*;
use crate::*;
use vello::kurbo::{self, Affine, Rect, Stroke};
use vello::peniko::Fill;

impl From<Position> for kurbo::Point {
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
        if matches!(brush, Brush::None) {
            return Ok(());
        }

        let rect = Rect::from_origin_size(position, size);
        let brush = match brush {
            Brush::None => unreachable!(),
            Brush::Solid(color) => vello::peniko::Brush::Solid(color),
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
        if matches!(brush, Brush::None) {
            return Ok(());
        }

        let rect = Rect::from_origin_size(position, size);
        let brush = match brush {
            Brush::None => unreachable!(),
            Brush::Solid(color) => vello::peniko::Brush::Solid(color),
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
}
