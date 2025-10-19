use std::fmt;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::*;

mod private {
    pub trait SealedUnit {}
}

pub trait Unit: private::SealedUnit {
    const SUFFIX: &'static str;
}

macro_rules! unit {
    ($name:ident($suffix:literal)) => {
        pub enum $name {}

        impl private::SealedUnit for $name {}

        impl Unit for $name {
            const SUFFIX: &'static str = $suffix;
        }
    };
}

unit!(Pixel("px"));

unit!(Point("pt"));

unit!(EM("em"));

#[repr(transparent)]
pub struct Float<U: Unit> {
    value: f32,
    _unit: PhantomData<fn() -> U>,
}

impl<U: Unit> Float<U> {
    #[must_use]
    #[inline]
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn value(self) -> f32 {
        self.value
    }
}

impl Float<Pixel> {
    #[must_use]
    #[inline]
    pub const fn px(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<f32> for Float<Pixel> {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl Float<Point> {
    #[must_use]
    #[inline]
    pub const fn pt(value: f32) -> Self {
        Self::new(value)
    }

    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_point: f32) -> Float<Pixel> {
        Float::px(self.value * pixel_per_point)
    }
}

impl From<f32> for Float<Point> {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl Float<EM> {
    #[must_use]
    #[inline]
    pub const fn em(value: f32) -> Self {
        Self::new(value)
    }

    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_em: f32) -> Float<Pixel> {
        Float::px(self.value * pixel_per_em)
    }
}

impl From<f32> for Float<EM> {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

pub trait IntoFloat {
    #[must_use]
    fn px(self) -> Float<Pixel>;

    #[must_use]
    fn pt(self) -> Float<Point>;

    #[must_use]
    fn em(self) -> Float<EM>;
}

macro_rules! impl_into_float {
    ($t:ty) => {
        impl IntoFloat for $t {
            #[inline]
            fn px(self) -> Float<Pixel> {
                Float::px(self as f32)
            }

            #[inline]
            fn pt(self) -> Float<Point> {
                Float::pt(self as f32)
            }

            #[inline]
            fn em(self) -> Float<EM> {
                Float::em(self as f32)
            }
        }
    };
}

impl_into_float!(u8);
impl_into_float!(u16);
impl_into_float!(u32);
impl_into_float!(u64);
impl_into_float!(usize);
impl_into_float!(i8);
impl_into_float!(i16);
impl_into_float!(i32);
impl_into_float!(i64);
impl_into_float!(isize);
impl_into_float!(f32);
impl_into_float!(f64);

impl<U: Unit> Default for Float<U> {
    #[inline]
    fn default() -> Self {
        Self {
            value: f32::default(),
            _unit: PhantomData,
        }
    }
}

impl<U: Unit> Copy for Float<U> {}

impl<U: Unit> Clone for Float<U> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<U: Unit> fmt::Debug for Float<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, U::SUFFIX)
    }
}

impl<U: Unit> fmt::Display for Float<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, U::SUFFIX)
    }
}

impl<U: Unit> PartialEq for Float<U> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<U: Unit> PartialOrd for Float<U> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<U: Unit> Neg for Float<U> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.value)
    }
}

impl<U: Unit> Add for Float<U> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.value + rhs.value)
    }
}

impl<U: Unit> AddAssign for Float<U> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<U: Unit> Sub for Float<U> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.value - rhs.value)
    }
}

impl<U: Unit> SubAssign for Float<U> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<U: Unit> Mul<f32> for Float<U> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.value * rhs)
    }
}

impl<U: Unit> MulAssign<f32> for Float<U> {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl<U: Unit> Mul<Float<U>> for f32 {
    type Output = Float<U>;

    fn mul(self, rhs: Float<U>) -> Self::Output {
        Float::new(self * rhs.value)
    }
}

impl<U: Unit> Div<f32> for Float<U> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.value / rhs)
    }
}

impl<U: Unit> DivAssign<f32> for Float<U> {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl<U: Unit> Div for Float<U> {
    type Output = f32;

    #[inline]
    fn div(self, rhs: Float<U>) -> Self::Output {
        self.value / rhs.value
    }
}

impl<U: Unit> Rem<f32> for Float<U> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: f32) -> Self::Output {
        Self::new(self.value % rhs)
    }
}

impl<U: Unit> RemAssign<f32> for Float<U> {
    #[inline]
    fn rem_assign(&mut self, rhs: f32) {
        *self = *self % rhs;
    }
}

impl<U: Unit> Rem for Float<U> {
    type Output = Float<U>;

    #[inline]
    fn rem(self, rhs: Float<U>) -> Self::Output {
        Float::new(self.value % rhs.value)
    }
}

impl<U: Unit> Float<U> {
    #[must_use]
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        Self::new(self.value.min(other.value))
    }

    #[must_use]
    #[inline]
    pub const fn max(self, other: Self) -> Self {
        Self::new(self.value.max(other.value))
    }

    #[must_use]
    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.value.clamp(min.value, max.value))
    }

    #[must_use]
    #[inline]
    pub const fn floor(self) -> Self {
        Self::new(self.value.floor())
    }

    #[must_use]
    #[inline]
    pub const fn ceil(self) -> Self {
        Self::new(self.value.ceil())
    }

    #[must_use]
    #[inline]
    pub const fn round(self) -> Self {
        Self::new(self.value.round())
    }

    #[must_use]
    #[inline]
    pub const fn fract(self) -> Self {
        Self::new(self.value.fract())
    }
}

impl<U: Unit> Sum for Float<U> {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self::new(iter.map(Self::value).sum())
    }
}

#[repr(C)]
pub struct Vec2<U: Unit> {
    pub x: Float<U>,
    pub y: Float<U>,
}

impl Vec2<Point> {
    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_point: f32) -> Vec2<Pixel> {
        Vec2 {
            x: self.x.to_pixel(pixel_per_point),
            y: self.y.to_pixel(pixel_per_point),
        }
    }
}

impl Vec2<EM> {
    #[must_use]
    #[inline]
    pub fn to_pixel(self, pixel_per_em: f32) -> Vec2<Pixel> {
        Vec2 {
            x: self.x.to_pixel(pixel_per_em),
            y: self.y.to_pixel(pixel_per_em),
        }
    }
}

impl<U: Unit> Default for Vec2<U> {
    #[inline]
    fn default() -> Self {
        Self {
            x: Float::default(),
            y: Float::default(),
        }
    }
}

impl<U: Unit> Copy for Vec2<U> {}

impl<U: Unit> Clone for Vec2<U> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<U: Unit> fmt::Debug for Vec2<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<U: Unit> fmt::Display for Vec2<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<U: Unit> Vec2<U> {
    pub const ZERO: Self = Self {
        x: Float::new(0.0),
        y: Float::new(0.0),
    };
    pub const UNIT_X: Self = Self {
        x: Float::new(1.0),
        y: Float::new(0.0),
    };
    pub const UNIT_Y: Self = Self {
        x: Float::new(0.0),
        y: Float::new(1.0),
    };
}

impl<U: Unit> From<Float<U>> for Vec2<U> {
    #[inline]
    fn from(value: Float<U>) -> Self {
        Self { x: value, y: value }
    }
}

impl<U: Unit> From<(Float<U>, Float<U>)> for Vec2<U> {
    #[inline]
    fn from(value: (Float<U>, Float<U>)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<U: Unit> From<[Float<U>; 2]> for Vec2<U> {
    #[inline]
    fn from(value: [Float<U>; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl<U: Unit> PartialEq for Vec2<U> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.x.eq(&other.x) && self.y.eq(&other.y)
    }
}

impl<U: Unit> Neg for Vec2<U> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<U: Unit> Add for Vec2<U> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<U: Unit> AddAssign for Vec2<U> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<U: Unit> Add<Float<U>> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Float<U>) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl<U: Unit> AddAssign<Float<U>> for Vec2<U> {
    #[inline]
    fn add_assign(&mut self, rhs: Float<U>) {
        *self = *self + rhs;
    }
}

impl<U: Unit> Add<Vec2<U>> for Float<U> {
    type Output = Vec2<U>;

    #[inline]
    fn add(self, rhs: Vec2<U>) -> Self::Output {
        Vec2 {
            x: self + rhs.x,
            y: self + rhs.y,
        }
    }
}

impl<U: Unit> Sub for Vec2<U> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<U: Unit> SubAssign for Vec2<U> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<U: Unit> Sub<Float<U>> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Float<U>) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl<U: Unit> SubAssign<Float<U>> for Vec2<U> {
    #[inline]
    fn sub_assign(&mut self, rhs: Float<U>) {
        *self = *self - rhs;
    }
}

impl<U: Unit> Mul<f32> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<U: Unit> MulAssign<f32> for Vec2<U> {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl<U: Unit> Mul<Vec2<U>> for f32 {
    type Output = Vec2<U>;

    #[inline]
    fn mul(self, rhs: Vec2<U>) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl<U: Unit> Mul<(f32, f32)> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: (f32, f32)) -> Self::Output {
        Self {
            x: self.x * rhs.0,
            y: self.y * rhs.1,
        }
    }
}

impl<U: Unit> MulAssign<(f32, f32)> for Vec2<U> {
    #[inline]
    fn mul_assign(&mut self, rhs: (f32, f32)) {
        *self = *self * rhs;
    }
}

impl<U: Unit> Mul<Vec2<U>> for (f32, f32) {
    type Output = Vec2<U>;

    #[inline]
    fn mul(self, rhs: Vec2<U>) -> Self::Output {
        Vec2 {
            x: self.0 * rhs.x,
            y: self.1 * rhs.y,
        }
    }
}

impl<U: Unit> Mul<[f32; 2]> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: [f32; 2]) -> Self::Output {
        Self {
            x: self.x * rhs[0],
            y: self.y * rhs[1],
        }
    }
}

impl<U: Unit> MulAssign<[f32; 2]> for Vec2<U> {
    #[inline]
    fn mul_assign(&mut self, rhs: [f32; 2]) {
        *self = *self * rhs;
    }
}

impl<U: Unit> Mul<Vec2<U>> for [f32; 2] {
    type Output = Vec2<U>;

    #[inline]
    fn mul(self, rhs: Vec2<U>) -> Self::Output {
        Vec2 {
            x: self[0] * rhs.x,
            y: self[1] * rhs.y,
        }
    }
}

impl<U: Unit> Div<f32> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<U: Unit> DivAssign<f32> for Vec2<U> {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl<U: Unit> Div<(f32, f32)> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: (f32, f32)) -> Self::Output {
        Self {
            x: self.x / rhs.0,
            y: self.y / rhs.1,
        }
    }
}

impl<U: Unit> DivAssign<(f32, f32)> for Vec2<U> {
    #[inline]
    fn div_assign(&mut self, rhs: (f32, f32)) {
        *self = *self / rhs;
    }
}

impl<U: Unit> Div<[f32; 2]> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: [f32; 2]) -> Self::Output {
        Self {
            x: self.x / rhs[0],
            y: self.y / rhs[1],
        }
    }
}

impl<U: Unit> DivAssign<[f32; 2]> for Vec2<U> {
    #[inline]
    fn div_assign(&mut self, rhs: [f32; 2]) {
        *self = *self / rhs;
    }
}

impl<U: Unit> Rem<f32> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

impl<U: Unit> RemAssign<f32> for Vec2<U> {
    #[inline]
    fn rem_assign(&mut self, rhs: f32) {
        *self = *self % rhs;
    }
}

impl<U: Unit> Rem<(f32, f32)> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: (f32, f32)) -> Self::Output {
        Self {
            x: self.x % rhs.0,
            y: self.y % rhs.1,
        }
    }
}

impl<U: Unit> RemAssign<(f32, f32)> for Vec2<U> {
    #[inline]
    fn rem_assign(&mut self, rhs: (f32, f32)) {
        *self = *self % rhs;
    }
}

impl<U: Unit> Rem<[f32; 2]> for Vec2<U> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: [f32; 2]) -> Self::Output {
        Self {
            x: self.x % rhs[0],
            y: self.y % rhs[1],
        }
    }
}

impl<U: Unit> RemAssign<[f32; 2]> for Vec2<U> {
    #[inline]
    fn rem_assign(&mut self, rhs: [f32; 2]) {
        *self = *self % rhs;
    }
}

impl<U: Unit> Vec2<U> {
    #[must_use]
    #[inline]
    pub const fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
        }
    }

    #[must_use]
    #[inline]
    pub const fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
        }
    }

    #[must_use]
    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    #[must_use]
    #[inline]
    pub const fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn fract(self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
        }
    }
}
