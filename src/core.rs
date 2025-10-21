//! Core shared functionality between sync and async implementations

use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::Error;

/// Type alias for Result with the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// An empty response structure, used for endpoints that return no data
#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyResponse;

/// Authentication credentials for the Jobsuche API
///
/// The Jobsuche API uses a simple API key authentication scheme.
/// The API key is sent via the `X-API-Key` HTTP header.
///
/// # Default API Key
///
/// The public API key is: `jobboerse-jobsuche`
#[derive(Clone, Debug)]
pub enum Credentials {
    /// API Key authentication (default: "jobboerse-jobsuche")
    ApiKey(String),
}

impl Default for Credentials {
    fn default() -> Self {
        Self::ApiKey("jobboerse-jobsuche".to_string())
    }
}

/// Common data required for both sync and async clients
#[derive(Clone, Debug)]
pub struct ClientCore {
    pub host: Url,
    pub credentials: Credentials,
}

impl ClientCore {
    /// Create a new ClientCore with the given host and credentials
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL of the Jobsuche API (e.g., "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service")
    /// * `credentials` - Authentication credentials (typically the default API key)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::core::{ClientCore, Credentials};
    ///
    /// let core = ClientCore::new(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default()
    /// ).unwrap();
    /// ```
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Self>
    where
        H: Into<String>,
    {
        let host_string = host.into();
        let parsed_host = Url::parse(&host_string).map_err(|e| {
            debug!("Failed to parse host URL '{}': {}", host_string, e);
            e
        })?;

        Ok(ClientCore {
            host: parsed_host,
            credentials,
        })
    }

    /// Get the API key from credentials
    pub fn api_key(&self) -> &str {
        match &self.credentials {
            Credentials::ApiKey(key) => key,
        }
    }

    /// Build a complete URL path
    pub fn path(&self, segments: &[&str]) -> String {
        let mut url = self.host.clone();
        {
            let mut path_segments = url.path_segments_mut().expect("base URL is valid");
            for segment in segments {
                path_segments.push(segment);
            }
        }
        url.to_string()
    }
}

/// Encode a reference number (refnr) to base64 for use in job details endpoint
///
/// The Jobsuche API requires reference numbers to be base64-encoded when
/// requesting job details. This is a known quirk of the API.
///
/// # Example
///
/// ```
/// use jobsuche::core::encode_refnr;
///
/// let refnr = "10001-1001601666-S";
/// let encoded = encode_refnr(refnr);
/// assert_eq!(encoded, "MTAwMDEtMTAwMTYwMTY2Ni1T");
/// ```
pub fn encode_refnr(refnr: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(refnr.as_bytes())
}

/// Decode a base64-encoded reference number back to its original form
///
/// # Example
///
/// ```
/// use jobsuche::core::decode_refnr;
///
/// let encoded = "MTAwMDEtMTAwMTYwMTY2Ni1T";
/// let decoded = decode_refnr(encoded).unwrap();
/// assert_eq!(decoded, "10001-1001601666-S");
/// ```
pub fn decode_refnr(encoded: &str) -> Result<String> {
    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::STANDARD.decode(encoded)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_refnr() {
        let refnr = "10001-1001601666-S";
        let encoded = encode_refnr(refnr);
        assert_eq!(encoded, "MTAwMDEtMTAwMTYwMTY2Ni1T");
    }

    #[test]
    fn test_decode_refnr() {
        let encoded = "MTAwMDEtMTAwMTYwMTY2Ni1T";
        let decoded = decode_refnr(encoded).unwrap();
        assert_eq!(decoded, "10001-1001601666-S");
    }

    #[test]
    fn test_roundtrip() {
        let refnr = "10000-1184867112-S";
        let encoded = encode_refnr(refnr);
        let decoded = decode_refnr(&encoded).unwrap();
        assert_eq!(refnr, decoded);
    }
}
