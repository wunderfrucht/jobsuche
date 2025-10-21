//! Rust interface for the Bundesagentur für Arbeit Jobsuche API
//!
//! This crate provides a client for interacting with Germany's Federal Employment Agency
//! (Bundesagentur für Arbeit) job search API. It allows you to search for jobs, get detailed
//! job information, and access employer logos.
//!
//! # Features
//!
//! - 🔍 **Job Search**: Search for jobs with rich filtering options (location, job title, employment type, etc.)
//! - 📄 **Job Details**: Get comprehensive information about specific job postings
//! - 🏢 **Employer Logos**: Download employer logos when available
//! - 🔄 **Pagination**: Automatic pagination support for large result sets (with lazy iteration)
//! - 🦀 **Type-Safe**: Strongly typed API with enums for all parameters
//! - ⚡ **Sync & Async**: Both synchronous and asynchronous clients (async with `async` feature flag)
//! - 🔁 **Retry Logic**: Automatic retry with exponential backoff for transient failures
//! - ⏱️ **Timeouts**: Configurable request and connection timeouts (default: 30s/10s)
//!
//! # Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! jobsuche = "0.1"
//! ```
//!
//! ## Basic Usage
//!
//! ```no_run
//! use jobsuche::{Jobsuche, Credentials, SearchOptions, Arbeitszeit};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with the default API key
//! let client = Jobsuche::new(
//!     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
//!     Credentials::default()
//! )?;
//!
//! // Search for jobs
//! let results = client.search().list(SearchOptions::builder()
//!     .was("Softwareentwickler")           // Job title
//!     .wo("Berlin")                        // Location
//!     .umkreis(50)                         // 50km radius
//!     .arbeitszeit(vec![Arbeitszeit::Vollzeit])  // Full-time only
//!     .veroeffentlichtseit(30)             // Posted in last 30 days
//!     .size(25)                            // 25 results per page
//!     .build()
//! )?;
//!
//! println!("Found {} jobs", results.stellenangebote.len());
//!
//! // Get details for a specific job
//! if let Some(job) = results.stellenangebote.first() {
//!     let details = client.job_details(&job.refnr)?;
//!     if let Some(title) = &details.titel {
//!         println!("Job: {}", title);
//!     }
//!     println!("Description: {:?}", details.stellenbeschreibung);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Type-Safe Filters
//!
//! ```no_run
//! use jobsuche::{SearchOptions, Arbeitszeit, Befristung, Angebotsart};
//!
//! let options = SearchOptions::builder()
//!     .was("Data Scientist")
//!     .wo("München")
//!     .angebotsart(Angebotsart::Arbeit)           // Regular employment
//!     .befristung(vec![Befristung::Unbefristet])  // Permanent contract
//!     .arbeitszeit(vec![
//!         Arbeitszeit::Vollzeit,
//!         Arbeitszeit::Teilzeit,
//!     ])
//!     .build();
//! ```
//!
//! ## Pagination
//!
//! ```no_run
//! use jobsuche::{Jobsuche, Credentials, SearchOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Jobsuche::new(
//!     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
//!     Credentials::default()
//! )?;
//!
//! // Manual pagination
//! let page1 = client.search().list(SearchOptions::builder()
//!     .was("Rust Developer")
//!     .page(1)
//!     .size(50)
//!     .build()
//! )?;
//!
//! // Automatic pagination - get all results
//! let all_jobs = client.search().iter(SearchOptions::builder()
//!     .was("Rust Developer")
//!     .veroeffentlichtseit(7)  // Last 7 days only (to limit results)
//!     .build()
//! )?;
//!
//! println!("Found {} total jobs", all_jobs.len());
//! # Ok(())
//! # }
//! ```
//!
//! # API Quirks & Known Issues
//!
//! Based on analysis of GitHub issues, be aware of:
//!
//! 1. **404 Errors**: Job details may return 404 even if the job appears in search results (jobs expire quickly)
//! 2. **Rate Limiting**: API may return 429 (Too Many Requests) - client automatically retries with exponential backoff
//! 3. **403 Errors**: Sporadic temporary blocks may occur
//! 4. **Employer Search**: Case-sensitive and exact-match only ("Deutsche Bahn AG" works, "bahn" doesn't)
//! 5. **Employer Logos**: Many employers don't have logos (expect 404s)
//! 6. **No Sorting**: Results are sorted oldest-to-newest, no way to change this
//!
//! # Rate Limiting
//!
//! The client handles rate limiting automatically:
//! - Detects 429 (Too Many Requests) responses
//! - Respects `Retry-After` header when present
//! - Falls back to exponential backoff if no `Retry-After` header
//! - Configurable retry attempts (default: 3)
//!
//! # Features
//!
//! - `async`: Enable asynchronous client (requires tokio runtime)
//! - `cache`: Enable response caching
//! - `metrics`: Enable performance metrics collection
//! - `full`: Enable all features

pub mod builder;
pub mod core;
mod errors;
pub mod pagination;
pub mod rep;
pub mod search;
pub mod sync;

#[cfg(feature = "async")]
pub mod async_client;

// Re-export main types for convenience
pub use builder::{SearchOptions, SearchOptionsBuilder};
pub use core::{decode_refnr, encode_refnr, ClientCore, Credentials};
pub use errors::{ApiErrors, Error, Result};
pub use rep::{
    Address, Angebotsart, Arbeitszeit, Befristung, Coordinates, Facet, FacetData, JobDetails,
    JobListing, JobSearchResponse, LeadershipSkills, Mobility, Skill, WorkLocation,
};
pub use search::Search;
pub use sync::{ClientConfig, Jobsuche};

#[cfg(feature = "async")]
pub use async_client::JobsucheAsync;
#[cfg(feature = "async")]
pub use search::SearchAsync;

// Re-export tracing for users who want logging
pub use tracing;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let client = Jobsuche::new(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            Credentials::default(),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_search_options() {
        let options = SearchOptions::builder()
            .was("Developer")
            .wo("Berlin")
            .size(10)
            .build();

        let query = options.serialize();
        assert!(query.is_some());
        let query_str = query.unwrap();
        assert!(query_str.contains("was=Developer"));
        assert!(query_str.contains("wo=Berlin"));
        assert!(query_str.contains("size=10"));
    }

    #[test]
    fn test_refnr_encoding() {
        let refnr = "10001-1001601666-S";
        let encoded = encode_refnr(refnr);
        assert_eq!(encoded, "MTAwMDEtMTAwMTYwMTY2Ni1T");

        let decoded = decode_refnr(&encoded).unwrap();
        assert_eq!(decoded, refnr);
    }
}
