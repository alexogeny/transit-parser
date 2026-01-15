//! TXC schema version handling.

/// Supported TXC schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxcSchemaVersion {
    V2_4,
    V2_5,
    Unknown,
}

impl TxcSchemaVersion {
    /// Parse version from schema location string.
    pub fn from_schema_location(location: &str) -> Self {
        if location.contains("2.5") {
            Self::V2_5
        } else if location.contains("2.4") {
            Self::V2_4
        } else {
            Self::Unknown
        }
    }

    /// Get the version string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V2_4 => "2.4",
            Self::V2_5 => "2.5",
            Self::Unknown => "unknown",
        }
    }
}

impl Default for TxcSchemaVersion {
    fn default() -> Self {
        Self::V2_4
    }
}
