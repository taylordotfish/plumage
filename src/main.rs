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

#![forbid(unsafe_op_in_unsafe_fn)]
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::exit;

const USAGE: &str = "\
Usage: plumage <name>

Creates `<name>.bmp` and `<name>.params`.
Optionally reads params from `./params`.
";

#[macro_use]
mod error;
mod color;
mod coords;
mod params;
mod pixmap;

use color::Color;
use coords::{Dimensions, Position};
use params::Params;
use pixmap::Pixmap;

type Float = f32;
type Seed = [u8; 32];

/// Shape of the area of neighboring pixels considered when averaging.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Spread {
    Square {
        width: usize,
    },
    QuarterCircle {
        radius: usize,
    },
}

impl Spread {
    /// The width of the bounding box that holds the spread shape.
    pub const fn bound(self) -> usize {
        match self {
            Self::Square {
                width,
            } => width,
            Self::QuarterCircle {
                radius,
            } => radius,
        }
    }
}

/// Generates the image.
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

        let bound = self.spread.bound();
        let bound =
            Dimensions::new(bound.min(pos.x + 1), bound.min(pos.y + 1));
        bound.for_each(|delta| {
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
            // SAFETY: Given a valid starting position, all positions in
            // this loop are valid (due to `saturating_sub`).
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
            let neg: bool = self.rng.gen();
            n * Float::from(neg as i8 * 2 - 1)
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

    /// Fills the entire image.
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

    /// Generates an image and writes it to `stream`.
    pub fn generate<W: Write>(mut self, mut stream: W) -> io::Result<()> {
        self.fill();
        self.apply_gamma();
        let dim = self.data.dimensions();

        // The algorithm we applied ensures no color components can fall
        // outside [0, 1].
        let bgr = unsafe { self.data.to_bgr_unchecked() };
        drop(self.data);

        // Write bitmap file header.
        stream.write_all(b"BM")?;
        let size: u32 = 14 + 40 + bgr.len() as u32;
        stream.write_all(&size.to_le_bytes())?;
        stream.write_all(b"PLMG")?;
        stream.write_all(&(14_u32 + 40).to_le_bytes())?;

        // Write BITMAPINFOHEADER.
        stream.write_all(&40_u32.to_le_bytes())?;
        stream.write_all(&(dim.width as u32).to_le_bytes())?;
        stream.write_all(&(dim.height as u32).wrapping_neg().to_le_bytes())?;
        stream.write_all(&1_u16.to_le_bytes())?;
        stream.write_all(&24_u16.to_le_bytes())?;
        stream.write_all(&0_u32.to_le_bytes())?;
        stream.write_all(&0_u32.to_le_bytes())?;
        stream.write_all(&96_u32.to_le_bytes())?;
        stream.write_all(&96_u32.to_le_bytes())?;
        stream.write_all(&0_u32.to_le_bytes())?;
        stream.write_all(&0_u32.to_le_bytes())?;

        // Write pixel array.
        stream.write_all(&bgr)?;
        Ok(())
    }
}

fn deserialize_params<R: Read>(stream: R) -> Params {
    ron::de::from_reader(stream).unwrap_or_else(|e| {
        error_exit!("error reading params: {e}");
    })
}

fn usage() {
    print!("{USAGE}");
    exit(0);
}

fn params_write_failed<T>(e: impl Display) -> T {
    error_exit!("could not write to output params file: {e}");
}

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let Some(mut filename) = args.next() else {
        args_error!("missing <name>");
    };

    if filename == "-h" || filename == "--help" {
        usage();
    }

    let params = if let Ok(f) = File::open("params") {
        deserialize_params(f)
    } else {
        deserialize_params("()".as_bytes())
    };

    // Create params file.
    let filename_len = filename.len();
    filename += ".params";
    let mut f = File::create(&filename).unwrap_or_else(|e| {
        error_exit!("could not create output params file: {e}");
    });

    let pretty = PrettyConfig::new().depth_limit(1);
    ron::ser::to_writer_pretty(&mut f, &params, pretty)
        .unwrap_or_else(params_write_failed);
    writeln!(f).unwrap_or_else(params_write_failed);
    drop(f);

    // Create image.
    filename.replace_range(filename_len.., ".bmp");
    let generator = Generator::new(params);
    let f = File::create(filename).unwrap_or_else(|e| {
        error_exit!("could not create output file: {e}");
    });
    generator.generate(f).unwrap_or_else(|e| {
        error_exit!("error generating image: {e}");
    });
}
