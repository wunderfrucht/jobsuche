//! Synchronous client for the Jobsuche API

use std::io::Read;
use std::thread;
use std::time::Duration;
use tracing::{debug, warn};

use backon::{BackoffBuilder, ExponentialBuilder};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;

use crate::core::{encode_refnr, ClientCore};
use crate::search::Search;
use crate::{ApiErrors, Credentials, Error, JobDetails, Result};

/// Configuration for the Jobsuche client
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Request timeout (default: 30 seconds)
    pub timeout: Duration,
    /// Connection timeout (default: 10 seconds)
    pub connect_timeout: Duration,
    /// Maximum number of retry attempts (default: 3)
    pub max_retries: u32,
    /// Enable retry logic for transient errors (default: true)
    pub retry_enabled: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_enabled: true,
        }
    }
}

/// Synchronous Jobsuche API client
///
/// This is the main entry point for interacting with the Jobsuche API
/// using synchronous/blocking requests.
///
/// # Example
///
/// ```no_run
/// use jobsuche::{Jobsuche, Credentials, SearchOptions};
///
/// let client = Jobsuche::new(
///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
///     Credentials::default()
/// ).unwrap();
///
/// // Search for jobs
/// let results = client.search()
///     .list(SearchOptions::builder()
///         .was("Softwareentwickler")
///         .wo("Berlin")
///         .size(10)
///         .build())
///     .unwrap();
///
/// println!("Found {} jobs", results.stellenangebote.len());
/// ```
#[derive(Clone, Debug)]
pub struct Jobsuche {
    pub(crate) core: ClientCore,
    client: Client,
    config: ClientConfig,
}

impl Jobsuche {
    /// Creates a new instance of the Jobsuche client with default configuration
    ///
    /// # Arguments
    ///
    /// * `host` - Base URL of the API (typically "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service")
    /// * `credentials` - Authentication credentials (use `Credentials::default()` for the public API key)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials};
    ///
    /// let client = Jobsuche::new(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default()
    /// ).unwrap();
    /// ```
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Jobsuche>
    where
        H: Into<String>,
    {
        Self::with_config(host, credentials, ClientConfig::default())
    }

    /// Creates a new instance with custom configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials, ClientConfig};
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig {
    ///     timeout: Duration::from_secs(60),
    ///     max_retries: 5,
    ///     ..Default::default()
    /// };
    ///
    /// let client = Jobsuche::with_config(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default(),
    ///     config
    /// ).unwrap();
    /// ```
    pub fn with_config<H>(
        host: H,
        credentials: Credentials,
        config: ClientConfig,
    ) -> Result<Jobsuche>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        let client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()?;

        Ok(Jobsuche {
            core,
            client,
            config,
        })
    }

    /// Creates a new instance using a custom reqwest client
    ///
    /// This is useful if you need to configure custom timeouts, proxies, or other
    /// HTTP client settings. Note: if using a custom client, timeout config will be ignored.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials, ClientConfig};
    /// use reqwest::blocking::Client;
    /// use std::time::Duration;
    ///
    /// let client = Client::builder()
    ///     .timeout(Duration::from_secs(30))
    ///     .build()
    ///     .unwrap();
    ///
    /// let jobsuche = Jobsuche::from_client(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default(),
    ///     client,
    ///     ClientConfig::default()
    /// ).unwrap();
    /// ```
    pub fn from_client<H>(
        host: H,
        credentials: Credentials,
        client: Client,
        config: ClientConfig,
    ) -> Result<Jobsuche>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        Ok(Jobsuche {
            core,
            client,
            config,
        })
    }

    /// Creates a client instance directly from an existing ClientCore
    ///
    /// This is useful for converting between sync and async clients.
    pub fn with_core(core: ClientCore) -> Result<Jobsuche> {
        Self::with_config_and_core(core, ClientConfig::default())
    }

    /// Creates a client instance from an existing ClientCore with custom config
    pub fn with_config_and_core(core: ClientCore, config: ClientConfig) -> Result<Jobsuche> {
        let client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .build()?;

        Ok(Jobsuche {
            core,
            client,
            config,
        })
    }

    /// Return search interface
    pub fn search(&self) -> Search {
        Search::new(self)
    }

    /// Get detailed information about a specific job
    ///
    /// # Arguments
    ///
    /// * `refnr` - The reference number of the job (e.g., "10001-1001601666-S")
    ///
    /// # Known Issues
    ///
    /// - Jobs may return 404 even if they appear in search results (Issue #61)
    /// - Reference numbers are base64-encoded for the API call
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials};
    ///
    /// let client = Jobsuche::new(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default()
    /// ).unwrap();
    ///
    /// let job = client.job_details("10001-1001601666-S").unwrap();
    /// if let Some(title) = &job.titel {
    ///     println!("Job title: {}", title);
    /// }
    /// ```
    pub fn job_details(&self, refnr: &str) -> Result<JobDetails> {
        let encoded = encode_refnr(refnr);
        let path = self.core.path(&["pc", "v4", "jobdetails", &encoded]);
        self.get(&path)
    }

    /// Get the logo of an employer
    ///
    /// Returns the raw PNG image bytes.
    ///
    /// # Arguments
    ///
    /// * `hash_id` - The employer hash ID (from job listing's `kundennummer_hash`)
    ///
    /// # Known Issues
    ///
    /// - Many employers don't have logos, resulting in 404 errors (Issue #62)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials};
    ///
    /// let client = Jobsuche::new(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default()
    /// ).unwrap();
    ///
    /// match client.employer_logo("VK2qoXBe0s-UAdH_qxLDRrZrY5iY8a1PJt3MjJCXsdo=") {
    ///     Ok(logo_bytes) => println!("Got logo: {} bytes", logo_bytes.len()),
    ///     Err(_) => println!("No logo available"),
    /// }
    /// ```
    pub fn employer_logo(&self, hash_id: &str) -> Result<Vec<u8>> {
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
            .send()?;

        let status = response.status();
        if !status.is_success() {
            return Err(self.error_from_status(status, response));
        }

        let bytes = response.bytes()?.to_vec();
        Ok(bytes)
    }

    /// Internal method to perform GET requests with retry logic
    pub(crate) fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if !self.config.retry_enabled {
            return self.get_once(path);
        }

        // Build exponential backoff strategy
        let backoff = ExponentialBuilder::default()
            .with_max_times(self.config.max_retries as usize)
            .with_max_delay(Duration::from_secs(60));

        let mut attempt = 0;
        let mut backoff_iter = backoff.build();

        loop {
            attempt += 1;
            debug!(
                "GET {} (attempt {}/{})",
                path,
                attempt,
                self.config.max_retries + 1
            );

            match self.get_once(path) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if error is retryable
                    let should_retry = matches!(
                        e,
                        Error::Http(_)
                            | Error::RateLimited { .. }
                            | Error::Fault {
                                code: StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT,
                                ..
                            }
                    );

                    if !should_retry || attempt > self.config.max_retries {
                        return Err(e);
                    }

                    // Handle rate limiting with Retry-After
                    if let Error::RateLimited {
                        retry_after: Some(seconds),
                    } = e
                    {
                        let duration = Duration::from_secs(seconds);
                        warn!(
                            "Rate limited, waiting {} seconds as requested by server (attempt {}/{})",
                            seconds, attempt, self.config.max_retries
                        );
                        thread::sleep(duration);
                    } else if let Some(duration) = backoff_iter.next() {
                        warn!(
                            "Request failed ({}), retrying in {:?}... (attempt {}/{})",
                            e, duration, attempt, self.config.max_retries
                        );
                        thread::sleep(duration);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Perform a single GET request without retry
    fn get_once<T>(&self, path: &str) -> Result<T>
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

        let response = self
            .client
            .request(Method::GET, path)
            .headers(headers)
            .send()?;

        let status = response.status();
        debug!("Response status: {}", status);

        if !status.is_success() {
            return Err(self.error_from_status(status, response));
        }

        let result = response.json::<T>()?;
        Ok(result)
    }

    /// Convert HTTP status and response into an appropriate Error
    fn error_from_status(
        &self,
        status: StatusCode,
        mut response: reqwest::blocking::Response,
    ) -> Error {
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
                let mut body = String::new();
                if response.read_to_string(&mut body).is_ok() {
                    if let Ok(api_errors) = serde_json::from_str::<ApiErrors>(&body) {
                        return Error::Fault {
                            code: status,
                            errors: api_errors,
                        };
                    }
                }
                // Fallback to generic HTTP error
                Error::Http(response.error_for_status().unwrap_err())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Jobsuche::new(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            Credentials::default(),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let client = Jobsuche::new("not a url", Credentials::default());
        assert!(client.is_err());
    }
}
