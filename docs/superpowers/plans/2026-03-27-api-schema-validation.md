# API Schema Validation & Auto-Alerting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Detect API schema drift against the live Bundesagentur REST API with tiered GitHub Issue alerting.

**Architecture:** A Rust integration test fetches raw JSON from the live API, extracts field paths and types, diffs against a checked-in baseline file, writes a report. A CI workflow reads the report and creates GitHub Issues for critical (removed/type-changed fields) or informational (new fields) changes. Pagination limits are verified as part of the schema check.

**Tech Stack:** Rust, serde_json::Value, reqwest::blocking, GitHub Actions, actions/github-script@v7

**Spec:** `docs/superpowers/specs/2026-03-27-api-schema-validation-design.md`

---

### Task 1: Setup — gitignore and fixtures directory

**Files:**
- Modify: `.gitignore`
- Create: `tests/fixtures/.gitkeep`

- [ ] **Step 1: Add schema-diff-report.json to .gitignore**

Append to `.gitignore`:
```
# Schema validation
tests/fixtures/schema-diff-report.json
```

- [ ] **Step 2: Create the fixtures directory**

```bash
mkdir -p tests/fixtures
touch tests/fixtures/.gitkeep
```

- [ ] **Step 3: Commit**

```bash
git add .gitignore tests/fixtures/.gitkeep
git commit -m "chore: add tests/fixtures directory and gitignore schema report"
```

---

### Task 2: Schema extraction and diffing logic

**Files:**
- Create: `tests/api_schema_validation.rs`

This is the core logic. The test file contains:
1. `extract_schema(value: &Value, prefix: &str, schema: &mut BTreeMap<String, String>)` — recursively walks a `serde_json::Value`, building a map of `"field.path" -> "type"` (string, number, boolean, array, object, null).
2. `merge_schemas(schemas: &[BTreeMap<String, String>]) -> BTreeMap<String, FieldInfo>` — merges multiple sample schemas, tracking how many samples each field appeared in and whether the type was consistent.
3. `diff_schemas(baseline: &Baseline, actual: &BTreeMap<String, FieldInfo>) -> Vec<SchemaChange>` — compares baseline against actual, producing a list of changes with severity.
4. `write_report(changes: &[SchemaChange], path: &Path)` — writes the diff report JSON.

- [ ] **Step 1: Write the test file with schema extraction**

Create `tests/api_schema_validation.rs` with these contents:

```rust
//! API Schema Validation — detects drift between live API and baseline.
//!
//! Normal run (diff mode):
//!   cargo test --all-features --test api_schema_validation -- --test-threads=1
//!
//! Generate/update baseline from live API:
//!   UPDATE_BASELINE=1 cargo test --all-features --test api_schema_validation -- --ignored --test-threads=1

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

const API_BASE: &str = "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service";
const API_KEY: &str = "jobboerse-jobsuche";
const BASELINE_PATH: &str = "tests/fixtures/api-schema-baseline.json";
const REPORT_PATH: &str = "tests/fixtures/schema-diff-report.json";

// --- Data structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Baseline {
    version: String,
    generated_at: String,
    endpoints: BTreeMap<String, EndpointBaseline>,
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
    change_type: String, // "removed", "type_changed", "new_field", "required_to_optional"
    severity: String,    // "critical", "warning", "informational"
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

// --- Schema extraction ---

/// Recursively extract field paths and their JSON types from a Value.
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
                schema.insert(path.clone(), type_name.to_string());
                match val {
                    Value::Object(_) => extract_schema(val, &path, schema),
                    Value::Array(arr) => {
                        // Extract schema from first element as representative
                        if let Some(first) = arr.first() {
                            let arr_prefix = format!("{}[]", path);
                            if first.is_object() {
                                extract_schema(first, &arr_prefix, schema);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn json_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Merge schemas from multiple samples into a single map with occurrence counts.
fn merge_schemas(
    schemas: &[BTreeMap<String, String>],
) -> BTreeMap<String, FieldInfo> {
    let total = schemas.len();
    let mut merged: BTreeMap<String, FieldInfo> = BTreeMap::new();

    for schema in schemas {
        for (path, field_type) in schema {
            let entry = merged.entry(path.clone()).or_insert_with(|| FieldInfo {
                field_type: field_type.clone(),
                seen_count: 0,
                total_samples: total,
            });
            entry.seen_count += 1;
            // If types conflict across samples, prefer the non-null type
            if entry.field_type == "null" && field_type != "null" {
                entry.field_type = field_type.clone();
            }
        }
    }
    merged
}

/// Diff the baseline against actual observed fields.
fn diff_schemas(
    endpoint_name: &str,
    baseline: &EndpointBaseline,
    actual: &BTreeMap<String, FieldInfo>,
) -> Vec<SchemaChange> {
    let mut changes = Vec::new();

    // Check for removed or type-changed fields
    for (path, def) in &baseline.fields {
        match actual.get(path) {
            None => {
                // Field not seen in any sample
                let (severity, change_type) = if def.required {
                    ("critical".to_string(), "removed".to_string())
                } else {
                    ("warning".to_string(), "removed".to_string())
                };
                changes.push(SchemaChange {
                    endpoint: endpoint_name.to_string(),
                    field_path: path.clone(),
                    change_type,
                    severity,
                    old_type: Some(def.field_type.clone()),
                    new_type: None,
                    detail: format!(
                        "Field '{}' (type: {}, required: {}) not found in any API sample",
                        path, def.field_type, def.required
                    ),
                    fingerprint: fingerprint("removed", path),
                });
            }
            Some(info) => {
                // Check type change
                if info.field_type != def.field_type && info.field_type != "null" {
                    changes.push(SchemaChange {
                        endpoint: endpoint_name.to_string(),
                        field_path: path.clone(),
                        change_type: "type_changed".to_string(),
                        severity: "critical".to_string(),
                        old_type: Some(def.field_type.clone()),
                        new_type: Some(info.field_type.clone()),
                        detail: format!(
                            "Field '{}' type changed from '{}' to '{}'",
                            path, def.field_type, info.field_type
                        ),
                        fingerprint: fingerprint("type_changed", path),
                    });
                }
                // Check required-to-optional drift
                if def.required && info.seen_count < info.total_samples {
                    changes.push(SchemaChange {
                        endpoint: endpoint_name.to_string(),
                        field_path: path.clone(),
                        change_type: "required_to_optional".to_string(),
                        severity: "warning".to_string(),
                        old_type: Some(def.field_type.clone()),
                        new_type: Some(info.field_type.clone()),
                        detail: format!(
                            "Field '{}' was required but only appeared in {}/{} samples",
                            path, info.seen_count, info.total_samples
                        ),
                        fingerprint: fingerprint("required_to_optional", path),
                    });
                }
            }
        }
    }

    // Check for new fields
    for (path, info) in actual {
        if !baseline.fields.contains_key(path) {
            changes.push(SchemaChange {
                endpoint: endpoint_name.to_string(),
                field_path: path.clone(),
                change_type: "new_field".to_string(),
                severity: "informational".to_string(),
                old_type: None,
                new_type: Some(info.field_type.clone()),
                detail: format!(
                    "New field '{}' (type: {}) found in {}/{} samples",
                    path, info.field_type, info.seen_count, info.total_samples
                ),
                fingerprint: fingerprint("new_field", path),
            });
        }
    }

    changes
}

fn fingerprint(change_type: &str, field_path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    change_type.hash(&mut hasher);
    field_path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn write_report(changes: &[SchemaChange], path: &Path) {
    let report = DiffReport {
        generated_at: chrono_now(),
        critical_count: changes.iter().filter(|c| c.severity == "critical").count(),
        warning_count: changes.iter().filter(|c| c.severity == "warning").count(),
        informational_count: changes
            .iter()
            .filter(|c| c.severity == "informational")
            .count(),
        changes: changes.to_vec(),
    };
    let json = serde_json::to_string_pretty(&report).expect("Failed to serialize report");
    fs::write(path, json).expect("Failed to write report");
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    format!("{}Z", duration.as_secs())
}

// --- API fetching ---

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::new()
}

fn fetch_search_raw(client: &reqwest::blocking::Client, page: u64, size: u64) -> Result<Value, String> {
    thread::sleep(Duration::from_millis(500));
    let url = format!(
        "{}/pc/v4/jobs?was=Informatik&size={}&page={}",
        API_BASE, size, page
    );
    let resp = client
        .get(&url)
        .header("X-API-Key", API_KEY)
        .send()
        .map_err(|e| format!("Connection error: {}", e))?;

    let status = resp.status();
    let body = resp.text().map_err(|e| format!("Body read error: {}", e))?;

    if status.is_success() {
        serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP {}: {}", status, &body[..body.len().min(200)]))
    }
}

fn fetch_job_details_raw(client: &reqwest::blocking::Client, refnr: &str) -> Result<Value, String> {
    thread::sleep(Duration::from_millis(500));
    let encoded = jobsuche::encode_refnr(refnr);
    let url = format!("{}/pc/v4/jobdetails/{}", API_BASE, encoded);
    let resp = client
        .get(&url)
        .header("X-API-Key", API_KEY)
        .send()
        .map_err(|e| format!("Connection error: {}", e))?;

    let status = resp.status();
    let body = resp.text().map_err(|e| format!("Body read error: {}", e))?;

    if status.is_success() {
        serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))
    } else if status.as_u16() == 404 {
        Err("404_not_found".to_string())
    } else {
        Err(format!("HTTP {}: {}", status, &body[..body.len().min(200)]))
    }
}

fn check_pagination_limit(client: &reqwest::blocking::Client) -> Vec<SchemaChange> {
    let mut changes = Vec::new();

    // Page 100 should succeed
    match fetch_search_raw(client, 100, 100) {
        Ok(_) => println!("  Page 100: OK (as expected)"),
        Err(e) => {
            println!("  Page 100: FAILED — {}", e);
            changes.push(SchemaChange {
                endpoint: "pagination".to_string(),
                field_path: "page_limit".to_string(),
                change_type: "pagination_limit_decreased".to_string(),
                severity: "critical".to_string(),
                old_type: Some("page 100 succeeds".to_string()),
                new_type: Some(format!("page 100 fails: {}", e)),
                detail: "API pagination limit may have decreased below 100 pages".to_string(),
                fingerprint: fingerprint("pagination_limit_decreased", "page_limit"),
            });
        }
    }

    // Page 101 should fail with 400
    match fetch_search_raw(client, 101, 100) {
        Ok(response) => {
            let count = response
                .get("stellenangebote")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            println!("  Page 101: SUCCEEDED with {} results (unexpected!)", count);
            changes.push(SchemaChange {
                endpoint: "pagination".to_string(),
                field_path: "page_limit".to_string(),
                change_type: "pagination_limit_increased".to_string(),
                severity: "critical".to_string(),
                old_type: Some("page 101 returns HTTP 400".to_string()),
                new_type: Some(format!("page 101 returns data ({} results)", count)),
                detail: "API pagination limit may have increased beyond 100 pages — client limit should be updated".to_string(),
                fingerprint: fingerprint("pagination_limit_increased", "page_limit"),
            });
        }
        Err(e) => {
            println!("  Page 101: Rejected (as expected) — {}", e);
        }
    }

    changes
}

// --- Baseline generation ---

fn generate_baseline(
    search_schemas: &BTreeMap<String, FieldInfo>,
    details_schemas: &BTreeMap<String, FieldInfo>,
) -> Baseline {
    let mut endpoints = BTreeMap::new();

    let search_fields: BTreeMap<String, FieldDef> = search_schemas
        .iter()
        .map(|(path, info)| {
            (
                path.clone(),
                FieldDef {
                    field_type: info.field_type.clone(),
                    required: info.seen_count == info.total_samples,
                },
            )
        })
        .collect();

    endpoints.insert(
        "search".to_string(),
        EndpointBaseline {
            method: "GET".to_string(),
            path: "/pc/v4/jobs".to_string(),
            fields: search_fields,
        },
    );

    let details_fields: BTreeMap<String, FieldDef> = details_schemas
        .iter()
        .map(|(path, info)| {
            (
                path.clone(),
                FieldDef {
                    field_type: info.field_type.clone(),
                    required: info.seen_count == info.total_samples,
                },
            )
        })
        .collect();

    endpoints.insert(
        "job_details".to_string(),
        EndpointBaseline {
            method: "GET".to_string(),
            path: "/pc/v4/jobdetails/{refnr_base64}".to_string(),
            fields: details_fields,
        },
    );

    Baseline {
        version: "1".to_string(),
        generated_at: chrono_now(),
        endpoints,
    }
}

// --- Test entry points ---

/// Main validation test — diffs live API against baseline.
/// This test is meant to run in the scheduled CI workflow.
#[test]
fn api_schema_validate() {
    // Check if we're in update mode
    if std::env::var("UPDATE_BASELINE").is_ok() {
        println!("UPDATE_BASELINE set — generating baseline instead of diffing");
        run_update_baseline();
        return;
    }

    // Load baseline
    let baseline_content = fs::read_to_string(BASELINE_PATH).unwrap_or_else(|e| {
        panic!(
            "Cannot read baseline at {}: {}.\nRun: UPDATE_BASELINE=1 cargo test --all-features --test api_schema_validation -- --test-threads=1",
            BASELINE_PATH, e
        )
    });
    let baseline: Baseline =
        serde_json::from_str(&baseline_content).expect("Failed to parse baseline JSON");

    let client = http_client();

    // Fetch multiple search samples
    println!("Fetching search samples...");
    let mut search_schemas = Vec::new();
    let mut refnrs = Vec::new();

    for page in 1..=3 {
        match fetch_search_raw(&client, page, 5) {
            Ok(value) => {
                // Collect refnrs for job details tests
                if let Some(jobs) = value.get("stellenangebote").and_then(|v| v.as_array()) {
                    for job in jobs {
                        if let Some(refnr) = job.get("refnr").and_then(|v| v.as_str()) {
                            refnrs.push(refnr.to_string());
                        }
                    }
                }
                let mut schema = BTreeMap::new();
                extract_schema(&value, "", &mut schema);
                search_schemas.push(schema);
                println!("  Search page {}: OK", page);
            }
            Err(e) => {
                eprintln!("  Search page {} failed: {} — API may be down", page, e);
                // If all fetches fail, exit with distinct message
                if page == 1 {
                    write_report(&[], Path::new(REPORT_PATH));
                    panic!("API unreachable on first request: {}", e);
                }
            }
        }
    }

    let merged_search = merge_schemas(&search_schemas);

    // Fetch job details samples
    println!("Fetching job details samples...");
    let mut details_schemas = Vec::new();
    let mut details_fetched = 0;

    for refnr in &refnrs {
        if details_fetched >= 2 {
            break;
        }
        match fetch_job_details_raw(&client, refnr) {
            Ok(value) => {
                let mut schema = BTreeMap::new();
                extract_schema(&value, "", &mut schema);
                details_schemas.push(schema);
                details_fetched += 1;
                println!("  Job details {}: OK", refnr);
            }
            Err(e) if e == "404_not_found" => {
                println!("  Job details {}: 404 (expired, skipping)", refnr);
            }
            Err(e) => {
                eprintln!("  Job details {} failed: {}", refnr, e);
            }
        }
    }

    let merged_details = merge_schemas(&details_schemas);

    // Diff against baseline
    println!("Diffing against baseline...");
    let mut all_changes = Vec::new();

    if let Some(search_baseline) = baseline.endpoints.get("search") {
        all_changes.extend(diff_schemas("search", search_baseline, &merged_search));
    }
    if let Some(details_baseline) = baseline.endpoints.get("job_details") {
        if !merged_details.is_empty() {
            all_changes.extend(diff_schemas("job_details", details_baseline, &merged_details));
        } else {
            println!("  Warning: No job details samples fetched — skipping details diff");
        }
    }

    // Pagination limit check
    println!("Checking pagination limits...");
    all_changes.extend(check_pagination_limit(&client));

    // Write report
    write_report(&all_changes, Path::new(REPORT_PATH));

    // Summary
    let critical = all_changes.iter().filter(|c| c.severity == "critical").count();
    let warnings = all_changes.iter().filter(|c| c.severity == "warning").count();
    let info = all_changes.iter().filter(|c| c.severity == "informational").count();

    println!();
    println!("=== Schema Validation Report ===");
    println!("Critical: {}", critical);
    println!("Warnings: {}", warnings);
    println!("Informational: {}", info);

    for change in &all_changes {
        let icon = match change.severity.as_str() {
            "critical" => "!!",
            "warning" => " !",
            _ => "  ",
        };
        println!("  [{}] {}: {} — {}", icon, change.endpoint, change.field_path, change.detail);
    }

    println!();
    println!("Report written to: {}", REPORT_PATH);

    // Fail only on critical changes
    assert!(
        critical == 0,
        "Found {} critical API schema change(s). See report at {}",
        critical,
        REPORT_PATH
    );
}

fn run_update_baseline() {
    let client = http_client();

    println!("Fetching search samples for baseline...");
    let mut search_schemas = Vec::new();
    let mut refnrs = Vec::new();

    for page in 1..=3 {
        match fetch_search_raw(&client, page, 10) {
            Ok(value) => {
                if let Some(jobs) = value.get("stellenangebote").and_then(|v| v.as_array()) {
                    for job in jobs {
                        if let Some(refnr) = job.get("refnr").and_then(|v| v.as_str()) {
                            refnrs.push(refnr.to_string());
                        }
                    }
                }
                let mut schema = BTreeMap::new();
                extract_schema(&value, "", &mut schema);
                search_schemas.push(schema);
                println!("  Search page {}: OK", page);
            }
            Err(e) => panic!("Failed to fetch search page {}: {}", page, e),
        }
    }

    let merged_search = merge_schemas(&search_schemas);

    println!("Fetching job details samples for baseline...");
    let mut details_schemas = Vec::new();
    let mut details_fetched = 0;

    for refnr in &refnrs {
        if details_fetched >= 3 {
            break;
        }
        match fetch_job_details_raw(&client, refnr) {
            Ok(value) => {
                let mut schema = BTreeMap::new();
                extract_schema(&value, "", &mut schema);
                details_schemas.push(schema);
                details_fetched += 1;
                println!("  Job details {}: OK", refnr);
            }
            Err(e) if e == "404_not_found" => {
                println!("  Job details {}: 404 (skipping)", refnr);
            }
            Err(e) => {
                println!("  Job details {} failed: {} (skipping)", refnr, e);
            }
        }
    }

    let merged_details = merge_schemas(&details_schemas);

    let baseline = generate_baseline(&merged_search, &merged_details);
    let json = serde_json::to_string_pretty(&baseline).expect("Failed to serialize baseline");
    fs::write(BASELINE_PATH, &json).expect("Failed to write baseline");

    println!();
    println!("Baseline written to: {}", BASELINE_PATH);
    println!("Search fields: {}", merged_search.len());
    println!("Job details fields: {}", merged_details.len());
    println!();
    println!("Review and commit: git add {} && git commit -m 'chore: update API schema baseline'", BASELINE_PATH);
}
```

- [ ] **Step 2: Verify compilation**

Run: `rustup run 1.93.0 cargo check --all-features --test api_schema_validation`
Expected: compiles with zero errors

- [ ] **Step 3: Commit**

```bash
git add tests/api_schema_validation.rs
git commit -m "test: add API schema validation test with extraction and diffing"
```

---

### Task 3: Generate initial baseline from live API

**Files:**
- Generated: `tests/fixtures/api-schema-baseline.json`

- [ ] **Step 1: Run baseline generation against live API**

```bash
UPDATE_BASELINE=1 rustup run 1.93.0 cargo test --all-features --test api_schema_validation -- --test-threads=1 2>/dev/null
```
Expected: prints field counts, writes baseline file

- [ ] **Step 2: Verify baseline was created and looks reasonable**

```bash
cat tests/fixtures/api-schema-baseline.json | python3 -m json.tool | head -40
```
Expected: JSON with version, generated_at, endpoints.search.fields, endpoints.job_details.fields

- [ ] **Step 3: Commit the baseline**

```bash
git add tests/fixtures/api-schema-baseline.json
git commit -m "chore: generate initial API schema baseline from live API"
```

---

### Task 4: Verify diff mode works (no changes expected)

- [ ] **Step 1: Run in diff mode against the baseline we just generated**

```bash
rustup run 1.93.0 cargo test --all-features --test api_schema_validation -- --test-threads=1 2>/dev/null
```
Expected: PASS with 0 critical, 0 warnings (baseline is fresh)

- [ ] **Step 2: Verify report was generated**

```bash
cat tests/fixtures/schema-diff-report.json | python3 -m json.tool | head -10
```
Expected: JSON with critical_count=0

---

### Task 5: Update CI workflow with schema validation and alerting

**Files:**
- Modify: `.github/workflows/api-smoke-test.yml`

- [ ] **Step 1: Update the workflow**

Replace the contents of `.github/workflows/api-smoke-test.yml` with:

```yaml
name: API Smoke Tests

on:
  schedule:
    # Run every 6 hours to detect API changes early
    - cron: '0 */6 * * *'
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  api-smoke-test:
    name: Live API smoke tests
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      - name: Enable cache
        uses: Swatinem/rust-cache@v2

      - name: Run integration tests against live API
        run: cargo test --all-features --test integration_test -- --ignored --test-threads=1
        continue-on-error: true

      - name: Run API contract tests
        run: cargo test --all-features --test api_contract_tests -- --test-threads=1
        continue-on-error: true

      - name: Run API schema validation
        id: schema-validation
        run: cargo test --all-features --test api_schema_validation -- --test-threads=1 2>&1 | tee schema-output.txt
        continue-on-error: true

      - name: Create GitHub issues for schema changes
        if: always() && hashFiles('tests/fixtures/schema-diff-report.json') != ''
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const path = 'tests/fixtures/schema-diff-report.json';

            if (!fs.existsSync(path)) {
              console.log('No schema diff report found — skipping alerting');
              return;
            }

            const report = JSON.parse(fs.readFileSync(path, 'utf8'));

            if (report.changes.length === 0) {
              console.log('No schema changes detected');
              return;
            }

            console.log(`Found ${report.critical_count} critical, ${report.warning_count} warning, ${report.informational_count} informational changes`);

            for (const change of report.changes) {
              // Determine labels based on severity
              const labels = change.severity === 'critical'
                ? ['api-breaking-change', 'critical']
                : change.severity === 'warning'
                  ? ['api-schema-change', 'warning']
                  : ['api-schema-change', 'informational'];

              // Check for existing open issue with same fingerprint
              const searchQuery = `repo:${context.repo.owner}/${context.repo.repo} is:issue is:open "${change.fingerprint}"`;
              const existing = await github.rest.search.issuesAndPullRequests({ q: searchQuery });

              if (existing.data.total_count > 0) {
                console.log(`Skipping duplicate: ${change.field_path} (${change.change_type})`);
                continue;
              }

              // Ensure labels exist
              for (const label of labels) {
                try {
                  await github.rest.issues.getLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    name: label,
                  });
                } catch {
                  const colors = {
                    'critical': 'B60205',
                    'api-breaking-change': 'B60205',
                    'warning': 'FBCA04',
                    'informational': '0E8A16',
                    'api-schema-change': '1D76DB',
                  };
                  await github.rest.issues.createLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    name: label,
                    color: colors[label] || 'CCCCCC',
                  });
                }
              }

              const severityEmoji = change.severity === 'critical' ? '!!' : change.severity === 'warning' ? '!' : 'i';
              const title = `[${severityEmoji}] API ${change.change_type}: ${change.endpoint}.${change.field_path}`;

              const body = [
                `## API Schema Change Detected`,
                ``,
                `| Property | Value |`,
                `|----------|-------|`,
                `| **Endpoint** | \`${change.endpoint}\` |`,
                `| **Field** | \`${change.field_path}\` |`,
                `| **Change** | ${change.change_type} |`,
                `| **Severity** | ${change.severity} |`,
                `| **Old type** | ${change.old_type || 'n/a'} |`,
                `| **New type** | ${change.new_type || 'n/a'} |`,
                ``,
                `### Detail`,
                change.detail,
                ``,
                `### Action Required`,
                change.severity === 'critical'
                  ? '**This is a breaking change.** Update the crate\'s response types to match the new API structure.'
                  : change.severity === 'warning'
                    ? 'Investigate whether this change affects crate functionality.'
                    : 'No immediate action required. Consider adding this field to the crate if useful.',
                ``,
                `### How to update baseline`,
                '```bash',
                'UPDATE_BASELINE=1 cargo test --all-features --test api_schema_validation -- --test-threads=1',
                'git add tests/fixtures/api-schema-baseline.json',
                'git commit -m "chore: update API schema baseline"',
                '```',
                ``,
                `---`,
                `Fingerprint: \`${change.fingerprint}\``,
                `Detected: ${report.generated_at}`,
              ].join('\n');

              await github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: title.substring(0, 256),
                body: body,
                labels: labels,
              });

              console.log(`Created issue: ${title}`);
            }
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/api-smoke-test.yml
git commit -m "ci: add schema validation with tiered GitHub Issue alerting"
```

---

### Task 6: End-to-end verification

- [ ] **Step 1: Run the full test suite to ensure nothing is broken**

```bash
rustup run 1.93.0 cargo test --all-features 2>&1 | grep "test result"
```
Expected: all existing tests still pass

- [ ] **Step 2: Run schema validation one more time**

```bash
rustup run 1.93.0 cargo test --all-features --test api_schema_validation -- --test-threads=1 2>/dev/null
```
Expected: PASS

- [ ] **Step 3: Verify the report has the pagination check results**

```bash
cat tests/fixtures/schema-diff-report.json | python3 -c "import sys,json; r=json.load(sys.stdin); print(f'Changes: {len(r[\"changes\"])}, Critical: {r[\"critical_count\"]}, Warning: {r[\"warning_count\"]}, Info: {r[\"informational_count\"]}')"
```

- [ ] **Step 4: Push and create PR**

```bash
git push -u origin HEAD
gh pr create --title "feat: API schema validation with tiered alerting" --body "..."
```
