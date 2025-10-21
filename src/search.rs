//! Job search functionality

use tracing::debug;

use crate::sync::Jobsuche;
use crate::{JobSearchResponse, Result, SearchOptions};

/// Search interface for finding jobs
///
/// This interface provides methods to search for jobs using the Jobsuche API.
/// It supports rich filtering, pagination, and iteration over results.
#[derive(Debug)]
pub struct Search {
    client: Jobsuche,
}

impl Search {
    pub(crate) fn new(client: &Jobsuche) -> Search {
        Search {
            client: client.clone(),
        }
    }

    /// Perform a job search with the given options
    ///
    /// Returns a single page of job search results. Use pagination parameters
    /// (page, size) in SearchOptions to retrieve different pages.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jobsuche::{Jobsuche, Credentials, SearchOptions, Arbeitszeit};
    ///
    /// let client = Jobsuche::new(
    ///     "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
    ///     Credentials::default()
    /// ).unwrap();
    ///
    /// let results = client.search().list(SearchOptions::builder()
    ///     .was("Softwareentwickler")
    ///     .wo("Berlin")
    ///     .umkreis(50)
    ///     .arbeitszeit(vec![Arbeitszeit::Vollzeit])
    ///     .page(1)
    ///     .size(25)
    ///     .build()
    /// ).unwrap();
    ///
    /// println!("Found {} jobs", results.stellenangebote.len());
    /// for job in &results.stellenangebote {
    ///     println!("- {}: {}", job.refnr, job.beruf);
    /// }
    /// ```
    pub fn list(&self, options: SearchOptions) -> Result<JobSearchResponse> {
        let mut path = self.client.core.path(&["pc", "v4", "jobs"]);

        if let Some(query) = options.serialize() {
            path.push('?');
            path.push_str(&query);
        }

        debug!("Searching jobs with path: {}", path);

        self.client.get(&path)
    }

    /// Search with automatic pagination, yielding all results
    ///
    /// This method automatically handles pagination by making multiple requests
    /// to retrieve all matching jobs. Use with caution for broad searches.
    ///
    /// # Note
    ///
    /// - The API has a maximum result limit (typically around 100 per page)
    /// - Some searches may return thousands of results
    /// - Consider using filters to narrow down results
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
    /// let all_jobs = client.search().iter(SearchOptions::builder()
    ///     .was("Rust Developer")
    ///     .wo("Deutschland")
    ///     .veroeffentlichtseit(7)
    ///     .build()
    /// ).unwrap();
    ///
    /// println!("Found {} total jobs", all_jobs.len());
    /// ```
    pub fn iter(&self, options: SearchOptions) -> Result<Vec<crate::JobListing>> {
        let mut all_jobs = Vec::new();
        let mut page = 1u64;
        let size = options.size().unwrap_or(50);

        loop {
            let page_options = options.as_builder().page(page).size(size).build();

            let results = self.list(page_options)?;

            let jobs_count = results.stellenangebote.len();
            all_jobs.extend(results.stellenangebote);

            // Stop if we got fewer results than requested (last page)
            if jobs_count < size as usize {
                break;
            }

            // Check if we've reached the maximum results
            if let Some(max) = results.max_ergebnisse {
                if all_jobs.len() >= max as usize {
                    break;
                }
            }

            page += 1;

            // Safety limit to prevent infinite loops
            if page > 1000 {
                debug!("Reached safety limit of 1000 pages");
                break;
            }
        }

        Ok(all_jobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_creation() {
        let client = Jobsuche::new(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            crate::Credentials::default(),
        )
        .unwrap();

        let search = client.search();
        assert!(format!("{:?}", search).contains("Search"));
    }
}
