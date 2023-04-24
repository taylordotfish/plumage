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

use super::{Color, Dimensions, Float, Seed};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

mod seed;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    #[serde(default = "Params::default_dimensions")]
    pub dimensions: Dimensions,
    #[serde(default = "Params::default_spread")]
    pub spread: usize,
    #[serde(default = "Params::default_distance_power")]
    pub distance_power: Float,
    #[serde(default = "Params::default_random_power")]
    pub random_power: Float,
    #[serde(default = "Params::default_random_max")]
    pub random_max: Float,
    #[serde(default = "Params::default_gamma")]
    pub gamma: Float,
    #[serde(default = "Params::default_start_color")]
    pub start_color: Color,
    #[serde(default = "Params::default_seed", with = "seed")]
    pub seed: Seed,
}

impl Params {
    fn default_dimensions() -> Dimensions {
        Dimensions::new(3840, 2160)
    }

    fn default_spread() -> usize {
        5
    }

    fn default_distance_power() -> Float {
        -1.75
    }

    fn default_random_power() -> Float {
        3.5
    }

    fn default_random_max() -> Float {
        0.05
    }

    fn default_gamma() -> Float {
        0.75
    }

    fn default_start_color() -> Color {
        Color::random(thread_rng())
    }

    fn default_seed() -> Seed {
        let mut seed = Seed::default();
        thread_rng().fill(&mut seed);
        seed
    }
}
