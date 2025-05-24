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

use super::{Color, Dimensions, Float, Position};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

/// A two-dimensional array of pixels.
pub struct Pixmap {
    dimensions: Dimensions,
    data: Vec<Color>,
}

impl Pixmap {
    /// Creates a new [`Pixmap`].
    pub fn new(dimensions: Dimensions) -> Self {
        let mut data = Vec::new();
        data.resize(dimensions.count(), Color::BLACK);
        Self {
            dimensions,
            data,
        }
    }

    /// The dimensions of the image.
    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    #[allow(dead_code)]
    /// The raw pixel data as an immutable reference.
    pub fn data(&self) -> &[Color] {
        &self.data
    }

    /// The raw pixel data as an immutable reference.
    pub fn data_mut(&mut self) -> &mut [Color] {
        &mut self.data
    }

    #[allow(dead_code)]
    /// Takes ownership of the raw pixel data.
    pub fn into_data(self) -> Box<[Color]> {
        self.data.into_boxed_slice()
    }

    /// Calculates the index into the internal array for the given position.
    fn pos_index(&self, pos: Position) -> usize {
        pos.y * self.dimensions.width + pos.x
    }

    /// Gets the pixel at `pos` without bounds checking.
    ///
    /// # Safety
    ///
    /// `pos.x` and `pos.y` must be less than the image width and height,
    /// respectively.
    pub unsafe fn get_unchecked(&self, pos: Position) -> Color {
        // SAFETY: Checked by caller.
        unsafe { *self.data.get_unchecked(self.pos_index(pos)) }
    }

    /// Mutably gets the pixel at `pos` without bounds checking.
    ///
    /// # Safety
    ///
    /// `pos.x` and `pos.y` must be less than the image width and height,
    /// respectively.
    pub unsafe fn get_unchecked_mut(&mut self, pos: Position) -> &mut Color {
        let index = self.pos_index(pos);
        // SAFETY: Checked by caller.
        unsafe { self.data.get_unchecked_mut(index) }
    }

    /// Converts the pixmap to a BMP-style BGR pixel array.
    ///
    /// # Safety
    ///
    /// All color components in the image must be between 0 and 1.
    pub unsafe fn to_bgr_unchecked(&self) -> Vec<u8> {
        let row_size = (self.dimensions.width * 3).div_ceil(4) * 4;
        let padding_len = row_size - (self.dimensions.width * 3);
        let padding_arr = [0_u8; 4];
        let padding = &padding_arr[..padding_len];

        let total = row_size * self.dimensions.height;
        let mut bgr = Vec::with_capacity(total);
        let mut i = 0;
        for color in &self.data {
            i += 1;
            if i == self.dimensions.width {
                bgr.extend_from_slice(padding);
                i = 0;
            }

            let conv = |n: Float| {
                // SAFETY: Checked by caller.
                unsafe { (n * 255.0).round().to_int_unchecked() }
            };
            bgr.push(conv(color.blue));
            bgr.push(conv(color.green));
            bgr.push(conv(color.red));
        }
        bgr
    }
}

impl Index<Position> for Pixmap {
    type Output = Color;

    fn index(&self, pos: Position) -> &Self::Output {
        &self.data[self.pos_index(pos)]
    }
}

impl IndexMut<Position> for Pixmap {
    fn index_mut(&mut self, pos: Position) -> &mut Self::Output {
        let index = self.pos_index(pos);
        &mut self.data[index]
    }
}
