use std::fmt;
use std::ops::*;

pub type Pixel = f32;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: Pixel,
    pub y: Pixel,
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0 };
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0 };
}

impl From<Pixel> for Vec2 {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self { x: value, y: value }
    }
}

impl From<(Pixel, Pixel)> for Vec2 {
    #[inline]
    fn from(value: (Pixel, Pixel)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<[Pixel; 2]> for Vec2 {
    #[inline]
    fn from(value: [Pixel; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<Pixel> for Vec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Pixel) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl AddAssign<Pixel> for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Pixel) {
        *self = *self + rhs;
    }
}

impl Add<Vec2> for Pixel {
    type Output = Vec2;

    #[inline]
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self + rhs.x,
            y: self + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Sub<Pixel> for Vec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Pixel) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl SubAssign<Pixel> for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Pixel) {
        *self = *self - rhs;
    }
}

impl Mul for Vec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Pixel> for Vec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Pixel) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<Pixel> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Pixel) {
        *self = *self * rhs;
    }
}

impl Mul<Vec2> for Pixel {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div for Vec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl DivAssign for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Div<Pixel> for Vec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Pixel) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<Pixel> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Pixel) {
        *self = *self / rhs;
    }
}

impl Rem for Vec2 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x % rhs.x,
            y: self.y % rhs.y,
        }
    }
}

impl RemAssign for Vec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl Rem<Pixel> for Vec2 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Pixel) -> Self::Output {
        Self {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

impl RemAssign<Pixel> for Vec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: Pixel) {
        *self = *self % rhs;
    }
}

impl Vec2 {
    #[inline]
    pub const fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
        }
    }

    #[inline]
    pub const fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
        }
    }
}
