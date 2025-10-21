//! Unit tests using mocked API responses
//!
//! These tests use mockito to simulate API responses without making real HTTP calls.
//! They run quickly and don't require network access.

use jobsuche::{Arbeitszeit, Credentials, Jobsuche, SearchOptions};
use mockito::Server;

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
