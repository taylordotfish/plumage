use super::{Color, Dimensions, Float, Seed};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Serialize, Deserialize)]
pub struct Params {
    pub dimensions: Dimensions,
    pub spread: usize,
    pub distance_power: Float,
    pub random_power: Float,
    pub random_max: Float,
    pub gamma: Float,
    pub start_color: Option<Color>,
    pub seed: Option<Seed>,
}

impl Params {
    pub const DEFAULT: Self = Self {
        dimensions: Dimensions::new(100, 100),
        spread: 5,
        distance_power: -2.0,
        random_power: 3.0,
        random_max: 0.15,
        gamma: 0.7,
        start_color: None,
        seed: None,
    };

    pub fn fill(mut self) -> FullParams {
        self.start_color.get_or_insert_with(|| Color::random(thread_rng()));
        self.seed.get_or_insert_with(|| {
            let mut seed = Seed::default();
            thread_rng().fill(&mut seed);
            seed
        });
        FullParams(self)
    }
}

pub struct FullParams(Params);

impl FullParams {
    pub fn into_params(self) -> Params {
        self.0
    }
}

impl Deref for FullParams {
    type Target = Params;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
