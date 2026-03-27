//! Test the actual pagination limit of the Bundesagentur API
//!
//! Uses raw HTTP requests with serde_json::Value to avoid deserialization issues.
//!
//! Run with: cargo run --example test_pagination_limit

use std::thread;
use std::time::Duration;

fn main() {
    let http_client = reqwest::blocking::Client::new();
    let api_key = "jobboerse-jobsuche";
    let test_query = "Informatik";

    println!("=== Bundesagentur API Pagination Limit Test ===");
    println!("Query: '{}'", test_query);
    println!();

    // Test specific page boundaries
    let test_pages: Vec<u64> = vec![1, 50, 99, 100, 101, 102, 200, 500, 999, 1000, 1001];

    for page in test_pages {
        thread::sleep(Duration::from_millis(500)); // Rate limiting

        let url = format!(
            "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service/pc/v4/jobs?was={}&size=100&page={}",
            test_query, page
        );

        let result = http_client
            .get(&url)
            .header("X-API-Key", api_key)
            .send();

        match result {
            Ok(response) => {
                let status = response.status();
                let body: String = match response.text() {
                    Ok(b) => b,
                    Err(e) => {
                        println!("Page {:>5}: HTTP {} | ERROR reading body: {}", page, status, e);
                        continue;
                    }
                };

                if status.is_success() {
                    match serde_json::from_str::<serde_json::Value>(&body) {
                        Ok(json) => {
                            let max_ergebnisse = json.get("maxErgebnisse");
                            let returned_page = json.get("page");
                            let returned_size = json.get("size");
                            let stellenangebote = json.get("stellenangebote")
                                .and_then(|v| v.as_array());
                            let count = stellenangebote.map(|a| a.len()).unwrap_or(0);
                            let first_refnr = stellenangebote
                                .and_then(|a| a.first())
                                .and_then(|j| j.get("refnr"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("EMPTY");

                            println!(
                                "Page {:>5}: {:>3} results | maxErgebnisse={} | returnedPage={} | size={} | first_refnr={}",
                                page,
                                count,
                                max_ergebnisse.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
                                returned_page.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
                                returned_size.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
                                first_refnr,
                            );
                        }
                        Err(e) => {
                            let preview = if body.len() > 200 { &body[..200] } else { &body };
                            println!("Page {:>5}: HTTP {} | JSON parse error: {} | body preview: {}", page, status, e, preview);
                        }
                    }
                } else {
                    let preview = if body.len() > 300 { &body[..300] } else { &body };
                    println!("Page {:>5}: HTTP {} | body: {}", page, status, preview);
                }
            }
            Err(e) => {
                println!("Page {:>5}: REQUEST ERROR - {:?}", page, e);
            }
        }
    }

    println!();
    println!("=== Test Complete ===");
}
