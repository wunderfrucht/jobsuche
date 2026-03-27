# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-03-27

### Breaking

- `JobListing.beruf` changed from `String` to `Option<String>` (API now sometimes omits this field)

### Changed

- MSRV bumped from 1.85.1 to 1.89.0
- reqwest 0.12 → 0.13, reqwest-middleware 0.4 → 0.5, reqwest-retry 0.7 → 0.9

### Added

- Property-based testing with proptest
- Nightly sanitizer CI (AddressSanitizer, ThreadSanitizer, LeakSanitizer)
- Mutation testing with cargo-mutants (nightly scheduled)
- API schema validation with tiered alerting
- STPA safety analysis and cybersecurity analysis (rivet)
- Pre-commit hooks matching CI checks

### Fixed

- Pagination investigation confirmed 100-page hard limit

### Removed

- `instant` transitive dependency (via reqwest-retry 0.9)
- `openssl-probe` from deny ban list (required by reqwest 0.13)
