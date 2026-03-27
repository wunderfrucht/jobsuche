# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-03-27

### Breaking

- `JobListing.beruf` changed from `String` to `Option<String>` (API now sometimes omits this field)
- `JobListing.arbeitgeber` changed from `String` to `Option<String>` (defensive, same API behavior expected)
- Removed `reqwest-middleware` and `reqwest-retry` dependencies — async client now uses manual retry loop with proper Retry-After support
- `Error::Middleware` variant removed from error enum

### Changed

- MSRV bumped from 1.85.1 to 1.89.0
- reqwest 0.12 → 0.13
- Async client now respects Retry-After headers (previously ignored by middleware)

### Added

- Property-based testing with proptest
- Nightly sanitizer CI (AddressSanitizer, ThreadSanitizer, LeakSanitizer)
- Mutation testing with cargo-mutants (nightly scheduled)
- API schema validation with tiered alerting (every 6 hours)
- STPA safety analysis and cybersecurity analysis (rivet, 76 artifacts)
- Pre-commit hooks matching CI checks (9 hooks)
- Community-maintained OpenAPI 3.0.3 spec (`docs/openapi.yaml`)
- Custom Debug impl for Credentials that redacts API key (CSREQ-001)
- Input validation warnings for `encode_refnr` (CSREQ-004)
- Warning log when Retry-After header is unparseable (LS-003)

### Fixed

- Pagination investigation confirmed 100-page hard limit
- Async client now properly handles rate limiting with Retry-After (CC-003)

### Removed

- `instant` transitive dependency (via reqwest-retry 0.9)
- `reqwest-middleware` and `reqwest-retry` dependencies (replaced with manual retry)
- `openssl-probe` from deny ban list (required by reqwest 0.13)
