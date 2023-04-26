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

use super::Seed;
use core::fmt;
use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};

pub fn serialize<S>(seed: &Seed, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(seed)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Seed, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_bytes(SeedVisitor)
}

struct SeedVisitor;

impl<'de> Visitor<'de> for SeedVisitor {
    type Value = Seed;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "a sequence of {} bytes", Seed::default().len())
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut seed = Seed::default();
        if bytes.len() != seed.len() {
            return Err(E::invalid_length(bytes.len(), &self));
        }
        seed.copy_from_slice(bytes);
        Ok(seed)
    }
}
