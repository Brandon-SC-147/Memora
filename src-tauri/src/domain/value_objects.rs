//! Value objects del dominio.

use sha2::{Digest, Sha256};
use std::fmt;

use super::errors::DomainError;

/// Hash de contenido (sha256 hex) usado para detección de duplicados.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// Calcula el hash sha256 del contenido textual.
    pub fn compute(content: &str) -> ContentHash {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        ContentHash(hex::encode(hasher.finalize()))
    }

    /// Construye un hash desde un string hex validado.
    pub fn from_hex(hex_str: &str) -> Result<ContentHash, DomainError> {
        if hex_str.len() != 64 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::InvalidHash(hex_str.to_string()));
        }
        Ok(ContentHash(hex_str.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ContentHash> for String {
    fn from(hash: ContentHash) -> Self {
        hash.0
    }
}

/// Timestamp en formato ISO 8601 UTC (consistente con SQLite strftime).
pub mod timestamp {
    /// Genera 'now' en el mismo formato usado por SQLite: `%Y-%m-%dT%H:%M:%fZ`.
    pub fn now() -> String {
        let ts = time::OffsetDateTime::now_utc();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
            ts.year(),
            ts.month() as u8,
            ts.day(),
            ts.hour(),
            ts.minute(),
            ts.second(),
            ts.microsecond(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_stable() {
        let a = ContentHash::compute("docker compose up");
        let b = ContentHash::compute("docker compose up");
        assert_eq!(a, b);
        assert_eq!(a.as_str().len(), 64);
    }

    #[test]
    fn content_hash_changes_with_content() {
        let a = ContentHash::compute("abc");
        let b = ContentHash::compute("abd");
        assert_ne!(a, b);
    }

    #[test]
    fn from_hex_validates_length_and_chars() {
        let valid = "a".repeat(64);
        assert!(ContentHash::from_hex(&valid).is_ok());
        assert!(ContentHash::from_hex("abc").is_err());
        assert!(ContentHash::from_hex(&"g".repeat(64)).is_err());
    }

    #[test]
    fn timestamp_is_utc_iso() {
        let now = timestamp::now();
        assert!(now.ends_with('Z'));
        assert_eq!(now.len(), 27); // %Y-%m-%dT%H:%M:%S.%fZ con microsegundos
    }
}