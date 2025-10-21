//! Asynchronous client for the Jobsuche API
//!
//! This module provides an async/await interface for non-blocking API calls.

use tracing::debug;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Client, Method, StatusCode};
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::de::DeserializeOwned;

use crate::core::{encode_refnr, ClientCore};
use crate::search::SearchAsync;
use crate::sync::ClientConfig;
use crate::{ApiErrors, Credentials, Error, JobDetails, Result};

/// Asynchronous Jobsuche API client
///
/// This is the async counterpart to the synchronous [`Jobsuche`](crate::Jobsuche) client.
/// It uses async/await for non-blocking I/O operations.
///
/// # Example
///
/// ```no_run
/// use jobsuche::{JobsucheAsync, Credentials, SearchOptions};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = JobsucheAsync::new(
///         "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
///         Credentials::default()
///     ).await?;
///
///     // Search for jobs asynchronously
///     let results = client.search()
///         .list(SearchOptions::builder()
///             .was("Rust Developer")
///             .wo("Berlin")
///             .size(10)
///             .build())
///         .await?;
///
///     println!("Found {} jobs", results.stellenangebote.len());
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct JobsucheAsync {
    pub(crate) core: ClientCore,
    client: ClientWithMiddleware,
    #[allow(dead_code)]
    config: ClientConfig,
}

impl JobsucheAsync {
    /// Creates a new async instance with default configuration
    ///
    /// # Arguments
    ///
    /// * `host` - Base URL of the API
    /// * `credentials` - Authentication credentials
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{JobsucheAsync, Credentials};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = JobsucheAsync::new(
    ///         "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///         Credentials::default()
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new<H>(host: H, credentials: Credentials) -> Result<JobsucheAsync>
    where
        H: Into<String>,
    {
        Self::with_config(host, credentials, ClientConfig::default()).await
    }

    /// Creates a new async instance with custom configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{JobsucheAsync, Credentials, ClientConfig};
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ClientConfig {
    ///         timeout: Duration::from_secs(60),
    ///         max_retries: 5,
    ///         ..Default::default()
    ///     };
    ///
    ///     let client = JobsucheAsync::with_config(
    ///         "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///         Credentials::default(),
    ///         config
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_config<H>(
        host: H,
        credentials: Credentials,
        config: ClientConfig,
    ) -> Result<JobsucheAsync>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;

        // Build base reqwest client with timeouts
        let reqwest_client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()?;

        // Wrap with retry middleware if enabled
        let client = if config.retry_enabled {
            let retry_policy =
                ExponentialBackoff::builder().build_with_max_retries(config.max_retries);

            MiddlewareClientBuilder::new(reqwest_client)
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build()
        } else {
            MiddlewareClientBuilder::new(reqwest_client).build()
        };

        Ok(JobsucheAsync {
            core,
            client,
            config,
        })
    }

    /// Creates an async client from an existing ClientCore
    pub async fn with_core(core: ClientCore) -> Result<JobsucheAsync> {
        Self::with_config_and_core(core, ClientConfig::default()).await
    }

    /// Creates an async client from ClientCore with custom config
    pub async fn with_config_and_core(
        core: ClientCore,
        config: ClientConfig,
    ) -> Result<JobsucheAsync> {
        let reqwest_client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()?;

        let client = if config.retry_enabled {
            let retry_policy =
                ExponentialBackoff::builder().build_with_max_retries(config.max_retries);

            MiddlewareClientBuilder::new(reqwest_client)
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build()
        } else {
            MiddlewareClientBuilder::new(reqwest_client).build()
        };

        Ok(JobsucheAsync {
            core,
            client,
            config,
        })
    }

    /// Return async search interface
    pub fn search(&self) -> SearchAsync {
        SearchAsync::new(self)
    }

    /// Get detailed information about a specific job (async)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{JobsucheAsync, Credentials};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = JobsucheAsync::new(
    ///         "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///         Credentials::default()
    ///     ).await?;
    ///
    ///     let job = client.job_details("10001-1001601666-S").await?;
    ///     if let Some(title) = &job.titel {
    ///         println!("Job title: {}", title);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn job_details(&self, refnr: &str) -> Result<JobDetails> {
        let encoded = encode_refnr(refnr);
        let path = self.core.path(&["pc", "v4", "jobdetails", &encoded]);
        self.get(&path).await
    }

    /// Get the logo of an employer (async)
    ///
    /// Returns the raw PNG image bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{JobsucheAsync, Credentials};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = JobsucheAsync::new(
    ///         "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///         Credentials::default()
    ///     ).await?;
    ///
    ///     match client.employer_logo("VK2qoXBe0s-UAdH_qxLDRrZrY5iY8a1PJt3MjJCXsdo=").await {
    ///         Ok(logo_bytes) => println!("Got logo: {} bytes", logo_bytes.len()),
    ///         Err(_) => println!("No logo available"),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn employer_logo(&self, hash_id: &str) -> Result<Vec<u8>> {
        let path = self.core.path(&["ed", "v1", "arbeitgeberlogo", hash_id]);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-API-Key",
            HeaderValue::from_str(self.core.api_key()).unwrap(),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("image/png"));

        let response = self
            .client
            .request(Method::GET, &path)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(self.error_from_status(status, response).await);
        }

        let bytes = response.bytes().await?.to_vec();
        Ok(bytes)
    }

    /// Internal method to perform async GET requests
    pub(crate) async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-API-Key",
            HeaderValue::from_str(self.core.api_key()).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        debug!("GET {} (async)", path);

        let response = self
            .client
            .request(Method::GET, path)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        debug!("Response status: {}", status);

        if !status.is_success() {
            return Err(self.error_from_status(status, response).await);
        }

        let result = response.json::<T>().await?;
        Ok(result)
    }

    /// Convert HTTP status and response into an appropriate Error (async)
    async fn error_from_status(&self, status: StatusCode, response: reqwest::Response) -> Error {
        match status {
            StatusCode::UNAUTHORIZED => Error::Unauthorized,
            StatusCode::FORBIDDEN => Error::Forbidden,
            StatusCode::NOT_FOUND => Error::NotFound,
            StatusCode::METHOD_NOT_ALLOWED => Error::MethodNotAllowed,
            StatusCode::TOO_MANY_REQUESTS => {
                // Parse Retry-After header if present
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| {
                        // Try parsing as delay-seconds (numeric)
                        if let Ok(seconds) = s.parse::<u64>() {
                            return Some(seconds);
                        }

                        // Try parsing as HTTP-date
                        if let Ok(date) = httpdate::parse_http_date(s) {
                            if let Ok(duration) = date.duration_since(std::time::SystemTime::now())
                            {
                                return Some(duration.as_secs());
                            }
                        }

                        None
                    });

                Error::RateLimited { retry_after }
            }
            _ => {
                // Try to parse error response
                if let Ok(body) = response.text().await {
                    if let Ok(api_errors) = serde_json::from_str::<ApiErrors>(&body) {
                        return Error::Fault {
                            code: status,
                            errors: api_errors,
                        };
                    }
                }
                // Fallback: create a Fault error with empty errors
                Error::Fault {
                    code: status,
                    errors: ApiErrors {
                        errors: vec![],
                        error_messages: vec![],
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_async_client_creation() {
        let client = JobsucheAsync::new(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            Credentials::default(),
        )
        .await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_async_client_with_config() {
        let config = ClientConfig {
            timeout: Duration::from_secs(10),
            max_retries: 2,
            retry_enabled: false,
            ..Default::default()
        };

        let client = JobsucheAsync::with_config(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            Credentials::default(),
            config,
        )
        .await;
        assert!(client.is_ok());
    }
}
