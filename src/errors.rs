use reqwest::StatusCode;
use thiserror::Error;

/// An enumeration over potential errors that may happen when sending a request to the Jobsuche API
#[derive(Error, Debug)]
pub enum Error {
    /// Error associated with HTTP request
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Error associated with IO
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Error associated with parsing or serializing
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Client request errors
    #[error("Jobsuche API error ({code}):\n{errors:#?}")]
    Fault { code: StatusCode, errors: ApiErrors },

    /// Unauthorized - invalid API key
    #[error("Could not connect to Jobsuche API: Unauthorized (check your API key)")]
    Unauthorized,

    /// Rate limiting or temporary block
    #[error("Jobsuche API request blocked: Forbidden (possible rate limiting)")]
    Forbidden,

    /// Resource not found (common for job details that have expired)
    #[error("Resource not found (job may have expired or been removed)")]
    NotFound,

    /// HTTP method is not allowed
    #[error("Jobsuche API error: MethodNotAllowed")]
    MethodNotAllowed,

    /// URI parse error
    #[error("Could not connect to Jobsuche API: {0}")]
    ParseError(#[from] url::ParseError),

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Builder validation error
    #[error("Builder validation failed: {message}")]
    BuilderError { message: String },

    /// Base64 encoding/decoding error
    #[error("Base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// API error response structure
#[derive(Debug, serde::Deserialize)]
pub struct ApiErrors {
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub error_messages: Vec<String>,
}

/// Type alias for Result with the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;
