use super::Seed;
use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

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
