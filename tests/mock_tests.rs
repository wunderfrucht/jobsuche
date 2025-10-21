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
    assert_eq!(results.stellenangebote[0].beruf, "Rust Developer");
    assert_eq!(results.stellenangebote[1].beruf, "Senior Rust Engineer");
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
    assert_eq!(results.stellenangebote[0].beruf, "Backend Developer");
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
        "refnr": "10001-DETAILS-S",
        "titel": "Senior Rust Developer",
        "arbeitgeber": "Test Company",
        "stellenbeschreibung": "Looking for an experienced Rust developer...",
        "arbeitsorte": [
            {
                "ort": "Berlin",
                "plz": "10115",
                "region": "Berlin"
            }
        ],
        "fertigkeiten": [
            {
                "hierarchieName": "Rust Programming"
            },
            {
                "hierarchieName": "Systems Programming"
            }
        ]
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
    assert_eq!(details.fertigkeiten.len(), 2);
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
