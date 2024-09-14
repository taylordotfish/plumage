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

use super::{Color, Float, Params, Pixmap, Position, Spread};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
#[cfg(feature = "std")]
use std::io::{self, Write};

/// Generates and writes the image.
pub struct Generator {
    spread: Spread,
    distance_power: Float,
    random_power: Float,
    random_max: Float,
    gamma: Float,
    data: Pixmap,
    rng: ChaChaRng,
}

impl Generator {
    /// Creates a new [`Generator`].
    pub fn new(params: Params) -> Self {
        let rng = ChaChaRng::from_seed(params.seed);
        let mut data = Pixmap::new(params.dimensions);
        data[Position::new(0, 0)] = params.start_color;
        Self {
            spread: params.spread,
            distance_power: params.distance_power,
            random_power: params.random_power,
            random_max: params.random_max,
            gamma: params.gamma,
            data,
            rng,
        }
    }

    /// Calculates the average color near a pixel.
    ///
    /// # Safety
    ///
    /// `pos.x` and `pos.y` must be less than the image width and height,
    /// respectively.
    unsafe fn avg_neighbor_unchecked(&self, pos: Position) -> Color {
        let mut count = 0.0;
        let mut avg = Color::BLACK;

        let bounds = self.spread.bounds();
        let bounds = bounds.min((pos + Position::new(1, 1)).into());
        bounds.for_each(|delta| {
            // Skip the pixel we haven't filled yet.
            if delta == Position::ZERO {
                return;
            }

            let dx = delta.x as Float;
            let dy = delta.y as Float;
            let dist = (dx.powf(2.0) + dy.powf(2.0)).powf(0.5);

            if let Spread::QuarterCircle {
                radius,
            } = self.spread
            {
                if dist > radius as Float {
                    return;
                }
            }

            let neighbor = pos - delta;
            // SAFETY: `delta` cannot be greater than `pos`, so `neighbor` is
            // valid.
            let color = unsafe { self.data.get_unchecked(neighbor) };
            let weight = dist.powf(self.distance_power);
            avg += color * weight;
            count += weight;
        });
        avg / count
    }

    /// Generates a random color similar to `color`.
    fn random_near(&mut self, color: Color) -> Color {
        let mut component = || {
            let n: Float = self.rng.gen();
            let n = n.powf(self.random_power) * self.random_max;
            let positive: bool = self.rng.gen();
            n * Float::from(positive as i8 * 2 - 1)
        };
        let delta = Color {
            red: component(),
            green: component(),
            blue: component(),
        };
        (color + delta).clamp(0.0, 1.0)
    }

    /// Fills a single pixel.
    ///
    /// # Safety
    ///
    /// `pos.x` and `pos.y` must be less than the image width and height,
    /// respectively.
    unsafe fn fill_pos_unchecked(&mut self, pos: Position) {
        // SAFETY: Checked by caller.
        let neighbor = unsafe { self.avg_neighbor_unchecked(pos) };
        let color = self.random_near(neighbor);
        // SAFETY: Checked by caller.
        *unsafe { self.data.get_unchecked_mut(pos) } = color;
    }

    /// Fills every pixel in the image.
    fn fill(&mut self) {
        self.data.dimensions().for_each(|pos| {
            // Don't fill the starting pixel.
            if pos == Position::ZERO {
                return;
            }
            // SAFETY: We call this method only with valid positions.
            unsafe {
                self.fill_pos_unchecked(pos);
            }
        })
    }

    /// Applies gamma correction.
    fn apply_gamma(&mut self) {
        for color in self.data.data_mut() {
            *color = color.powf(self.gamma);
        }
    }

    /// Applies all passes.
    fn apply_all(&mut self) {
        self.fill();
        self.apply_gamma();
    }

    #[cfg(feature = "std")]
    /// Generates an image and writes it to `stream`.
    pub fn generate<W: Write>(self, mut stream: W) -> io::Result<()> {
        self.generate_with(|bytes| stream.write_all(bytes))
    }

    /// Generates an image and writes it by calling a custom function.
    ///
    /// `push` should append the given bytes when called.
    pub fn generate_with<F, E>(mut self, mut push: F) -> Result<(), E>
    where
        F: FnMut(&[u8]) -> Result<(), E>,
    {
        self.apply_all();
        let dim = self.data.dimensions();

        // SAFETY: The algorithm we applied ensures no color components can
        // fall outside [0, 1].
        let bgr = unsafe { self.data.to_bgr_unchecked() };
        drop(self.data);
        let size: u32 = 14 + 40 + bgr.len() as u32;

        // Write bitmap file header.
        push(b"BM")?;
        push(&size.to_le_bytes())?;
        push(b"PLMG")?;
        push(&(14_u32 + 40).to_le_bytes())?;

        // Write BITMAPINFOHEADER.
        push(&40_u32.to_le_bytes())?;
        push(&(dim.width as u32).to_le_bytes())?;
        push(&(dim.height as u32).wrapping_neg().to_le_bytes())?;
        push(&1_u16.to_le_bytes())?;
        push(&24_u16.to_le_bytes())?;
        push(&0_u32.to_le_bytes())?;
        push(&0_u32.to_le_bytes())?;
        push(&96_u32.to_le_bytes())?;
        push(&96_u32.to_le_bytes())?;
        push(&0_u32.to_le_bytes())?;
        push(&0_u32.to_le_bytes())?;

        // Write pixel array.
        push(&bgr)?;
        Ok(())
    }
}
