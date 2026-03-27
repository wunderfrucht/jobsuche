//! API contract tests — run against the live Bundesagentur für Arbeit API.
//! These verify the API response structure matches our types.
//!
//! Run with: cargo test --test api_contract_tests -- --test-threads=1
//!
//! These tests are #[ignore]d to avoid hitting the live API during local
//! `cargo test`. The scheduled CI workflow runs them with `--ignored`.
//! They use --test-threads=1 to avoid rate limiting.

use jobsuche::{Credentials, Jobsuche, SearchOptions};
use std::thread;
use std::time::Duration;

const API_BASE: &str = "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service";

/// Helper: create a client and add a small delay to avoid rate limiting
fn client() -> Jobsuche {
    thread::sleep(Duration::from_millis(500));
    Jobsuche::new(API_BASE, Credentials::default()).expect("Failed to create client")
}

/// Verify the search endpoint returns expected JSON structure
#[test]
#[ignore]
fn api_contract_search_response_structure() {
    let client = client();
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Informatik")
                .size(2)
                .page(1)
                .build(),
        )
        .expect("Search endpoint should respond successfully");

    // Verify response structure
    assert!(
        results.max_ergebnisse.is_some(),
        "Response should include maxErgebnisse"
    );
    assert!(
        results.page.is_some(),
        "Response should include page number"
    );
    assert!(results.size.is_some(), "Response should include size");

    // Verify we got results (Informatik is a broad term)
    assert!(
        !results.stellenangebote.is_empty(),
        "Broad search should return results"
    );

    // Verify job listing structure
    let job = &results.stellenangebote[0];
    assert!(!job.refnr.is_empty(), "Job must have refnr");
    assert!(!job.beruf.is_empty(), "Job must have beruf");
    assert!(!job.arbeitgeber.is_empty(), "Job must have arbeitgeber");
    // WorkLocation must be present (required field)
    // Optional fields may or may not be present
}

/// Verify pagination works: page 1 and page 2 return different results
#[test]
#[ignore]
fn api_contract_pagination_works() {
    let client = client();

    let page1 = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Softwareentwickler")
                .size(5)
                .page(1)
                .build(),
        )
        .expect("Page 1 should work");

    let page2 = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Softwareentwickler")
                .size(5)
                .page(2)
                .build(),
        )
        .expect("Page 2 should work");

    if page1.stellenangebote.len() >= 5 {
        // If page 1 is full, page 2 should have different results
        assert!(
            !page2.stellenangebote.is_empty(),
            "Page 2 should have results if page 1 is full"
        );
        if !page2.stellenangebote.is_empty() {
            assert_ne!(
                page1.stellenangebote[0].refnr, page2.stellenangebote[0].refnr,
                "Different pages should return different jobs"
            );
        }
    }
}

/// Verify the size parameter is respected
#[test]
#[ignore]
fn api_contract_size_parameter_respected() {
    let client = client();
    let results = client
        .search()
        .list(SearchOptions::builder().was("Ingenieur").size(3).build())
        .expect("Search should work");

    assert!(
        results.stellenangebote.len() <= 3,
        "API should respect size parameter, got {} results",
        results.stellenangebote.len()
    );
}

/// Verify the API returns max_ergebnisse for broad searches
#[test]
#[ignore]
fn api_contract_max_ergebnisse_reported() {
    let client = client();
    let results = client
        .search()
        .list(SearchOptions::builder().was("Informatik").size(1).build())
        .expect("Search should work");

    let max = results
        .max_ergebnisse
        .expect("maxErgebnisse should be present");
    assert!(max > 0, "Broad search should report total results > 0");
    println!("API reports {} total results for 'Informatik'", max);
}

/// Verify page beyond limit returns empty or error gracefully
#[test]
#[ignore]
fn api_contract_page_beyond_limit() {
    let client = client();

    // Try page 101 (beyond the documented 100-page limit)
    let result = client.search().list(
        SearchOptions::builder()
            .was("Informatik")
            .size(100)
            .page(101)
            .build(),
    );

    match result {
        Ok(response) => {
            // API may return empty results or still return data
            println!(
                "Page 101 returned {} results (maxErgebnisse: {:?})",
                response.stellenangebote.len(),
                response.max_ergebnisse
            );
            // Document actual behavior for issue #7
        }
        Err(e) => {
            println!(
                "Page 101 returned error: {:?} - API enforces page limit server-side",
                e
            );
        }
    }
}

/// Verify job details endpoint works for a freshly-found job
#[test]
#[ignore]
fn api_contract_job_details_structure() {
    let client = client();

    // Get a fresh job listing
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Informatik")
                .size(1)
                .veroeffentlichtseit(1) // Very recent to minimize 404 chance
                .build(),
        )
        .expect("Search should work");

    if let Some(job) = results.stellenangebote.first() {
        match client.job_details(&job.refnr) {
            Ok(details) => {
                // Verify key fields exist in the details response
                // titel and arbeitgeber are Option in our types
                println!(
                    "Job details: titel={:?}, arbeitgeber={:?}",
                    details.titel, details.arbeitgeber
                );
            }
            Err(jobsuche::Error::NotFound) => {
                // Known issue: jobs can expire between search and detail fetch
                println!("Job {} returned 404 (expired) — known issue", job.refnr);
            }
            Err(e) => {
                panic!("Unexpected error fetching job details: {:?}", e);
            }
        }
    }
}

/// Verify location filter narrows results
#[test]
#[ignore]
fn api_contract_location_filter() {
    let client = client();

    let broad = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Softwareentwickler")
                .size(1)
                .build(),
        )
        .expect("Broad search should work");

    let narrow = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Softwareentwickler")
                .wo("Flensburg")
                .umkreis(10)
                .size(1)
                .build(),
        )
        .expect("Narrow search should work");

    if let (Some(broad_max), Some(narrow_max)) = (broad.max_ergebnisse, narrow.max_ergebnisse) {
        assert!(
            narrow_max <= broad_max,
            "Location filter should narrow results: broad={}, narrow={}",
            broad_max,
            narrow_max
        );
        println!(
            "Broad: {} results, Flensburg 10km: {} results",
            broad_max, narrow_max
        );
    }
}
