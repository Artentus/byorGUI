use super::computed::*;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
}

impl std::ops::Not for Axis {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

impl Axis {
    #[inline]
    pub fn primary_direction(self) -> Direction {
        match self {
            Axis::X => Direction::LeftToRight,
            Axis::Y => Direction::TopToBottom,
        }
    }

    #[inline]
    pub fn cross_direction(self) -> Direction {
        match self {
            Axis::X => Direction::TopToBottom,
            Axis::Y => Direction::LeftToRight,
        }
    }

    #[inline]
    pub fn is_primary(self, direction: Direction) -> bool {
        self.primary_direction() == direction
    }

    #[inline]
    pub fn persistent_state_scroll_key(self) -> PersistentStateKey {
        match self {
            Axis::X => PersistentStateKey::HorizontalScroll,
            Axis::Y => PersistentStateKey::VerticalScroll,
        }
    }
}

impl Direction {
    #[inline]
    pub fn primary_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::X,
            Direction::TopToBottom => Axis::Y,
        }
    }

    #[inline]
    pub fn cross_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::Y,
            Direction::TopToBottom => Axis::X,
        }
    }
}

impl<U: Unit> Vec2<U> {
    #[inline]
    pub fn along_axis(self, axis: Axis) -> Float<U> {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    #[inline]
    pub fn along_axis_mut(&mut self, axis: Axis) -> &mut Float<U> {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl ComputedPadding {
    #[inline]
    pub fn along_axis(&self, axis: Axis) -> [Float<Pixel>; 2] {
        match axis {
            Axis::X => [self.left, self.right],
            Axis::Y => [self.top, self.bottom],
        }
    }
}

impl Style {
    #[inline]
    pub fn with_size_along_axis(self, axis: Axis, size: impl Into<Sizing>) -> Self {
        match axis {
            Axis::X => self.with_width(size),
            Axis::Y => self.with_height(size),
        }
    }

    #[inline]
    pub fn with_min_size_along_axis(self, axis: Axis, size: impl Into<Measurement>) -> Self {
        match axis {
            Axis::X => self.with_min_width(size),
            Axis::Y => self.with_min_height(size),
        }
    }

    #[inline]
    pub fn with_max_size_along_axis(self, axis: Axis, size: impl Into<Measurement>) -> Self {
        match axis {
            Axis::X => self.with_max_width(size),
            Axis::Y => self.with_max_height(size),
        }
    }
}

impl ComputedStyle {
    #[inline]
    pub(crate) fn size_along_axis(&self, axis: Axis) -> ComputedSizing {
        match axis {
            Axis::X => self.width(),
            Axis::Y => self.height(),
        }
    }
}
