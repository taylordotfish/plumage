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
mod params;
mod pixmap;

use color::Color;
use params::Params;
use pixmap::Pixmap;

type Float = f32;
type Seed = [u8; 32];

/// The dimensions of an image.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
        }
    }

    /// The total number of pixels in the image.
    pub const fn count(&self) -> usize {
        self.width * self.height
    }
}

/// A position within an image.
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub const fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
        }
    }
}

/// Generates the image.
pub struct Generator {
    spread: usize,
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
        let spread = self.spread;
        let mut count = 0.0;
        let mut avg = Color::default();

        let mut iteration = |x: usize, y: usize| {
            // This is the pixel we haven't filled yet.
            if x == pos.x && y == pos.y {
                return;
            }

            // SAFETY: Given a valid starting position, all positions in
            // this loop are valid (due to `saturating_sub`).
            let color =
                unsafe { self.data.get_unchecked(Position::new(x, y)) };

            let dx = x as Float - pos.x as Float;
            let dy = y as Float - pos.y as Float;
            let dist = (dx.powf(2.0) + dy.powf(2.0)).powf(0.5);

            let weight = dist.powf(self.distance_power);
            avg += color * weight;
            count += weight;
        };

        for y in pos.y.saturating_sub(spread)..=pos.y {
            for x in pos.x.saturating_sub(spread)..=pos.x {
                iteration(x, y);
            }
        }
        avg / count
    }

    /// Generates a random color near `color`.
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
        let dim = self.data.dimensions();

        let mut iteration = |x: usize, y: usize| {
            if x == 0 && y == 0 {
                return;
            }
            // SAFETY: We call this method only with valid positions.
            unsafe {
                self.fill_pos_unchecked(Position::new(x, y));
            }
        };

        for y in 0..dim.height {
            for x in 0..dim.width {
                iteration(x, y);
            }
        }
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

    let filename_len = filename.len();
    filename += ".params";
    let mut f = File::create(&filename).unwrap_or_else(|e| {
        error_exit!("could not create output params file: {e}");
    });

    let pretty = PrettyConfig::new().struct_names(true).depth_limit(1);
    ron::ser::to_writer_pretty(&mut f, &params, pretty)
        .unwrap_or_else(params_write_failed);
    writeln!(f).unwrap_or_else(params_write_failed);
    drop(f);

    filename.replace_range(filename_len.., ".bmp");
    let generator = Generator::new(params);
    let f = File::create(filename).unwrap_or_else(|e| {
        error_exit!("could not create output file: {e}");
    });
    generator.generate(f).unwrap_or_else(|e| {
        error_exit!("error generating image: {e}");
    });
}
