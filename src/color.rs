use super::Float;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};
use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

/// The color of a pixel in an image. Each component is between 0 and 1.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Color {
    pub red: Float,
    pub green: Float,
    pub blue: Float,
}

impl Color {
    /// Generates a random color.
    pub fn random<R: Rng>(mut rng: R) -> Self {
        Self {
            red: rng.gen(),
            green: rng.gen(),
            blue: rng.gen(),
        }
    }

    pub fn powf(self, n: Float) -> Self {
        Self {
            red: self.red.powf(n),
            green: self.green.powf(n),
            blue: self.blue.powf(n),
        }
    }

    pub fn clamp(self, min: Float, max: Float) -> Self {
        Self {
            red: self.red.clamp(min, max),
            green: self.green.clamp(min, max),
            blue: self.blue.clamp(min, max),
        }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            red: self.red - rhs.red,
            green: self.green - rhs.green,
            blue: self.blue - rhs.blue,
        }
    }
}

impl Mul<Float> for Color {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl Div<Float> for Color {
    type Output = Self;

    fn div(self, rhs: Float) -> Self {
        Self {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl MulAssign<Float> for Color {
    fn mul_assign(&mut self, rhs: Float) {
        *self = *self * rhs
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        *self = *self / rhs
    }
}
