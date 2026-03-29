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
#[derive(Clone)]
pub enum Credentials {
    /// API Key authentication (default: "jobboerse-jobsuche")
    ApiKey(String),
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => f.debug_tuple("ApiKey").field(&"[REDACTED]").finish(),
        }
    }
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
        let parsed_host = Url::parse(&host_string).inspect_err(|e| {
            debug!("Failed to parse host URL '{}': {}", host_string, e);
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
/// Reference numbers are expected to contain only ASCII alphanumeric characters
/// and hyphens (e.g., `10001-1001601666-S`). Inputs that are empty, longer than
/// 50 characters, or contain unexpected characters will trigger a warning log
/// but will still be encoded to avoid breaking existing callers.
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

    if refnr.is_empty() {
        tracing::warn!("encode_refnr called with empty string");
    } else if refnr.len() > 50 {
        tracing::warn!(
            "encode_refnr called with unusually long input ({} chars)",
            refnr.len()
        );
    } else if !refnr
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        tracing::warn!(
            "encode_refnr called with non-standard characters: {:?}",
            refnr
        );
    }

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
    use tracing_test::traced_test;

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

    #[test]
    fn test_credentials_debug_redacts_key() {
        let creds = Credentials::default();
        let debug_output = format!("{:?}", creds);
        assert!(
            !debug_output.contains("jobboerse-jobsuche"),
            "API key must not appear in Debug output"
        );
        assert!(
            debug_output.contains("REDACTED"),
            "Debug output should show REDACTED"
        );
    }

    #[test]
    fn test_encode_refnr_valid_formats() {
        let encoded = encode_refnr("10001-1001601666-S");
        assert!(!encoded.is_empty());

        let encoded = encode_refnr("10001-TEST123-S");
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_refnr_empty() {
        let encoded = encode_refnr("");
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_encode_refnr_long_input() {
        let long_input = "a".repeat(51);
        let encoded = encode_refnr(&long_input);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_refnr_non_standard_chars() {
        let encoded = encode_refnr("10001/1001601666@S");
        assert!(!encoded.is_empty());
    }

    // --- Mutation-killing tests ---

    #[test]
    fn test_api_key_returns_default_value() {
        let core = ClientCore::new("https://example.com", Credentials::default()).unwrap();
        assert_eq!(core.api_key(), "jobboerse-jobsuche");
    }

    #[test]
    fn test_api_key_returns_custom_value() {
        let core = ClientCore::new(
            "https://example.com",
            Credentials::ApiKey("custom-key".to_string()),
        )
        .unwrap();
        assert_eq!(core.api_key(), "custom-key");
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_no_warn_on_length_exactly_50() {
        let exact = "a".repeat(50);
        encode_refnr(&exact);
        assert!(!logs_contain("unusually long input"));
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_warns_on_length_51() {
        let long = "a".repeat(51);
        encode_refnr(&long);
        assert!(logs_contain("unusually long input"));
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_no_warn_on_valid_refnr() {
        encode_refnr("10001-TEST-S");
        assert!(!logs_contain("non-standard characters"));
        assert!(!logs_contain("unusually long"));
        assert!(!logs_contain("empty"));
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_hyphen_only_is_valid() {
        encode_refnr("---");
        assert!(!logs_contain("non-standard"));
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_warns_on_non_standard_chars_traced() {
        encode_refnr("hello@world");
        assert!(logs_contain("non-standard characters"));
    }

    #[traced_test]
    #[test]
    fn test_encode_refnr_warns_on_empty_traced() {
        encode_refnr("");
        assert!(logs_contain("empty string"));
    }
}
