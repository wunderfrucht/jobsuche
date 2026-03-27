//! API schema validation tests.
//!
//! These tests detect when the live Bundesagentur fur Arbeit API changes its
//! response structure. They compare live responses against a stored baseline and
//! produce a diff report with tiered severity.
//!
//! Run validation:
//!   cargo test --all-features --test api_schema_validation -- --ignored --test-threads=1
//!
//! Generate/update baseline:
//!   UPDATE_BASELINE=1 cargo test --all-features --test api_schema_validation -- --ignored --test-threads=1

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service";
const API_KEY: &str = "jobboerse-jobsuche";
const BASELINE_PATH: &str = "tests/fixtures/api-schema-baseline.json";
const REPORT_PATH: &str = "tests/fixtures/schema-diff-report.json";
const REQUEST_DELAY_MS: u64 = 500;
const SAMPLE_SIZE: usize = 5; // 3 search pages + 2 job details

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Baseline {
    version: String,
    generated_at: String,
    endpoints: BTreeMap<String, EndpointBaseline>,
    #[serde(default)]
    pagination: Option<PaginationBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaginationBaseline {
    page_100_status: u16,
    page_101_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndpointBaseline {
    method: String,
    path: String,
    fields: BTreeMap<String, FieldDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldDef {
    #[serde(rename = "type")]
    field_type: String,
    required: bool,
}

#[derive(Debug, Clone)]
struct FieldInfo {
    field_type: String,
    seen_count: usize,
    total_samples: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SchemaChange {
    endpoint: String,
    field_path: String,
    change_type: String,
    severity: String,
    old_type: Option<String>,
    new_type: Option<String>,
    detail: String,
    fingerprint: String,
}

#[derive(Debug, Serialize)]
struct DiffReport {
    generated_at: String,
    changes: Vec<SchemaChange>,
    critical_count: usize,
    warning_count: usize,
    informational_count: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Produce an ISO 8601 timestamp from SystemTime without a chrono dependency.
fn chrono_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // Convert unix seconds to date/time components
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Civil date from days since epoch (algorithm from Howard Hinnant)
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

/// Build a blocking reqwest client with the API key header.
fn api_client() -> reqwest::blocking::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", reqwest::header::HeaderValue::from_static(API_KEY));
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()
        .expect("Failed to build HTTP client")
}

fn delay() {
    thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
}

// ---------------------------------------------------------------------------
// Schema extraction
// ---------------------------------------------------------------------------

/// Recursively walk a JSON value and build a map of field_path -> JSON type name.
fn extract_schema(value: &Value, prefix: &str, schema: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                let type_name = json_type_name(val);
                schema.insert(path.clone(), type_name);
                // Recurse into objects
                if val.is_object() {
                    extract_schema(val, &path, schema);
                }
                // Recurse into arrays: use [] suffix and inspect first element
                if let Value::Array(arr) = val {
                    let arr_path = format!("{}[]", path);
                    if let Some(first) = arr.first() {
                        let inner_type = json_type_name(first);
                        schema.insert(arr_path.clone(), inner_type);
                        if first.is_object() {
                            extract_schema(first, &arr_path, schema);
                        }
                    }
                }
            }
        }
        _ => {
            // Leaf value at the top-level (unusual but handle gracefully)
            if !prefix.is_empty() {
                schema.insert(prefix.to_string(), json_type_name(value));
            }
        }
    }
}

fn json_type_name(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Schema merging
// ---------------------------------------------------------------------------

/// Merge multiple per-sample schemas into a single map tracking presence counts.
fn merge_schemas(schemas: &[BTreeMap<String, String>]) -> BTreeMap<String, FieldInfo> {
    let total = schemas.len();
    let mut merged: BTreeMap<String, FieldInfo> = BTreeMap::new();

    for schema in schemas {
        for (path, type_name) in schema {
            let entry = merged.entry(path.clone()).or_insert_with(|| FieldInfo {
                field_type: type_name.clone(),
                seen_count: 0,
                total_samples: total,
            });
            entry.seen_count += 1;
            entry.total_samples = total;

            // If we see a non-null type that conflicts with the stored type, warn and prefer the non-null one.
            if entry.field_type != *type_name {
                if entry.field_type == "null" {
                    // Upgrade from null to the concrete type
                    entry.field_type = type_name.clone();
                } else if type_name != "null" {
                    // Genuine conflict between two non-null types
                    eprintln!(
                        "WARNING: conflicting types for field '{}': '{}' vs '{}' — keeping '{}'",
                        path, entry.field_type, type_name, entry.field_type
                    );
                }
                // If the new type is null but existing is non-null, keep the non-null one (no action needed).
            }
        }
    }

    merged
}

// ---------------------------------------------------------------------------
// Schema diffing
// ---------------------------------------------------------------------------

fn diff_schemas(
    endpoint_name: &str,
    baseline: &EndpointBaseline,
    actual: &BTreeMap<String, FieldInfo>,
) -> Vec<SchemaChange> {
    let mut changes = Vec::new();

    // Check fields in baseline but missing from actual
    for (field_path, field_def) in &baseline.fields {
        match actual.get(field_path) {
            None => {
                // Field is completely absent from all samples
                let severity = if field_def.required {
                    "critical"
                } else {
                    "warning"
                };
                changes.push(SchemaChange {
                    endpoint: endpoint_name.to_string(),
                    field_path: field_path.clone(),
                    change_type: "removed".to_string(),
                    severity: severity.to_string(),
                    old_type: Some(field_def.field_type.clone()),
                    new_type: None,
                    detail: format!(
                        "{} field '{}' is no longer present in any sample",
                        if field_def.required {
                            "Required"
                        } else {
                            "Optional"
                        },
                        field_path
                    ),
                    fingerprint: format!("schema:removed:{}", field_path),
                });
            }
            Some(info) => {
                // Check for type changes (ignore null → something since null is absence)
                if info.field_type != field_def.field_type
                    && info.field_type != "null"
                    && field_def.field_type != "null"
                {
                    changes.push(SchemaChange {
                        endpoint: endpoint_name.to_string(),
                        field_path: field_path.clone(),
                        change_type: "type_changed".to_string(),
                        severity: "critical".to_string(),
                        old_type: Some(field_def.field_type.clone()),
                        new_type: Some(info.field_type.clone()),
                        detail: format!(
                            "Field '{}' changed type from '{}' to '{}'",
                            field_path, field_def.field_type, info.field_type
                        ),
                        fingerprint: format!("schema:type_changed:{}", field_path),
                    });
                }

                // Check if a required field became optional
                if field_def.required && info.seen_count < info.total_samples {
                    changes.push(SchemaChange {
                        endpoint: endpoint_name.to_string(),
                        field_path: field_path.clone(),
                        change_type: "required_to_optional".to_string(),
                        severity: "warning".to_string(),
                        old_type: Some(field_def.field_type.clone()),
                        new_type: Some(info.field_type.clone()),
                        detail: format!(
                            "Required field '{}' now only appears in {}/{} samples",
                            field_path, info.seen_count, info.total_samples
                        ),
                        fingerprint: format!("schema:required_to_optional:{}", field_path),
                    });
                }
            }
        }
    }

    // Check for new fields in actual that are not in baseline
    for (field_path, info) in actual {
        if !baseline.fields.contains_key(field_path) {
            changes.push(SchemaChange {
                endpoint: endpoint_name.to_string(),
                field_path: field_path.clone(),
                change_type: "new_field".to_string(),
                severity: "informational".to_string(),
                old_type: None,
                new_type: Some(info.field_type.clone()),
                detail: format!(
                    "New field '{}' of type '{}' appeared in {}/{} samples",
                    field_path, info.field_type, info.seen_count, info.total_samples
                ),
                fingerprint: format!("schema:new_field:{}", field_path),
            });
        }
    }

    changes
}

// ---------------------------------------------------------------------------
// Pagination limit check
// ---------------------------------------------------------------------------

fn fetch_pagination_status(client: &reqwest::blocking::Client, page: u32) -> u16 {
    delay();
    let url = format!(
        "{}/pc/v4/jobs?was=Informatik&size=5&page={}",
        API_BASE, page
    );
    match client.get(&url).send() {
        Ok(r) => r.status().as_u16(),
        Err(_) => 0, // network error
    }
}

fn check_pagination_limit(
    client: &reqwest::blocking::Client,
    baseline: &Option<PaginationBaseline>,
) -> Vec<SchemaChange> {
    let mut changes = Vec::new();

    let Some(expected) = baseline else {
        // No pagination baseline recorded — skip check
        return changes;
    };

    let actual_100 = fetch_pagination_status(client, 100);
    let actual_101 = fetch_pagination_status(client, 101);

    if actual_100 != expected.page_100_status {
        changes.push(SchemaChange {
            endpoint: "search".to_string(),
            field_path: "pagination.page_100".to_string(),
            change_type: "pagination_changed".to_string(),
            severity: "critical".to_string(),
            old_type: Some(expected.page_100_status.to_string()),
            new_type: Some(actual_100.to_string()),
            detail: format!(
                "Page 100 returned HTTP {} instead of expected {} — pagination limit may have changed",
                actual_100, expected.page_100_status
            ),
            fingerprint: "schema:pagination_changed:pagination.page_100".to_string(),
        });
    }

    if actual_101 != expected.page_101_status {
        changes.push(SchemaChange {
            endpoint: "search".to_string(),
            field_path: "pagination.page_101".to_string(),
            change_type: "pagination_changed".to_string(),
            severity: "critical".to_string(),
            old_type: Some(expected.page_101_status.to_string()),
            new_type: Some(actual_101.to_string()),
            detail: format!(
                "Page 101 returned HTTP {} instead of expected {} — pagination limit may have changed",
                actual_101, expected.page_101_status
            ),
            fingerprint: "schema:pagination_changed:pagination.page_101".to_string(),
        });
    }

    changes
}

// ---------------------------------------------------------------------------
// Report writing
// ---------------------------------------------------------------------------

fn write_report(changes: &[SchemaChange], path: &str) {
    let report = DiffReport {
        generated_at: chrono_now(),
        changes: changes.to_vec(),
        critical_count: changes.iter().filter(|c| c.severity == "critical").count(),
        warning_count: changes.iter().filter(|c| c.severity == "warning").count(),
        informational_count: changes
            .iter()
            .filter(|c| c.severity == "informational")
            .count(),
    };
    let json = serde_json::to_string_pretty(&report).expect("Failed to serialize diff report");
    fs::write(path, json).expect("Failed to write diff report");
}

// ---------------------------------------------------------------------------
// API fetching helpers
// ---------------------------------------------------------------------------

/// Fetch raw JSON from the search endpoint for a given page number.
fn fetch_search_page(client: &reqwest::blocking::Client, page: u32) -> Value {
    delay();
    let url = format!(
        "{}/pc/v4/jobs?was=Informatik&size={}&page={}",
        API_BASE, SAMPLE_SIZE, page
    );
    let resp = client
        .get(&url)
        .send()
        .expect("Search request failed")
        .error_for_status()
        .expect("Search request returned error status");
    resp.json::<Value>().expect("Failed to parse search JSON")
}

/// Fetch raw JSON for a specific job's details.
fn fetch_job_details(client: &reqwest::blocking::Client, refnr: &str) -> Option<Value> {
    delay();
    let encoded = jobsuche::encode_refnr(refnr);
    let url = format!("{}/pc/v4/jobdetails/{}", API_BASE, encoded);
    match client.get(&url).send() {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<Value>().ok()
            } else {
                eprintln!(
                    "Job details for '{}' returned HTTP {} — skipping",
                    refnr,
                    resp.status()
                );
                None
            }
        }
        Err(e) => {
            eprintln!("Job details request for '{}' failed: {} — skipping", refnr, e);
            None
        }
    }
}

/// Collect search schemas (3 pages) and job-detail schemas (2 details) from the live API.
fn collect_live_schemas(
    client: &reqwest::blocking::Client,
) -> (Vec<BTreeMap<String, String>>, Vec<BTreeMap<String, String>>) {
    let mut search_schemas = Vec::new();
    let mut detail_schemas = Vec::new();

    // Fetch 3 search pages
    for page in 1..=3 {
        let json = fetch_search_page(client, page);
        let mut schema = BTreeMap::new();
        extract_schema(&json, "", &mut schema);
        search_schemas.push(schema);
    }

    // Collect refnrs from the first search page to fetch job details
    let first_page = fetch_search_page(client, 1);
    let refnrs: Vec<String> = first_page
        .get("stellenangebote")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|j| j.get("refnr").and_then(|r| r.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Fetch 2 job details
    let mut details_fetched = 0;
    for refnr in &refnrs {
        if details_fetched >= 2 {
            break;
        }
        if let Some(json) = fetch_job_details(client, refnr) {
            let mut schema = BTreeMap::new();
            extract_schema(&json, "", &mut schema);
            detail_schemas.push(schema);
            details_fetched += 1;
        }
    }

    (search_schemas, detail_schemas)
}

// ---------------------------------------------------------------------------
// Baseline generation
// ---------------------------------------------------------------------------

fn run_update_baseline() {
    println!("Generating API schema baseline...");
    let client = api_client();

    let (search_schemas, detail_schemas) = collect_live_schemas(&client);

    let search_merged = merge_schemas(&search_schemas);
    let detail_merged = merge_schemas(&detail_schemas);

    let mut endpoints = BTreeMap::new();

    // Search endpoint baseline
    let mut search_fields = BTreeMap::new();
    for (path, info) in &search_merged {
        search_fields.insert(
            path.clone(),
            FieldDef {
                field_type: info.field_type.clone(),
                required: info.seen_count == info.total_samples,
            },
        );
    }
    endpoints.insert(
        "search".to_string(),
        EndpointBaseline {
            method: "GET".to_string(),
            path: "/pc/v4/jobs".to_string(),
            fields: search_fields,
        },
    );

    // Job details endpoint baseline
    if !detail_merged.is_empty() {
        let mut detail_fields = BTreeMap::new();
        for (path, info) in &detail_merged {
            detail_fields.insert(
                path.clone(),
                FieldDef {
                    field_type: info.field_type.clone(),
                    required: info.seen_count == info.total_samples,
                },
            );
        }
        endpoints.insert(
            "job_details".to_string(),
            EndpointBaseline {
                method: "GET".to_string(),
                path: "/pc/v4/jobdetails/{encoded_refnr}".to_string(),
                fields: detail_fields,
            },
        );
    }

    // Record pagination status codes
    let page_100_status = fetch_pagination_status(&client, 100);
    let page_101_status = fetch_pagination_status(&client, 101);
    println!(
        "  pagination: page 100 -> HTTP {}, page 101 -> HTTP {}",
        page_100_status, page_101_status
    );

    let baseline = Baseline {
        version: "1.0".to_string(),
        generated_at: chrono_now(),
        endpoints,
        pagination: Some(PaginationBaseline {
            page_100_status,
            page_101_status,
        }),
    };

    let json = serde_json::to_string_pretty(&baseline).expect("Failed to serialize baseline");
    fs::write(BASELINE_PATH, json).expect("Failed to write baseline file");

    let search_count = baseline
        .endpoints
        .get("search")
        .map(|e| e.fields.len())
        .unwrap_or(0);
    let detail_count = baseline
        .endpoints
        .get("job_details")
        .map(|e| e.fields.len())
        .unwrap_or(0);

    println!("Baseline written to {}", BASELINE_PATH);
    println!(
        "  search: {} fields, job_details: {} fields",
        search_count, detail_count
    );
}

// ---------------------------------------------------------------------------
// Main test
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn api_schema_validate() {
    // Check if we should update the baseline instead of validating
    if std::env::var("UPDATE_BASELINE")
        .ok()
        .filter(|v| v == "1" || v == "true")
        .is_some()
    {
        run_update_baseline();
        return;
    }

    // Load baseline
    let baseline_content = fs::read_to_string(BASELINE_PATH).unwrap_or_else(|_| {
        panic!(
            "Baseline file not found at '{}'. Run with UPDATE_BASELINE=1 to generate it.",
            BASELINE_PATH
        )
    });
    let baseline: Baseline =
        serde_json::from_str(&baseline_content).expect("Failed to parse baseline JSON");

    println!("Loaded baseline (generated {})", baseline.generated_at);

    // Fetch live schemas
    let client = api_client();
    let (search_schemas, detail_schemas) = collect_live_schemas(&client);

    let search_merged = merge_schemas(&search_schemas);
    let detail_merged = merge_schemas(&detail_schemas);

    let mut all_changes = Vec::new();

    // Diff search endpoint
    if let Some(search_baseline) = baseline.endpoints.get("search") {
        let search_changes = diff_schemas("search", search_baseline, &search_merged);
        all_changes.extend(search_changes);
    }

    // Diff job_details endpoint
    if let Some(detail_baseline) = baseline.endpoints.get("job_details") {
        if !detail_merged.is_empty() {
            let detail_changes = diff_schemas("job_details", detail_baseline, &detail_merged);
            all_changes.extend(detail_changes);
        } else {
            eprintln!("WARNING: could not fetch any job details — skipping job_details diff");
        }
    }

    // Check pagination limits
    let pagination_changes = check_pagination_limit(&client, &baseline.pagination);
    all_changes.extend(pagination_changes);

    // Write report
    write_report(&all_changes, REPORT_PATH);

    let critical = all_changes
        .iter()
        .filter(|c| c.severity == "critical")
        .count();
    let warnings = all_changes
        .iter()
        .filter(|c| c.severity == "warning")
        .count();
    let informational = all_changes
        .iter()
        .filter(|c| c.severity == "informational")
        .count();

    println!("Schema diff complete:");
    println!("  critical:      {}", critical);
    println!("  warning:       {}", warnings);
    println!("  informational: {}", informational);

    if !all_changes.is_empty() {
        println!("\nChanges:");
        for change in &all_changes {
            println!(
                "  [{}] {} — {}.{}: {}",
                change.severity, change.change_type, change.endpoint, change.field_path, change.detail
            );
        }
    }

    println!("\nReport written to {}", REPORT_PATH);

    assert_eq!(
        critical, 0,
        "Found {} critical API schema changes — see report at {}",
        critical, REPORT_PATH
    );
}
