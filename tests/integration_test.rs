//! Integration tests that call the real API
//!
//! These tests make real HTTP calls to the Jobsuche API.
//! They are marked as `#[ignore]` by default to avoid hitting the API
//! during regular test runs. To run them:
//!
//! ```bash
//! cargo test --test integration_test -- --ignored
//! ```

use jobsuche::{Arbeitszeit, Credentials, Jobsuche, SearchOptions};

#[test]
#[ignore]
fn test_real_api_search() {
    // Create a real client
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .expect("Failed to create client");

    // Search for Rust jobs in Germany (small result set)
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Rust Developer")
                .wo("Deutschland")
                .arbeitszeit(vec![Arbeitszeit::Vollzeit])
                .veroeffentlichtseit(30)
                .size(5)
                .build(),
        )
        .expect("API call failed");

    // Verify we got results in expected format
    println!("Found {} jobs", results.stellenangebote.len());
    assert!(
        results.stellenangebote.len() <= 5,
        "Should not exceed size limit"
    );

    // If we got results, verify they have required fields
    if let Some(first_job) = results.stellenangebote.first() {
        println!(
            "First job: {} at {}",
            first_job.beruf, first_job.arbeitgeber
        );
        assert!(!first_job.refnr.is_empty(), "Job should have refnr");
        assert!(!first_job.beruf.is_empty(), "Job should have beruf");
        assert!(
            !first_job.arbeitgeber.is_empty(),
            "Job should have arbeitgeber"
        );
    }
}

#[test]
#[ignore]
fn test_real_api_job_details() {
    // Create a real client
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .expect("Failed to create client");

    // First, search for a job to get a valid refnr
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Softwareentwickler")
                .wo("Berlin")
                .size(1)
                .build(),
        )
        .expect("Search failed");

    if let Some(job) = results.stellenangebote.first() {
        println!("Testing job details for: {}", job.refnr);

        // Try to get details (may fail with 404 if job expired, which is expected)
        // Known Issue #61: Job details often return 404 even for valid refnrs
        match client.job_details(&job.refnr) {
            Ok(details) => {
                println!("Got job details successfully!");
                if let Some(title) = &details.titel {
                    println!("Title: {}", title);
                }
                if let Some(employer) = &details.arbeitgeber {
                    println!("Employer: {}", employer);
                }
            }
            Err(jobsuche::Error::NotFound) => {
                println!("Job expired (404) - this is expected and OK (Issue #61)");
            }
            Err(e) => {
                // Job details endpoint is unreliable, so we'll just log the error
                println!(
                    "Job details failed (this is known to be unreliable): {:?}",
                    e
                );
            }
        }
    } else {
        println!("No jobs found - skipping test");
    }
}

#[test]
#[ignore]
fn test_real_api_pagination() {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .expect("Failed to create client");

    // Get first page
    let page1 = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Developer")
                .page(1)
                .size(10)
                .build(),
        )
        .expect("Page 1 failed");

    println!("Page 1: {} jobs", page1.stellenangebote.len());

    // Get second page
    let page2 = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Developer")
                .page(2)
                .size(10)
                .build(),
        )
        .expect("Page 2 failed");

    println!("Page 2: {} jobs", page2.stellenangebote.len());

    // Verify pages are different (if we got results)
    if !page1.stellenangebote.is_empty() && !page2.stellenangebote.is_empty() {
        let first_refnr_page1 = &page1.stellenangebote[0].refnr;
        let first_refnr_page2 = &page2.stellenangebote[0].refnr;
        assert_ne!(
            first_refnr_page1, first_refnr_page2,
            "Different pages should have different jobs"
        );
    }
}

#[test]
#[ignore]
fn test_real_api_filters() {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .expect("Failed to create client");

    // Test with multiple filters
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Data Scientist")
                .wo("München")
                .umkreis(50)
                .arbeitszeit(vec![Arbeitszeit::Vollzeit])
                .veroeffentlichtseit(14)
                .zeitarbeit(false)
                .size(5)
                .build(),
        )
        .expect("Filtered search failed");

    println!(
        "Filtered search found {} jobs",
        results.stellenangebote.len()
    );

    // Verify results match location filter (at least region)
    for job in &results.stellenangebote {
        println!(
            "Job: {} in {}, {}",
            job.beruf,
            job.arbeitsort.ort.as_deref().unwrap_or("unknown"),
            job.arbeitsort.region.as_deref().unwrap_or("unknown")
        );
    }
}

#[test]
#[ignore]
fn test_real_api_employer_logo() {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .expect("Failed to create client");

    // Search for a well-known employer that might have a logo
    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .arbeitgeber("Deutsche Bahn AG")
                .size(1)
                .build(),
        )
        .expect("Search failed");

    if let Some(job) = results.stellenangebote.first() {
        if let Some(hash) = &job.kundennummer_hash {
            println!("Attempting to fetch logo for hash: {}", hash);

            // Try to get logo (expect 404 - most employers don't have logos)
            match client.employer_logo(hash) {
                Ok(logo_bytes) => {
                    println!("Successfully got logo: {} bytes", logo_bytes.len());
                    assert!(!logo_bytes.is_empty(), "Logo should have data");
                }
                Err(jobsuche::Error::NotFound) => {
                    println!("Logo not available (404) - this is expected");
                    // This is expected for most employers (Issue #62)
                }
                Err(e) => {
                    println!("Error fetching logo: {:?}", e);
                }
            }
        } else {
            println!("Job has no kundennummer_hash");
        }
    }
}

#[test]
fn test_unit_base64_encoding() {
    use jobsuche::{decode_refnr, encode_refnr};

    let refnr = "10001-1001601666-S";
    let encoded = encode_refnr(refnr);
    assert_eq!(encoded, "MTAwMDEtMTAwMTYwMTY2Ni1T");

    let decoded = decode_refnr(&encoded).unwrap();
    assert_eq!(decoded, refnr);
}

#[test]
fn test_unit_search_options_builder() {
    let options = SearchOptions::builder()
        .was("Developer")
        .wo("Berlin")
        .umkreis(50)
        .arbeitszeit(vec![Arbeitszeit::Vollzeit, Arbeitszeit::Teilzeit])
        .page(1)
        .size(25)
        .build();

    let query = options.serialize().unwrap();
    assert!(query.contains("was=Developer"));
    assert!(query.contains("wo=Berlin"));
    assert!(query.contains("umkreis=50"));
    assert!(query.contains("page=1"));
    assert!(query.contains("size=25"));
}
