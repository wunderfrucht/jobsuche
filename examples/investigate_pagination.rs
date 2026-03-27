//! Comprehensive investigation of Bundesagentur API pagination behavior
//!
//! Tests boundary scan, data quality, large page numbers, total consistency,
//! result uniqueness, and behavior across different search terms.
//!
//! Run with: cargo run --example investigate_pagination

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

const API_KEY: &str = "jobboerse-jobsuche";
const BASE_URL: &str = "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service/pc/v4/jobs";
const DELAY_MS: u64 = 500;

struct PageResult {
    page: u64,
    status: u16,
    count: usize,
    max_ergebnisse: Option<u64>,
    returned_page: Option<u64>,
    refnrs: Vec<String>,
    error_detail: Option<String>,
}

fn fetch_page(client: &reqwest::blocking::Client, query: &str, page: u64) -> PageResult {
    let url = format!("{}?was={}&size=100&page={}", BASE_URL, query, page);

    let result = client.get(&url).header("X-API-Key", API_KEY).send();

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = match response.text() {
                Ok(b) => b,
                Err(e) => {
                    return PageResult {
                        page,
                        status,
                        count: 0,
                        max_ergebnisse: None,
                        returned_page: None,
                        refnrs: vec![],
                        error_detail: Some(format!("body read error: {}", e)),
                    };
                }
            };

            if (200..300).contains(&status) {
                match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(json) => {
                        let max_ergebnisse = json.get("maxErgebnisse").and_then(|v| v.as_u64());
                        let returned_page = json.get("page").and_then(|v| v.as_u64());
                        let stellenangebote =
                            json.get("stellenangebote").and_then(|v| v.as_array());
                        let count = stellenangebote.map(|a| a.len()).unwrap_or(0);
                        let refnrs: Vec<String> = stellenangebote
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|j| {
                                        j.get("refnr").and_then(|v| v.as_str()).map(String::from)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        PageResult {
                            page,
                            status,
                            count,
                            max_ergebnisse,
                            returned_page,
                            refnrs,
                            error_detail: None,
                        }
                    }
                    Err(e) => {
                        let preview = if body.len() > 200 {
                            &body[..200]
                        } else {
                            &body
                        };
                        PageResult {
                            page,
                            status,
                            count: 0,
                            max_ergebnisse: None,
                            returned_page: None,
                            refnrs: vec![],
                            error_detail: Some(format!(
                                "JSON parse error: {} | preview: {}",
                                e, preview
                            )),
                        }
                    }
                }
            } else {
                let preview = if body.len() > 300 {
                    &body[..300]
                } else {
                    &body
                };
                PageResult {
                    page,
                    status,
                    count: 0,
                    max_ergebnisse: None,
                    returned_page: None,
                    refnrs: vec![],
                    error_detail: Some(format!("body: {}", preview)),
                }
            }
        }
        Err(e) => PageResult {
            page,
            status: 0,
            count: 0,
            max_ergebnisse: None,
            returned_page: None,
            refnrs: vec![],
            error_detail: Some(format!("request error: {}", e)),
        },
    }
}

fn print_result_row(r: &PageResult) {
    if let Some(ref err) = r.error_detail {
        println!(
            "  Page {:>5} | HTTP {:>3} | {:>3} results | maxErg={:<10} | retPage={:<6} | refnrs=0 | ERR: {}",
            r.page,
            r.status,
            r.count,
            r.max_ergebnisse.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            r.returned_page.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            err,
        );
    } else {
        let first = r.refnrs.first().map(|s| s.as_str()).unwrap_or("EMPTY");
        let last = r.refnrs.last().map(|s| s.as_str()).unwrap_or("EMPTY");
        println!(
            "  Page {:>5} | HTTP {:>3} | {:>3} results | maxErg={:<10} | retPage={:<6} | refnrs={:<3} | first={} last={}",
            r.page,
            r.status,
            r.count,
            r.max_ergebnisse.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            r.returned_page.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
            r.refnrs.len(),
            first,
            last,
        );
    }
}

fn main() {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client");

    println!("============================================================");
    println!("  BUNDESAGENTUR API PAGINATION INVESTIGATION");
    println!(
        "  Timestamp: {:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );
    println!("============================================================");
    println!();

    // ================================================================
    // TEST 1: Boundary scan pages 95-110 with "Informatik"
    // ================================================================
    println!("=== TEST 1: Boundary scan pages 95-110 (query='Informatik') ===");
    let mut boundary_results: Vec<PageResult> = Vec::new();
    for page in 95..=110 {
        thread::sleep(Duration::from_millis(DELAY_MS));
        let r = fetch_page(&client, "Informatik", page);
        print_result_row(&r);
        boundary_results.push(r);
    }
    println!();

    // ================================================================
    // TEST 2: Data quality at boundaries — check for unique refnrs
    // ================================================================
    println!("=== TEST 2: Data quality at boundaries (unique refnrs near page 100) ===");
    {
        let mut all_refnrs: Vec<(u64, Vec<String>)> = Vec::new();
        for page in [98, 99, 100, 101, 102] {
            thread::sleep(Duration::from_millis(DELAY_MS));
            let r = fetch_page(&client, "Informatik", page);
            println!(
                "  Page {} => {} results, {} unique refnrs",
                page,
                r.count,
                {
                    let set: HashSet<&str> = r.refnrs.iter().map(|s| s.as_str()).collect();
                    set.len()
                }
            );
            all_refnrs.push((page, r.refnrs));
        }

        // Check for duplicates across pages
        let mut seen: HashSet<String> = HashSet::new();
        let mut duplicates_across = 0u64;
        for (page, refnrs) in &all_refnrs {
            for refnr in refnrs {
                if !seen.insert(refnr.clone()) {
                    duplicates_across += 1;
                    println!(
                        "    DUPLICATE refnr across pages: {} (seen again on page {})",
                        refnr, page
                    );
                    if duplicates_across >= 10 {
                        println!("    ... (stopping after 10 duplicates)");
                        break;
                    }
                }
            }
            if duplicates_across >= 10 {
                break;
            }
        }
        if duplicates_across == 0 {
            println!("  No cross-page duplicate refnrs found among pages 98-102");
        } else {
            println!("  Total cross-page duplicates found: {}", duplicates_across);
        }
    }
    println!();

    // ================================================================
    // TEST 3: Large page numbers
    // ================================================================
    println!("=== TEST 3: Large page numbers (query='Informatik') ===");
    for page in [200, 500, 1000, 5000, 10000] {
        thread::sleep(Duration::from_millis(DELAY_MS));
        let r = fetch_page(&client, "Informatik", page);
        print_result_row(&r);
    }
    println!();

    // ================================================================
    // TEST 4: Does maxErgebnisse change across pages?
    // ================================================================
    println!("=== TEST 4: maxErgebnisse consistency (query='Informatik') ===");
    {
        let test_pages = [1, 10, 50, 99, 100, 101, 150, 200];
        let mut max_values: Vec<(u64, Option<u64>)> = Vec::new();
        for page in test_pages {
            thread::sleep(Duration::from_millis(DELAY_MS));
            let r = fetch_page(&client, "Informatik", page);
            println!(
                "  Page {:>5} => maxErgebnisse={}, count={}",
                page,
                r.max_ergebnisse
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null/missing".into()),
                r.count
            );
            max_values.push((page, r.max_ergebnisse));
        }
        let unique_max: HashSet<Option<u64>> = max_values.iter().map(|(_, v)| *v).collect();
        println!("  Distinct maxErgebnisse values seen: {:?}", unique_max);
    }
    println!();

    // ================================================================
    // TEST 5: Are high-page results different from page 1?
    // ================================================================
    println!("=== TEST 5: Result uniqueness — page 1 vs page 101 (query='Informatik') ===");
    {
        thread::sleep(Duration::from_millis(DELAY_MS));
        let p1 = fetch_page(&client, "Informatik", 1);
        thread::sleep(Duration::from_millis(DELAY_MS));
        let p101 = fetch_page(&client, "Informatik", 101);

        let set1: HashSet<&str> = p1.refnrs.iter().map(|s| s.as_str()).collect();
        let set101: HashSet<&str> = p101.refnrs.iter().map(|s| s.as_str()).collect();
        let overlap: Vec<&str> = set1
            .iter()
            .filter(|r| set101.contains(*r))
            .copied()
            .collect();

        println!(
            "  Page   1: {} results, {} unique refnrs",
            p1.count,
            set1.len()
        );
        println!(
            "  Page 101: {} results, {} unique refnrs (HTTP {})",
            p101.count,
            set101.len(),
            p101.status
        );
        println!("  Overlap: {} refnrs appear on BOTH pages", overlap.len());
        if !overlap.is_empty() {
            let sample: Vec<&str> = overlap.iter().take(5).copied().collect();
            println!("  Sample overlapping refnrs: {:?}", sample);
        }
        if p1.count > 0 {
            println!(
                "  Page 1 first 3 refnrs: {:?}",
                &p1.refnrs[..p1.refnrs.len().min(3)]
            );
        }
        if p101.count > 0 {
            println!(
                "  Page 101 first 3 refnrs: {:?}",
                &p101.refnrs[..p101.refnrs.len().min(3)]
            );
        }
    }
    println!();

    // ================================================================
    // TEST 6: Different search terms — narrow query
    // ================================================================
    println!("=== TEST 6: Narrow query 'Rust Developer' — boundary scan ===");
    {
        // First check how many results exist
        thread::sleep(Duration::from_millis(DELAY_MS));
        let r1 = fetch_page(&client, "Rust%20Developer", 1);
        println!(
            "  Page 1: {} results, maxErgebnisse={}, HTTP {}",
            r1.count,
            r1.max_ergebnisse
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into()),
            r1.status,
        );

        // Test around page 100 boundary
        for page in [10, 50, 99, 100, 101, 102, 200] {
            thread::sleep(Duration::from_millis(DELAY_MS));
            let r = fetch_page(&client, "Rust%20Developer", page);
            print_result_row(&r);
        }
    }
    println!();

    // ================================================================
    // TEST 6b: Different search terms — very broad query
    // ================================================================
    println!("=== TEST 6b: Very broad query (empty 'was' param) — boundary scan ===");
    {
        for page in [1, 99, 100, 101, 102, 200, 500] {
            thread::sleep(Duration::from_millis(DELAY_MS));
            // Empty query to get maximum results
            let url = format!("{}?size=100&page={}", BASE_URL, page);
            let result = client.get(&url).header("X-API-Key", API_KEY).send();

            match result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let body = response.text().unwrap_or_default();
                    if (200..300).contains(&status) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            let max_e = json.get("maxErgebnisse").and_then(|v| v.as_u64());
                            let count = json
                                .get("stellenangebote")
                                .and_then(|v| v.as_array())
                                .map(|a| a.len())
                                .unwrap_or(0);
                            let ret_page = json.get("page").and_then(|v| v.as_u64());
                            println!(
                                "  Page {:>5} | HTTP {} | {:>3} results | maxErg={:<10} | retPage={}",
                                page, status, count,
                                max_e.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                                ret_page.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                            );
                        } else {
                            let preview = if body.len() > 200 {
                                &body[..200]
                            } else {
                                &body
                            };
                            println!(
                                "  Page {:>5} | HTTP {} | JSON error | {}",
                                page, status, preview
                            );
                        }
                    } else {
                        let preview = if body.len() > 200 {
                            &body[..200]
                        } else {
                            &body
                        };
                        println!("  Page {:>5} | HTTP {} | {}", page, status, preview);
                    }
                }
                Err(e) => println!("  Page {:>5} | ERROR: {}", page, e),
            }
        }
    }
    println!();

    println!("============================================================");
    println!("  INVESTIGATION COMPLETE");
    println!("============================================================");
}
