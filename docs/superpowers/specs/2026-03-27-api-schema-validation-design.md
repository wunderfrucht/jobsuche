# API Schema Validation & Auto-Alerting

## Problem

The jobsuche crate depends on the Bundesagentur fur Arbeit REST API response structure. If the API silently adds, removes, or changes fields, the crate may break without warning. We need automated detection of schema drift with tiered alerting.

## Design

### Layer 1: Schema Baseline

A JSON file at `tests/fixtures/api-schema-baseline.json` describes every field the API returns for each endpoint, including type and required/optional status.

Structure:
```json
{
  "version": "1",
  "generated_at": "2026-03-27T00:00:00Z",
  "endpoints": {
    "search": {
      "method": "GET",
      "path": "/pc/v4/jobs",
      "fields": {
        "stellenangebote": { "type": "array", "required": true },
        "maxErgebnisse": { "type": "number", "required": false },
        "page": { "type": "number", "required": false },
        "size": { "type": "number", "required": false },
        "facetten": { "type": "object", "required": false },
        "stellenangebote[].refnr": { "type": "string", "required": true },
        "stellenangebote[].beruf": { "type": "string", "required": true },
        "stellenangebote[].arbeitgeber": { "type": "string", "required": true },
        "stellenangebote[].arbeitsort": { "type": "object", "required": true },
        "stellenangebote[].hashId": { "type": "string", "required": false },
        "stellenangebote[].titel": { "type": "string", "required": false },
        "stellenangebote[].aktuelleVeroeffentlichungsdatum": { "type": "string", "required": false },
        "stellenangebote[].eintrittsdatum": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.plz": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.ort": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.strasse": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.region": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.land": { "type": "string", "required": false },
        "stellenangebote[].arbeitsort.koordinaten": { "type": "object", "required": false },
        "stellenangebote[].arbeitsort.koordinaten.lat": { "type": "number", "required": false },
        "stellenangebote[].arbeitsort.koordinaten.lon": { "type": "number", "required": false },
        "stellenangebote[].arbeitsort.entfernung": { "type": "string", "required": false },
        "stellenangebote[].modifikationsTimestamp": { "type": "string", "required": false },
        "stellenangebote[].externeUrl": { "type": "string", "required": false },
        "stellenangebote[].kundennummerHash": { "type": "string", "required": false }
      }
    },
    "job_details": {
      "method": "GET",
      "path": "/pc/v4/jobs/{refnr_base64}",
      "fields": {
        "referenznummer": { "type": "string", "required": false },
        "stellenangebotsTitel": { "type": "string", "required": false },
        "stellenangebotsart": { "type": "string", "required": false },
        "firma": { "type": "string", "required": false },
        "arbeitgeberKundennummerHash": { "type": "string", "required": false },
        "hauptberuf": { "type": "string", "required": false },
        "stellenangebotsBeschreibung": { "type": "string", "required": false },
        "stellenlokationen": { "type": "array", "required": false },
        "stellenlokationen[].adresse": { "type": "object", "required": false },
        "stellenlokationen[].adresse.plz": { "type": "string", "required": false },
        "stellenlokationen[].adresse.ort": { "type": "string", "required": false },
        "stellenlokationen[].adresse.region": { "type": "string", "required": false },
        "stellenlokationen[].adresse.land": { "type": "string", "required": false },
        "stellenlokationen[].breite": { "type": "number", "required": false },
        "stellenlokationen[].laenge": { "type": "number", "required": false },
        "arbeitszeitVollzeit": { "type": "boolean", "required": false },
        "verguetungsangabe": { "type": "string", "required": false },
        "vertragsdauer": { "type": "string", "required": false },
        "eintrittszeitraum": { "type": "object", "required": false },
        "eintrittszeitraum.von": { "type": "string", "required": false },
        "eintrittszeitraum.bis": { "type": "string", "required": false },
        "veroeffentlichungszeitraum": { "type": "object", "required": false },
        "veroeffentlichungszeitraum.von": { "type": "string", "required": false },
        "veroeffentlichungszeitraum.bis": { "type": "string", "required": false },
        "datumErsteVeroeffentlichung": { "type": "string", "required": false },
        "aenderungsdatum": { "type": "string", "required": false },
        "istBetreut": { "type": "boolean", "required": false },
        "istBehinderungGefordert": { "type": "boolean", "required": false },
        "istGeringfuegigeBeschaeftigung": { "type": "boolean", "required": false },
        "istArbeitnehmerUeberlassung": { "type": "boolean", "required": false },
        "istPrivateArbeitsvermittlung": { "type": "boolean", "required": false },
        "quereinstiegGeeignet": { "type": "boolean", "required": false },
        "allianzpartnerName": { "type": "string", "required": false },
        "allianzpartnerUrl": { "type": "string", "required": false },
        "chiffrenummer": { "type": "string", "required": false }
      }
    }
  }
}
```

### Layer 2: Rust Schema Validation Test

File: `tests/api_schema_validation.rs`

The test:
1. Fetches live API responses as raw `serde_json::Value` (bypassing our typed structs to detect fields we don't model)
2. Walks the JSON recursively to extract the actual schema (field paths, types)
3. Loads the baseline from `tests/fixtures/api-schema-baseline.json`
4. Diffs actual vs baseline:
   - **Removed field**: in baseline but missing from live response across multiple samples = `critical`
   - **Type changed**: field present but different JSON type = `critical`
   - **New field**: in response but not in baseline = `informational`
5. Writes `schema-diff-report.json` (gitignored) with structured results
6. The test itself **fails only on critical changes** (removed or type-changed fields)
7. New fields are logged as warnings but do not fail the test

**Baseline update command:** When a schema change is accepted, run:
```bash
cargo test --all-features --test api_schema_validation -- --update-baseline --ignored --test-threads=1
```
This regenerates the baseline file from the live API. Commit the updated baseline.

**Multiple samples:** To distinguish "field removed from API" from "field not present on this particular job", the test fetches 3 search results and 2 job details. A field is only considered removed if it's absent from ALL samples AND was marked `required: true` in the baseline. Optional fields missing from all samples are flagged as `warning` not `critical`.

### Layer 3: CI Alerting

Updated `.github/workflows/api-smoke-test.yml` adds:

1. **Schema validation step** — runs the validation test, captures the diff report
2. **Alerting step** — uses `actions/github-script@v7` to:
   - Read `schema-diff-report.json`
   - For each change, compute a fingerprint (SHA256 of change_type + field_path)
   - Search open issues for existing issue with that fingerprint in the body
   - If no duplicate found:
     - Critical changes: create issue with labels `api-breaking-change`, `critical`
     - Informational changes: create issue with labels `api-schema-change`, `informational`
   - Issue body includes: field path, change type, old/new type, detection timestamp, fingerprint

3. **API unreachable handling** — if the API is down (connection error, not schema change), the test exits with a distinct exit code. The CI step only creates an `api-unreachable` issue after checking that the previous run also failed (via workflow artifacts).

### File Structure

| File | Purpose | Committed |
|------|---------|-----------|
| `tests/api_schema_validation.rs` | Schema extraction, diff, report generation | Yes |
| `tests/fixtures/api-schema-baseline.json` | Expected API schema | Yes |
| `tests/fixtures/schema-diff-report.json` | Generated diff report | No (gitignored) |
| `.github/workflows/api-smoke-test.yml` | Updated with validation + alerting | Yes |

### No New Dependencies

All schema comparison uses `serde_json::Value` (already a dependency). No new crates needed.

### Error Handling

- API down: distinct from schema change, separate issue label
- Rate limited: tests use `--test-threads=1` and 500ms delays between requests
- Flaky fields: optional fields missing from some responses are not flagged as removed
- Baseline out of date: clear error message pointing to the update command
