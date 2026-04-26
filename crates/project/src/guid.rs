use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable identity for an asset. Survives file renames and moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guid(uuid::Uuid);

impl Guid {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        uuid::Uuid::parse_str(s).ok().map(Self)
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}