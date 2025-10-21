//! Lazy pagination iterator for job search results
//!
//! This module provides a lazy iterator that fetches job results page-by-page,
//! avoiding loading all results into memory at once.

use tracing::debug;

use crate::sync::Jobsuche;
use crate::{JobListing, Result, SearchOptions};

/// A lazy iterator over job search results
///
/// This iterator fetches results page-by-page from the API, yielding individual
/// job listings one at a time. This is more memory-efficient than loading all
/// results at once.
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
/// let options = SearchOptions::builder()
///     .was("Rust Developer")
///     .wo("Deutschland")
///     .build();
///
/// // Process jobs one at a time without loading all into memory
/// for job in client.search().jobs(options).unwrap() {
///     match job {
///         Ok(job) => println!("Found: {}", job.beruf),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct JobIterator {
    client: Jobsuche,
    options: SearchOptions,
    current_page: u64,
    page_size: u64,
    current_page_jobs: Vec<JobListing>,
    current_index: usize,
    finished: bool,
    max_results: Option<u64>,
    total_yielded: u64,
}

impl JobIterator {
    /// Create a new lazy job iterator
    pub(crate) fn new(client: &Jobsuche, options: SearchOptions) -> Result<Self> {
        let page_size = options.size().unwrap_or(50);

        Ok(JobIterator {
            client: client.clone(),
            options,
            current_page: 0,
            page_size,
            current_page_jobs: Vec::new(),
            current_index: 0,
            finished: false,
            max_results: None,
            total_yielded: 0,
        })
    }

    /// Fetch the next page of results
    fn fetch_next_page(&mut self) -> Result<bool> {
        if self.finished {
            return Ok(false);
        }

        self.current_page += 1;

        // Safety limit
        if self.current_page > 1000 {
            debug!("Reached safety limit of 1000 pages");
            self.finished = true;
            return Ok(false);
        }

        let page_options = self
            .options
            .as_builder()
            .page(self.current_page)
            .size(self.page_size)
            .build();

        debug!("Fetching page {}", self.current_page);

        let response = self.client.search().list(page_options)?;

        // Store max_results from first page
        if self.current_page == 1 {
            self.max_results = response.max_ergebnisse;
        }

        let jobs_count = response.stellenangebote.len();
        self.current_page_jobs = response.stellenangebote;
        self.current_index = 0;

        // Check if this is the last page
        if jobs_count < self.page_size as usize {
            self.finished = true;
        }

        // Check if we've reached max_results
        if let Some(max) = self.max_results {
            if self.total_yielded >= max {
                self.finished = true;
            }
        }

        Ok(jobs_count > 0)
    }
}

impl Iterator for JobIterator {
    type Item = Result<JobListing>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have jobs in the current page, return the next one
            if self.current_index < self.current_page_jobs.len() {
                let job = self.current_page_jobs[self.current_index].clone();
                self.current_index += 1;
                self.total_yielded += 1;
                return Some(Ok(job));
            }

            // If we're finished, we're done
            if self.finished {
                return None;
            }

            // Otherwise, fetch the next page
            match self.fetch_next_page() {
                Ok(true) => continue,     // Successfully fetched, loop will return first job
                Ok(false) => return None, // No more pages
                Err(e) => return Some(Err(e)), // Error fetching page
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[test]
    fn test_iterator_creation() {
        let client = Jobsuche::new(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
            Credentials::default(),
        )
        .unwrap();

        let options = SearchOptions::builder().was("test").build();
        let iterator = JobIterator::new(&client, options);
        assert!(iterator.is_ok());
    }
}
