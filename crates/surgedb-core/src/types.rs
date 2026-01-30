//! Core types for SurgeDB

use serde::{Deserialize, Serialize};

/// A vector represented as a slice of f32 values
pub type Vector = [f32];

/// External vector identifier (user-facing)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorId(String);

impl VectorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for VectorId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for VectorId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for VectorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Helper for serializing/deserializing metadata with bincode
/// Bincode does not support deserialize_any, which serde_json::Value uses.
/// We work around this by serializing Value to/from a JSON string.
pub mod metadata_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde_json::Value;

    pub fn serialize<S>(value: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => {
                let s = serde_json::to_string(v).map_err(serde::ser::Error::custom)?;
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => {
                let v = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }
}

/// Internal vector identifier (for indexing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternalId(pub(crate) u32);

impl InternalId {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<usize> for InternalId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}
