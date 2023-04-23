use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process::exit;

mod params;
use params::{FullParams, Params};

type Float = f32;
type Seed = [u8; 32];

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub red: Float,
    pub green: Float,
    pub blue: Float,
}

impl Color {
    pub fn random<R: Rng>(mut rng: R) -> Self {
        Self {
            red: rng.gen(),
            green: rng.gen(),
            blue: rng.gen(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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

    pub fn count(&self) -> usize {
        self.width * self.height
    }
}

#[derive(Clone, Copy)]
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

pub struct ImageData {
    dimensions: Dimensions,
    data: Vec<Option<Color>>,
}

impl ImageData {
    pub fn new(dimensions: Dimensions) -> Self {
        let mut data = Vec::new();
        data.resize(dimensions.count(), None);
        Self {
            dimensions,
            data,
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        self.dimensions
    }

    #[allow(dead_code)]
    pub fn data(&self) -> &[Option<Color>] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [Option<Color>] {
        &mut self.data
    }

    pub fn into_data(self) -> Vec<Option<Color>> {
        self.data
    }

    fn pos_index(&self, pos: Position) -> usize {
        pos.y * self.dimensions.width + pos.x
    }

    pub fn get(&self, pos: Position) -> Option<Color> {
        self.data.get(self.pos_index(pos)).copied().flatten()
    }

    pub fn set(&mut self, pos: Position, color: Color) {
        let index = self.pos_index(pos);
        self.data[index] = Some(color);
    }
}

pub struct Generator {
    spread: usize,
    distance_power: Float,
    random_power: Float,
    random_max: Float,
    gamma: Float,
    data: ImageData,
    rng: ChaChaRng,
}

impl Generator {
    pub fn new(params: FullParams) -> Self {
        let params = params.into_params();
        let rng = ChaChaRng::from_seed(params.seed.unwrap());
        let mut data = ImageData::new(params.dimensions);
        data.set(Position::new(0, 0), params.start_color.unwrap());
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

    fn avg_neighbor(&self, pos: Position) -> Color {
        let spread = self.spread;
        let mut count = 0.0;
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;

        for y in pos.y.saturating_sub(spread)..=pos.y {
            for x in pos.x.saturating_sub(spread)..=pos.x {
                let Some(color) = self.data.get(Position::new(x, y)) else {
                    continue;
                };

                let dx = x as Float - pos.x as Float;
                let dy = y as Float - pos.y as Float;
                let dist = (dx.powf(2.0) + dy.powf(2.0)).powf(0.5);
                let mult = dist.powf(self.distance_power);

                r += color.red * mult;
                g += color.green * mult;
                b += color.blue * mult;
                count += mult;
            }
        }

        Color {
            red: r / count,
            green: g / count,
            blue: b / count,
        }
    }

    fn random_near(&mut self, color: Color) -> Color {
        let mut component = || {
            let n: Float = self.rng.gen();
            let n = n.powf(self.random_power) * self.random_max;
            let neg: bool = self.rng.gen();
            n * Float::from(neg as i8 * 2 - 1)
        };

        Color {
            red: (color.red + component()).clamp(0.0, 1.0),
            green: (color.green + component()).clamp(0.0, 1.0),
            blue: (color.blue + component()).clamp(0.0, 1.0),
        }
    }

    fn fill_pos(&mut self, pos: Position) {
        if self.data.get(pos).is_some() {
            return;
        }
        let neighbor = self.avg_neighbor(pos);
        let color = self.random_near(neighbor);
        self.data.set(pos, color);
    }

    fn fill(&mut self) {
        let dim = self.data.dimensions();
        for y in 0..dim.height {
            for x in 0..dim.width {
                self.fill_pos(Position::new(x, y));
            }
        }
    }

    fn apply_gamma(&mut self) {
        for color in self.data.data_mut() {
            let color = color.as_mut().unwrap();
            color.red = color.red.powf(self.gamma);
            color.green = color.green.powf(self.gamma);
            color.blue = color.blue.powf(self.gamma);
        }
    }

    pub fn generate<W: Write>(mut self, mut stream: W) -> io::Result<()> {
        self.fill();
        self.apply_gamma();
        let dim = self.data.dimensions();
        let data = self.data.into_data();

        let mut int_data = Vec::with_capacity(data.len() * 3);
        for color in data {
            let color = color.unwrap();
            let conv = |n: Float| (n * 255.0).round() as u8;
            int_data.push(conv(color.blue));
            int_data.push(conv(color.green));
            int_data.push(conv(color.red));
        }

        let size: u32 = 14 + 40 + int_data.len() as u32;
        stream.write_all(b"BM")?;
        stream.write_all(&size.to_le_bytes())?;
        stream.write_all(b"PLMG")?;
        stream.write_all(&(14_u32 + 40).to_le_bytes())?;

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

        stream.write_all(&int_data)?;
        Ok(())
    }
}

#[macro_use]
mod error_exit {
    use std::fmt::Display;
    use std::process::exit;

    macro_rules! error_exit {
        ($($args:tt)*) => {
            crate::error_exit::__run(format_args!($($args)*))
        };
    }

    #[doc(hidden)]
    pub fn __run(args: impl Display) -> ! {
        eprintln!("error: {}", args);
        if cfg!(feature = "panic") {
            panic!("error: {}", args);
        } else {
            exit(1);
        }
    }
}

fn main() {
    let pretty = || PrettyConfig::new().struct_names(true).depth_limit(1);
    let params = if let Ok(f) = File::open("params") {
        ron::de::from_reader(f).unwrap_or_else(|e| {
            error_exit!("error reading params: {e}");
        })
    } else {
        Params::DEFAULT.clone()
    };

    let mut args = env::args();
    let _ = args.next();
    let Some(filename) = args.next() else {
        eprintln!("Usage: plumage <output-name>");
        exit(1);
    };

    let params = params.fill();
    let f = File::create(filename.clone() + ".params").unwrap_or_else(|e| {
        error_exit!("could not create output params file: {e}");
    });
    ron::ser::to_writer_pretty(f, &*params, pretty()).unwrap_or_else(|e| {
        error_exit!("could not write to output params file: {e}");
    });

    let generator = Generator::new(params);
    let f = File::create(filename + ".bmp").unwrap_or_else(|e| {
        error_exit!("could not create output file: {e}");
    });
    generator.generate(f).unwrap_or_else(|e| {
        error_exit!("error generating image: {e}");
    });
}
