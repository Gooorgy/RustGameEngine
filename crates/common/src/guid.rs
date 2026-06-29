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

    pub const fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

/// Creates a compile-time validated [`Guid`] constant from a UUID string literal.
///
/// The string must be a valid UUID — a typo causes a compile error.
///
/// # Example
/// ```rust
/// use common::{guid, Guid};
/// const FLOOR_OBJ: Guid = guid!("34ac65e2-b2b2-4588-b379-13802dba85bc");
/// ```
#[macro_export]
macro_rules! guid {
    ($s:literal) => {
        $crate::Guid::from_uuid($crate::uuid::uuid!($s))
    };
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}