//! Unit tests using mocked API responses
//!
//! These tests use mockito to simulate API responses without making real HTTP calls.
//! They run quickly and don't require network access.

use jobsuche::{Arbeitszeit, ClientConfig, Credentials, Jobsuche, SearchOptions};
use mockito::Server;
use std::time::Duration;

#[test]
fn test_search_with_mock() {
    let mut server = Server::new();

    let mock_response = r#"{
        "stellenangebote": [
            {
                "refnr": "10001-TEST123-S",
                "beruf": "Rust Developer",
                "arbeitgeber": "Test Company GmbH",
                "aktuelleVeroeffentlichungsdatum": "2025-10-20",
                "arbeitsort": {
                    "ort": "Berlin",
                    "region": "Berlin",
                    "plz": "10115"
                }
            },
            {
                "refnr": "10001-TEST456-S",
                "beruf": "Senior Rust Engineer",
                "arbeitgeber": "Another Corp",
                "aktuelleVeroeffentlichungsdatum": "2025-10-19",
                "arbeitsort": {
                    "ort": "München",
                    "region": "Bayern",
                    "plz": "80331"
                }
            }
        ],
        "maxErgebnisse": 2,
        "page": 1,
        "size": 10
    }"#;

    // Note: mockito matches the path component, query params can be in any order
    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*was=Rust.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Rust Developer")
                .size(10)
                .build(),
        )
        .unwrap();

    assert_eq!(results.stellenangebote.len(), 2);
    assert_eq!(results.stellenangebote[0].refnr, "10001-TEST123-S");
    assert_eq!(
        results.stellenangebote[0].beruf,
        Some("Rust Developer".to_string())
    );
    assert_eq!(
        results.stellenangebote[1].beruf,
        Some("Senior Rust Engineer".to_string())
    );
}

#[test]
fn test_search_with_filters_mock() {
    let mut server = Server::new();

    let mock_response = r#"{
        "stellenangebote": [
            {
                "refnr": "10001-FULLTIME-S",
                "beruf": "Backend Developer",
                "arbeitgeber": "Test GmbH",
                "arbeitsort": {
                    "ort": "Hamburg",
                    "region": "Hamburg"
                }
            }
        ],
        "maxErgebnisse": 1
    }"#;

    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*was=Developer.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("Developer")
                .wo("Hamburg")
                .umkreis(50)
                .arbeitszeit(vec![Arbeitszeit::Vollzeit])
                .build(),
        )
        .unwrap();

    assert_eq!(results.stellenangebote.len(), 1);
    assert_eq!(
        results.stellenangebote[0].beruf,
        Some("Backend Developer".to_string())
    );
}

#[test]
fn test_404_error_handling() {
    let mut server = Server::new();

    // Mock any base64-encoded refnr path to return 404
    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobdetails/.*".to_string()),
        )
        .with_status(404)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let result = client.job_details("testref"); // Will be base64 encoded

    // We expect a NotFound error
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::NotFound));
}

#[test]
fn test_401_unauthorized() {
    let mut server = Server::new();

    let _m = server.mock("GET", "/pc/v4/jobs").with_status(401).create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let result = client.search().list(SearchOptions::default());

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::Unauthorized));
}

#[test]
fn test_empty_results() {
    let mut server = Server::new();

    let mock_response = r#"{
        "stellenangebote": [],
        "maxErgebnisse": 0,
        "page": 1,
        "size": 10
    }"#;

    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*was=NonexistentJob.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("NonexistentJob")
                .size(10)
                .build(),
        )
        .unwrap();

    assert_eq!(results.stellenangebote.len(), 0);
    assert_eq!(results.max_ergebnisse, Some(0));
}

#[test]
fn test_job_details_mock() {
    let mut server = Server::new();

    let mock_response = r#"{
        "referenznummer": "10001-DETAILS-S",
        "stellenangebotsTitel": "Senior Rust Developer",
        "firma": "Test Company",
        "stellenangebotsBeschreibung": "Looking for an experienced Rust developer...",
        "hauptberuf": "Softwareentwickler/in",
        "stellenlokationen": [
            {
                "adresse": {
                    "ort": "Berlin",
                    "plz": "10115",
                    "region": "Berlin",
                    "land": "Deutschland"
                },
                "breite": 52.52,
                "laenge": 13.40
            }
        ],
        "arbeitszeitVollzeit": true,
        "verguetungsangabe": "KEINE_ANGABEN"
    }"#;

    // The refnr will be base64 encoded
    let encoded_ref = "MTAwMDEtREVUQUlMUy1T"; // base64("10001-DETAILS-S")

    let _m = server
        .mock("GET", format!("/pc/v4/jobdetails/{}", encoded_ref).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let details = client.job_details("10001-DETAILS-S").unwrap();

    assert_eq!(details.refnr, Some("10001-DETAILS-S".to_string()));
    assert_eq!(details.titel, Some("Senior Rust Developer".to_string()));
    assert_eq!(details.arbeitgeber, Some("Test Company".to_string()));
    assert_eq!(
        details.stellenbeschreibung,
        Some("Looking for an experienced Rust developer...".to_string())
    );
    assert_eq!(
        details.hauptberuf,
        Some("Softwareentwickler/in".to_string())
    );
    assert_eq!(details.arbeitsorte.len(), 1);
}

#[test]
fn test_pagination_mock() {
    let mut server = Server::new();

    // Page 1
    let page1_response = r#"{
        "stellenangebote": [
            {"refnr": "1", "beruf": "Job 1", "arbeitgeber": "Co 1", "arbeitsort": {"ort": "Berlin"}},
            {"refnr": "2", "beruf": "Job 2", "arbeitgeber": "Co 2", "arbeitsort": {"ort": "Berlin"}}
        ],
        "maxErgebnisse": 4,
        "page": 1,
        "size": 2
    }"#;

    // Page 2
    let page2_response = r#"{
        "stellenangebote": [
            {"refnr": "3", "beruf": "Job 3", "arbeitgeber": "Co 3", "arbeitsort": {"ort": "Berlin"}},
            {"refnr": "4", "beruf": "Job 4", "arbeitgeber": "Co 4", "arbeitsort": {"ort": "Berlin"}}
        ],
        "maxErgebnisse": 4,
        "page": 2,
        "size": 2
    }"#;

    // Page 3 - empty (signals end of results)
    let page3_response = r#"{
        "stellenangebote": [],
        "maxErgebnisse": 4,
        "page": 3,
        "size": 2
    }"#;

    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page1_response)
        .create();

    let _m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=2.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page2_response)
        .create();

    let _m3 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=3.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page3_response)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs = client
        .search()
        .iter(SearchOptions::builder().size(2).build())
        .unwrap();

    assert_eq!(all_jobs.len(), 4);
    assert_eq!(all_jobs[0].refnr, "1");
    assert_eq!(all_jobs[3].refnr, "4");
}

#[test]
fn test_timeout_configuration() {
    use jobsuche::ClientConfig;
    use std::time::Duration;

    let config = ClientConfig {
        timeout: Duration::from_secs(5),
        connect_timeout: Duration::from_secs(2),
        max_retries: 2,
        retry_enabled: true,
    };

    let server = Server::new();
    let client = Jobsuche::with_config(server.url(), Credentials::default(), config);

    assert!(client.is_ok());
}

#[test]
fn test_retry_disabled() {
    use jobsuche::ClientConfig;

    let config = ClientConfig {
        retry_enabled: false,
        ..Default::default()
    };

    let server = Server::new();
    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    // Just verify the client was created with retry disabled
    // (actual retry behavior is tested in integration tests)
    assert!(format!("{:?}", client).contains("ClientConfig"));
}

#[test]
fn test_rate_limit_429_detection() {
    let mut server = Server::new();

    // Return 429 with Retry-After header
    let _m = server
        .mock("GET", "/pc/v4/jobs")
        .with_status(429)
        .with_header("Retry-After", "60")
        .create();

    let config = ClientConfig {
        max_retries: 0, // Don't retry, just check error detection
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.search().list(SearchOptions::default());

    // Should detect rate limit error with Retry-After
    assert!(result.is_err());
    match result.unwrap_err() {
        jobsuche::Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(60), "Should parse Retry-After header");
        }
        other => panic!("Expected RateLimited error, got: {:?}", other),
    }
}

#[test]
fn test_rate_limit_429_without_retry_after() {
    let mut server = Server::new();

    // Return 429 without Retry-After header
    let _m = server.mock("GET", "/pc/v4/jobs").with_status(429).create();

    let config = ClientConfig {
        max_retries: 0, // Don't retry, just check error detection
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.search().list(SearchOptions::default());

    // Should detect rate limit error without Retry-After
    assert!(result.is_err());
    match result.unwrap_err() {
        jobsuche::Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, None, "Should have no Retry-After header");
        }
        other => panic!("Expected RateLimited error, got: {:?}", other),
    }
}

#[test]
fn test_rate_limit_error_display() {
    use jobsuche::Error;

    let error = Error::RateLimited {
        retry_after: Some(60),
    };

    let display = format!("{}", error);
    assert!(display.contains("Rate limited"));
    assert!(display.contains("60"));

    let error_no_retry = Error::RateLimited { retry_after: None };
    let display_no_retry = format!("{}", error_no_retry);
    assert!(display_no_retry.contains("Rate limited"));
}

#[test]
fn test_from_client() {
    use reqwest::blocking::Client;

    let custom_client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let server = Server::new();
    let client = Jobsuche::from_client(
        server.url(),
        Credentials::default(),
        custom_client,
        ClientConfig::default(),
    );

    assert!(client.is_ok());
}

#[test]
fn test_with_core() {
    use jobsuche::core::ClientCore;

    let core = ClientCore::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .unwrap();

    let client = Jobsuche::with_core(core);
    assert!(client.is_ok());
}

#[test]
fn test_with_config_and_core() {
    use jobsuche::core::ClientCore;

    let core = ClientCore::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .unwrap();

    let config = ClientConfig {
        timeout: Duration::from_secs(20),
        connect_timeout: Duration::from_secs(5),
        max_retries: 2,
        retry_enabled: true,
    };

    let client = Jobsuche::with_config_and_core(core, config);
    assert!(client.is_ok());
}

#[test]
fn test_403_forbidden() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(403)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::Forbidden));
}

#[test]
fn test_405_method_not_allowed() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(405)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        jobsuche::Error::MethodNotAllowed
    ));
}

#[test]
fn test_500_server_error_with_api_errors() {
    let mut server = Server::new();

    let error_response = r#"{
        "errors": [
            {
                "code": "INTERNAL_ERROR",
                "message": "Internal server error"
            }
        ],
        "errorMessages": ["Internal server error"]
    }"#;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(error_response)
        .create();

    let config = ClientConfig {
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());

    // Should get either Fault or Http error
    assert!(matches!(
        result.unwrap_err(),
        jobsuche::Error::Fault { .. } | jobsuche::Error::Http(_)
    ));
}

#[test]
fn test_500_server_error_plain_text() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create();

    let config = ClientConfig {
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());

    // Should get HTTP error for unparseable response
    assert!(matches!(result.unwrap_err(), jobsuche::Error::Http(_)));
}

#[test]
fn test_employer_logo_success() {
    let mut server = Server::new();

    // Mock PNG image (minimal valid PNG)
    let png_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width
        0x00, 0x00, 0x00, 0x01, // height
        0x08, 0x06, 0x00, 0x00, 0x00, // bit depth, color type, etc.
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk
        0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let _m = server
        .mock("GET", "/ed/v1/arbeitgeberlogo/test-hash")
        .with_status(200)
        .with_header("content-type", "image/png")
        .with_body(&png_bytes)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let logo = client.employer_logo("test-hash").unwrap();
    assert_eq!(logo.len(), png_bytes.len());
    assert_eq!(logo, png_bytes);
}

#[test]
fn test_employer_logo_not_found() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", "/ed/v1/arbeitgeberlogo/nonexistent")
        .with_status(404)
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let result = client.employer_logo("nonexistent");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::NotFound));
}

#[test]
fn test_search_interface() {
    let server = Server::new();
    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    // Just verify we can create a search interface
    let _search = client.search();
}

// --- Pagination boundary tests ---

/// Test that pagination stops after page 100 (API limit).
/// Pages 1-100 return exactly page_size results, page 101 should never be requested.
#[test]
fn test_pagination_stops_after_page_100() {
    let mut server = Server::new();

    // Use page_size=1 so each page returns exactly 1 job and the iterator keeps going.
    // A catch-all mock returns 1 result for every page request.
    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "R-1", "beruf": "Dev", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 200,
                "page": 1,
                "size": 1
            }"#,
        )
        .expect_at_most(100) // must not request page 101
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(1).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // Exactly 100 pages * 1 job per page = 100 jobs
    assert_eq!(all_jobs.len(), 100);
}

/// Test that page exactly 100 still returns results (should NOT stop).
/// Verifies the boundary: page 100 is the last allowed page.
#[test]
fn test_pagination_page_100_still_fetched() {
    let mut server = Server::new();

    // Pages 1-99 return 1 result each; page 100 returns 1 result (short page = last)
    // We use a catch-all that returns 1 result with page_size=1.
    // Since jobs_count (1) == page_size (1), the iterator won't stop due to short page.
    // But page 100 is <= 100, so it should be fetched.
    // After page 100, page 101 > 100 triggers the limit.
    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "R-1", "beruf": "Dev", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 999,
                "page": 1,
                "size": 1
            }"#,
        )
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(1).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // Page 100 is fetched (100 jobs total), page 101 is not
    assert_eq!(all_jobs.len(), 100);
}

/// Test that max_results is stored from page 1 only (not overwritten by page 2).
/// Page 1 returns maxErgebnisse=2, page 2 returns maxErgebnisse=999.
/// With correct code, max_results=2 (from page 1), so after yielding 2 jobs and
/// fetching page 2, total_yielded >= max_results stops further pages.
/// With mutant (`!= 1`): max_results is NOT stored from page 1, IS stored from
/// page 2 as 999, allowing page 3 to be fetched.
#[test]
fn test_pagination_max_results_stored_from_page1_only() {
    let mut server = Server::new();

    // Page 1: 2 jobs (full page), maxErgebnisse=2
    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "1", "beruf": "Job 1", "arbeitsort": {"ort": "Berlin"}},
                    {"refnr": "2", "beruf": "Job 2", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 2,
                "page": 1,
                "size": 2
            }"#,
        )
        .create();

    // Page 2: 2 jobs (full page), maxErgebnisse=999
    let _m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=2.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "3", "beruf": "Job 3", "arbeitsort": {"ort": "Berlin"}},
                    {"refnr": "4", "beruf": "Job 4", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 999,
                "page": 2,
                "size": 2
            }"#,
        )
        .create();

    // Page 3: should NOT be requested with correct code (max_results=2 from page 1
    // causes finished=true after page 2). With mutant it WOULD be requested.
    let m3 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=3.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [],
                "maxErgebnisse": 999,
                "page": 3,
                "size": 2
            }"#,
        )
        .expect(0) // page 3 must NOT be requested
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(2).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // Page 1 yields 2, page 2 yields 2 (total 4).
    // After page 2 fetch, total_yielded (2) >= max_results (2) → finished=true.
    // Page 2 jobs are still yielded, then iterator stops.
    assert_eq!(all_jobs.len(), 4);

    // Verify page 3 was never requested
    m3.assert();
}

/// Test that jobs_count == page_size means "continue" (not the last page).
/// With page_size=2 and exactly 2 results on page 1, it should fetch page 2.
#[test]
fn test_pagination_exact_page_size_continues() {
    let mut server = Server::new();

    // Page 1: exactly page_size (2) results
    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "1", "beruf": "Job 1", "arbeitsort": {"ort": "Berlin"}},
                    {"refnr": "2", "beruf": "Job 2", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 3,
                "page": 1,
                "size": 2
            }"#,
        )
        .create();

    // Page 2: fewer than page_size (1 result) -- last page
    let _m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=2.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "3", "beruf": "Job 3", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 3,
                "page": 2,
                "size": 2
            }"#,
        )
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(2).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // 2 from page 1 + 1 from page 2 = 3 total
    // If the mutant changed `<` to `>` on line 106, page 1 (2 results, 2 == page_size)
    // would be treated as the last page, and we'd only get 2 results.
    assert_eq!(all_jobs.len(), 3);
}

/// Test that jobs_count < page_size signals the last page (stops pagination).
#[test]
fn test_pagination_fewer_than_page_size_stops() {
    let mut server = Server::new();

    // Page 1: fewer than page_size results (1 < 2)
    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "1", "beruf": "Job 1", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 1,
                "page": 1,
                "size": 2
            }"#,
        )
        .create();

    // No page 2 mock -- it should never be requested

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(2).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // Only 1 result from page 1, pagination stopped because 1 < 2 (page_size)
    assert_eq!(all_jobs.len(), 1);
}

/// Test that an empty page (0 results) returns false from fetch_next_page.
/// The iterator should yield no results when the first page is empty.
#[test]
fn test_pagination_empty_page_returns_no_results() {
    let mut server = Server::new();

    // Page 1: 0 results
    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [],
                "maxErgebnisse": 0,
                "page": 1,
                "size": 2
            }"#,
        )
        .create();

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(2).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // 0 results, iterator should stop immediately
    // If the mutant changed `> 0` to `>= 0` on line 117, fetch_next_page would
    // return Ok(true) even with 0 results, causing an infinite loop.
    assert_eq!(all_jobs.len(), 0);
}

/// Test that an empty page mid-pagination stops the iterator.
/// Page 1 has results, page 2 returns empty -- should stop without requesting page 3.
#[test]
fn test_pagination_empty_page_mid_stream_stops() {
    let mut server = Server::new();

    // Page 1: full page (2 results = page_size)
    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [
                    {"refnr": "1", "beruf": "Job 1", "arbeitsort": {"ort": "Berlin"}},
                    {"refnr": "2", "beruf": "Job 2", "arbeitsort": {"ort": "Berlin"}}
                ],
                "maxErgebnisse": 4,
                "page": 1,
                "size": 2
            }"#,
        )
        .create();

    // Page 2: 0 results -- empty page
    let _m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=2.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "stellenangebote": [],
                "maxErgebnisse": 4,
                "page": 2,
                "size": 2
            }"#,
        )
        .create();

    // No page 3 mock

    let client = Jobsuche::new(server.url(), Credentials::default()).unwrap();

    let all_jobs: Vec<_> = client
        .search()
        .jobs(SearchOptions::builder().size(2).build())
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    // 2 from page 1, 0 from page 2, stops.
    assert_eq!(all_jobs.len(), 2);
}

#[test]
fn test_503_service_unavailable_no_retry() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(503)
        .create();

    let config = ClientConfig {
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());

    // Should get either Fault or Http error
    assert!(matches!(
        result.unwrap_err(),
        jobsuche::Error::Fault { .. } | jobsuche::Error::Http(_)
    ));
}

#[test]
fn test_504_gateway_timeout() {
    let mut server = Server::new();

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(504)
        .create();

    let config = ClientConfig {
        retry_enabled: false,
        ..Default::default()
    };

    let client = Jobsuche::with_config(server.url(), Credentials::default(), config).unwrap();

    let result = client.job_details("test");
    assert!(result.is_err());
}
