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
    #[must_use]
    #[inline]
    pub fn primary_direction(self) -> Direction {
        match self {
            Axis::X => Direction::LeftToRight,
            Axis::Y => Direction::TopToBottom,
        }
    }

    #[must_use]
    #[inline]
    pub fn cross_direction(self) -> Direction {
        match self {
            Axis::X => Direction::TopToBottom,
            Axis::Y => Direction::LeftToRight,
        }
    }

    #[must_use]
    #[inline]
    pub fn is_primary(self, direction: Direction) -> bool {
        self.primary_direction() == direction
    }

    #[must_use]
    #[inline]
    pub fn persistent_state_scroll_key(self) -> PersistentStateKey {
        match self {
            Axis::X => PersistentStateKey::HorizontalScroll,
            Axis::Y => PersistentStateKey::VerticalScroll,
        }
    }
}

impl Direction {
    #[must_use]
    #[inline]
    pub fn primary_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::X,
            Direction::TopToBottom => Axis::Y,
        }
    }

    #[must_use]
    #[inline]
    pub fn cross_axis(self) -> Axis {
        match self {
            Direction::LeftToRight => Axis::Y,
            Direction::TopToBottom => Axis::X,
        }
    }
}

impl<U: Unit> Vec2<U> {
    #[must_use]
    #[inline]
    pub fn along_axis(self, axis: Axis) -> Float<U> {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }

    #[must_use]
    #[inline]
    pub fn along_axis_mut(&mut self, axis: Axis) -> &mut Float<U> {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl ComputedPadding {
    #[must_use]
    #[inline]
    pub fn along_axis(&self, axis: Axis) -> [Float<Pixel>; 2] {
        match axis {
            Axis::X => [self.left, self.right],
            Axis::Y => [self.top, self.bottom],
        }
    }
}

impl Style {
    #[must_use]
    #[inline]
    pub fn size_along_axis(&self, axis: Axis) -> Property<Sizing, false> {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_size_along_axis(self, axis: Axis, size: impl Into<Sizing>) -> Self {
        match axis {
            Axis::X => self.with_width(size),
            Axis::Y => self.with_height(size),
        }
    }

    #[must_use]
    #[inline]
    pub fn min_size_along_axis(&self, axis: Axis) -> Property<AbsoluteMeasurement, false> {
        match axis {
            Axis::X => self.min_width,
            Axis::Y => self.min_height,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_min_size_along_axis(
        self,
        axis: Axis,
        size: impl Into<AbsoluteMeasurement>,
    ) -> Self {
        match axis {
            Axis::X => self.with_min_width(size),
            Axis::Y => self.with_min_height(size),
        }
    }

    #[must_use]
    #[inline]
    pub fn max_size_along_axis(&self, axis: Axis) -> Property<AbsoluteMeasurement, false> {
        match axis {
            Axis::X => self.max_width,
            Axis::Y => self.max_height,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_max_size_along_axis(
        self,
        axis: Axis,
        size: impl Into<AbsoluteMeasurement>,
    ) -> Self {
        match axis {
            Axis::X => self.with_max_width(size),
            Axis::Y => self.with_max_height(size),
        }
    }
}

impl CascadedStyle {
    #[must_use]
    #[inline]
    pub fn size_along_axis(&self, axis: Axis) -> Sizing {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }

    #[must_use]
    #[inline]
    pub fn min_size_along_axis(&self, axis: Axis) -> AbsoluteMeasurement {
        match axis {
            Axis::X => self.min_width,
            Axis::Y => self.min_height,
        }
    }

    #[must_use]
    #[inline]
    pub fn max_size_along_axis(&self, axis: Axis) -> AbsoluteMeasurement {
        match axis {
            Axis::X => self.max_width,
            Axis::Y => self.max_height,
        }
    }
}

impl ComputedStyle {
    #[must_use]
    #[inline]
    pub(crate) fn size_along_axis(&self, axis: Axis) -> ComputedSizing {
        match axis {
            Axis::X => self.width(),
            Axis::Y => self.height(),
        }
    }
}
