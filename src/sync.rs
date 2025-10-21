//! Synchronous client for the Jobsuche API

use std::io::Read;
use tracing::debug;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;

use crate::core::{encode_refnr, ClientCore};
use crate::search::Search;
use crate::{ApiErrors, Credentials, Error, JobDetails, Result};

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
}

impl Jobsuche {
    /// Creates a new instance of the Jobsuche client
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
        let core = ClientCore::new(host, credentials)?;
        Ok(Jobsuche {
            core,
            client: Client::new(),
        })
    }

    /// Creates a new instance using a custom reqwest client
    ///
    /// This is useful if you need to configure custom timeouts, proxies, or other
    /// HTTP client settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials};
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
    ///     client
    /// ).unwrap();
    /// ```
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jobsuche>
    where
        H: Into<String>,
    {
        let core = ClientCore::new(host, credentials)?;
        Ok(Jobsuche { core, client })
    }

    /// Creates a client instance directly from an existing ClientCore
    ///
    /// This is useful for converting between sync and async clients.
    pub fn with_core(core: ClientCore) -> Result<Jobsuche> {
        Ok(Jobsuche {
            core,
            client: Client::new(),
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

    /// Internal method to perform GET requests
    pub(crate) fn get<T>(&self, path: &str) -> Result<T>
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

        debug!("GET {}", path);

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
