//! Async client tests using mocked API responses
//!
//! These tests verify the async client functionality without making real HTTP calls.

use jobsuche::{ClientConfig, Credentials, JobsucheAsync, SearchOptions};
use mockito::Server;
use std::time::Duration;

#[tokio::test]
async fn test_async_search_with_mock() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "stellenangebote": [
            {
                "refnr": "10001-ASYNC-TEST-S",
                "beruf": "Async Rust Developer",
                "arbeitgeber": "Async Test Company",
                "aktuelleVeroeffentlichungsdatum": "2025-10-20",
                "arbeitsort": {
                    "ort": "Berlin",
                    "region": "Berlin",
                    "plz": "10115"
                }
            }
        ],
        "maxErgebnisse": 1,
        "page": 1,
        "size": 10
    }"#;

    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*was=Async.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let results = client
        .search()
        .list(SearchOptions::builder().was("Async Rust").size(10).build())
        .await
        .unwrap();

    assert_eq!(results.stellenangebote.len(), 1);
    assert_eq!(results.stellenangebote[0].refnr, "10001-ASYNC-TEST-S");
    assert_eq!(results.stellenangebote[0].beruf, "Async Rust Developer");
}

#[tokio::test]
async fn test_async_job_details_mock() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "refnr": "10001-1001601666-S",
        "titel": "Senior Rust Developer",
        "arbeitgeber": "Tech Company GmbH",
        "stellenbeschreibung": "We are looking for an experienced Rust developer...",
        "arbeitszeitmodelle": ["VOLLZEIT"],
        "arbeitsorte": [
            {
                "ort": "Berlin",
                "plz": "10115"
            }
        ]
    }"#;

    // The refnr gets base64url encoded
    let _m = server
        .mock("GET", "/pc/v4/jobdetails/MTAwMDEtMTAwMTYwMTY2Ni1T")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let job = client.job_details("10001-1001601666-S").await.unwrap();

    assert_eq!(job.refnr, Some("10001-1001601666-S".to_string()));
    assert_eq!(job.titel, Some("Senior Rust Developer".to_string()));
    assert_eq!(job.arbeitgeber, Some("Tech Company GmbH".to_string()));
}

#[tokio::test]
async fn test_async_job_details_not_found() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(404)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::NotFound));
}

#[tokio::test]
async fn test_async_employer_logo_success() {
    let mut server = Server::new_async().await;

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
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let logo = client.employer_logo("test-hash").await.unwrap();
    assert_eq!(logo.len(), png_bytes.len());
    assert_eq!(logo, png_bytes);
}

#[tokio::test]
async fn test_async_employer_logo_not_found() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", "/ed/v1/arbeitgeberlogo/nonexistent")
        .with_status(404)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.employer_logo("nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::NotFound));
}

#[tokio::test]
async fn test_async_401_unauthorized() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(401)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::Unauthorized));
}

#[tokio::test]
async fn test_async_403_forbidden() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(403)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), jobsuche::Error::Forbidden));
}

#[tokio::test]
async fn test_async_405_method_not_allowed() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(405)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        jobsuche::Error::MethodNotAllowed
    ));
}

#[tokio::test]
async fn test_async_rate_limit_429_with_retry_after() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "120")
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        jobsuche::Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(120));
        }
        _ => panic!("Expected RateLimited error"),
    }
}

#[tokio::test]
async fn test_async_rate_limit_429_without_retry_after() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(429)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        jobsuche::Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, None);
        }
        _ => panic!("Expected RateLimited error"),
    }
}

#[tokio::test]
async fn test_async_empty_results() {
    let mut server = Server::new_async().await;

    let mock_response = r#"{
        "stellenangebote": [],
        "maxErgebnisse": 0,
        "page": 1,
        "size": 10
    }"#;

    let _m = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let results = client
        .search()
        .list(
            SearchOptions::builder()
                .was("NonexistentJobTitle12345")
                .build(),
        )
        .await
        .unwrap();

    assert_eq!(results.stellenangebote.len(), 0);
    assert_eq!(results.max_ergebnisse, Some(0));
}

#[tokio::test]
async fn test_async_500_server_error_with_api_errors() {
    let mut server = Server::new_async().await;

    let error_response = r#"{
        "errors": [
            {
                "code": "INTERNAL_ERROR",
                "message": "Internal server error occurred"
            }
        ],
        "errorMessages": ["Internal server error occurred"]
    }"#;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(error_response)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        jobsuche::Error::Fault { code, .. } => {
            assert_eq!(code, 500);
            // Successfully parsed as a Fault error
        }
        _ => panic!("Expected Fault error"),
    }
}

#[tokio::test]
async fn test_async_500_server_error_plain_text() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    let result = client.job_details("test").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        jobsuche::Error::Fault { code, errors } => {
            assert_eq!(code, 500);
            assert_eq!(errors.errors.len(), 0);
        }
        _ => panic!("Expected Fault error"),
    }
}

#[tokio::test]
async fn test_async_with_config_custom_timeout() {
    let config = ClientConfig {
        timeout: Duration::from_secs(5),
        connect_timeout: Duration::from_secs(2),
        max_retries: 1,
        retry_enabled: false,
    };

    let client = JobsucheAsync::with_config(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
        config,
    )
    .await;

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_with_config_retries_enabled() {
    let config = ClientConfig {
        timeout: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(10),
        max_retries: 3,
        retry_enabled: true,
    };

    let client = JobsucheAsync::with_config(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
        config,
    )
    .await;

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_with_core() {
    use jobsuche::core::ClientCore;

    let core = ClientCore::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .unwrap();

    let client = JobsucheAsync::with_core(core).await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_with_config_and_core() {
    use jobsuche::core::ClientCore;

    let core = ClientCore::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .unwrap();

    let config = ClientConfig {
        timeout: Duration::from_secs(15),
        connect_timeout: Duration::from_secs(5),
        max_retries: 2,
        retry_enabled: true,
    };

    let client = JobsucheAsync::with_config_and_core(core, config).await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_search_interface() {
    let client = JobsucheAsync::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .await
    .unwrap();

    // Just verify we can create a search interface
    let _search = client.search();
}

#[tokio::test]
async fn test_async_pagination_mock() {
    let mut server = Server::new_async().await;

    let page1_response = r#"{
        "stellenangebote": [
            {"refnr": "REF1", "beruf": "Job 1", "arbeitgeber": "Company 1", "arbeitsort": {"ort": "Berlin"}}
        ],
        "maxErgebnisse": 2,
        "page": 0,
        "size": 1
    }"#;

    let page2_response = r#"{
        "stellenangebote": [
            {"refnr": "REF2", "beruf": "Job 2", "arbeitgeber": "Company 2", "arbeitsort": {"ort": "Munich"}}
        ],
        "maxErgebnisse": 2,
        "page": 1,
        "size": 1
    }"#;

    let _m1 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=0.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page1_response)
        .create_async()
        .await;

    let _m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/pc/v4/jobs\?.*page=1.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page2_response)
        .create_async()
        .await;

    let client = JobsucheAsync::new(server.url(), Credentials::default())
        .await
        .unwrap();

    // Test page 0
    let results_page1 = client
        .search()
        .list(SearchOptions::builder().page(0).size(1).build())
        .await
        .unwrap();
    assert_eq!(results_page1.stellenangebote.len(), 1);
    assert_eq!(results_page1.stellenangebote[0].refnr, "REF1");

    // Test page 1
    let results_page2 = client
        .search()
        .list(SearchOptions::builder().page(1).size(1).build())
        .await
        .unwrap();
    assert_eq!(results_page2.stellenangebote.len(), 1);
    assert_eq!(results_page2.stellenangebote[0].refnr, "REF2");
}
