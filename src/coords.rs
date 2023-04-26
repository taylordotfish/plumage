/*
 * Copyright (C) 2023 taylor.fish <contact@taylor.fish>
 *
 * This file is part of Plumage.
 *
 * Plumage is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Plumage is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with Plumage. If not, see <https://www.gnu.org/licenses/>.
 */

use core::ops::{Add, AddAssign, Sub, SubAssign};
use serde::{Deserialize, Serialize};

/// The dimensions of an image.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    /// Creates a new set of [`Dimensions`].
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
        }
    }

    /// Same as [`Self::new(width, width)`].
    pub const fn square(width: usize) -> Self {
        Self::new(width, width)
    }

    /// The total number of pixels in the image.
    pub const fn count(&self) -> usize {
        self.width * self.height
    }

    /// Calls a function for each pixel in the image.
    pub fn for_each<F: FnMut(Position)>(self, mut f: F) {
        for y in 0..self.height {
            for x in 0..self.width {
                f(Position::new(x, y));
            }
        }
    }

    /// Calls [`min`](usize::min) on each dimension.
    pub fn min(self, other: Self) -> Self {
        Self {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    #[allow(dead_code)]
    /// Calls [`max`](usize::max) on each dimension.
    pub fn max(self, other: Self) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }
}

impl From<Position> for Dimensions {
    fn from(pos: Position) -> Self {
        Self {
            width: pos.x,
            height: pos.y,
        }
    }
}

/// A position within an image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// (0, 0).
    pub const ZERO: Self = Position::new(0, 0);

    /// Creates a new [`Position`].
    pub const fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl From<Dimensions> for Position {
    fn from(dim: Dimensions) -> Self {
        Self {
            x: dim.width,
            y: dim.height,
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}
