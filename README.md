# jobsuche

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/wunderfrucht/jobsuche/graph/badge.svg?token=riQYXs6xgb)](https://codecov.io/gh/wunderfrucht/jobsuche)

> A Rust interface for the [Bundesagentur für Arbeit Jobsuche API](https://jobsuche.api.bund.dev/)

Access Germany's largest job database programmatically. Search for jobs, get detailed information, and access employer data through a type-safe, ergonomic Rust API.

## Features

- 🔍 **Rich Job Search**: Filter by location, job title, employment type, contract type, and more
- 📄 **Detailed Job Information**: Get comprehensive details about specific job postings
- 🏢 **Employer Data**: Access employer logos and information
- 🔄 **Automatic Pagination**: Iterate over all results seamlessly
- 🦀 **Type-Safe API**: Strong typing with enums for all parameters
- ⚡ **Sync & Async**: Both synchronous and asynchronous clients
- 🎯 **Based on gouqi**: Built with the same battle-tested patterns as [gouqi](https://github.com/wunderfrucht/gouqi)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jobsuche = "0.1"

# Optional: Enable async support
jobsuche = { version = "0.1", features = ["async"] }
```

## Quick Start

```rust
use jobsuche::{Jobsuche, Credentials, SearchOptions, Arbeitszeit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client (uses the public API key by default)
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default()
    )?;

    // Search for jobs
    let results = client.search().list(SearchOptions::builder()
        .was("Softwareentwickler")           // What: job title
        .wo("Berlin")                        // Where: location
        .umkreis(50)                         // Radius: 50km
        .arbeitszeit(vec![Arbeitszeit::Vollzeit])  // Full-time only
        .veroeffentlichtseit(30)             // Posted in last 30 days
        .size(25)                            // 25 results per page
        .build()
    )?;

    println!("Found {} jobs", results.stellenangebote.len());

    // Get details for the first job
    if let Some(job) = results.stellenangebote.first() {
        println!("Job: {} at {}", job.beruf, job.arbeitgeber);
        println!("Location: {}", job.arbeitsort.ort);

        // Fetch full details
        let details = client.job_details(&job.refnr)?;
        println!("Description: {:?}", details.stellenbeschreibung);
    }

    Ok(())
}
```

## Usage Examples

### Advanced Filtering

```rust
use jobsuche::{SearchOptions, Arbeitszeit, Befristung, Angebotsart};

let options = SearchOptions::builder()
    .was("Data Scientist")
    .wo("München")
    .umkreis(100)                           // 100km radius
    .angebotsart(Angebotsart::Arbeit)       // Regular employment
    .befristung(vec![Befristung::Unbefristet])  // Permanent only
    .arbeitszeit(vec![
        Arbeitszeit::Vollzeit,
        Arbeitszeit::Teilzeit,
    ])
    .veroeffentlichtseit(14)                // Last 2 weeks
    .zeitarbeit(false)                      // Exclude temp agencies
    .build();

let results = client.search().list(options)?;
```

### Pagination

```rust
// Manual pagination
for page in 1..=5 {
    let results = client.search().list(SearchOptions::builder()
        .was("Rust Developer")
        .page(page)
        .size(50)
        .build()
    )?;

    for job in results.stellenangebote {
        println!("{}: {}", job.refnr, job.beruf);
    }
}

// Automatic pagination - get all results
let all_jobs = client.search().iter(SearchOptions::builder()
    .was("DevOps Engineer")
    .wo("Hamburg")
    .veroeffentlichtseit(7)  // Limit to last week to avoid too many results
    .build()
)?;

println!("Found {} total jobs", all_jobs.len());
```

### Job Details

```rust
let job_listing = /* ... from search results ... */;

// Get comprehensive job information
let details = client.job_details(&job_listing.refnr)?;

println!("Title: {}", details.titel);
println!("Employer: {}", details.arbeitgeber);
println!("Locations: {:?}", details.arbeitsorte);
println!("Work time models: {:?}", details.arbeitszeitmodelle);
println!("Salary: {:?}", details.verguetung);
println!("Skills: {:?}", details.fertigkeiten);
```

### Employer Logos

```rust
use std::fs::File;
use std::io::Write;

let job = /* ... from search results ... */;

// Try to get employer logo (many employers don't have one)
if let Some(hash) = &job.kundennummer_hash {
    match client.employer_logo(hash) {
        Ok(logo_bytes) => {
            let mut file = File::create("logo.png")?;
            file.write_all(&logo_bytes)?;
            println!("Logo saved!");
        }
        Err(_) => println!("No logo available"),
    }
}
```

## Type-Safe Parameters

### Employment Types (Angebotsart)

```rust
use jobsuche::Angebotsart;

Angebotsart::Arbeit                // Regular employment
Angebotsart::Selbstaendigkeit      // Self-employment
Angebotsart::Ausbildung            // Apprenticeship/Dual study
Angebotsart::PraktikumTrainee      // Internship/Trainee
```

### Contract Types (Befristung)

```rust
use jobsuche::Befristung;

Befristung::Befristet      // Fixed-term contract
Befristung::Unbefristet    // Permanent contract
```

### Working Time (Arbeitszeit)

```rust
use jobsuche::Arbeitszeit;

Arbeitszeit::Vollzeit                        // Full-time
Arbeitszeit::Teilzeit                        // Part-time
Arbeitszeit::SchichtNachtarbeitWochenende    // Shift/Night/Weekend
Arbeitszeit::HeimTelearbeit                  // Home office/Remote
Arbeitszeit::Minijob                         // Mini job
```

## Known API Quirks

Based on [GitHub issues](https://github.com/bundesAPI/jobsuche-api/issues):

1. **404 Errors (Issue #61)**: Job details may return 404 even if the job appears in search results. Jobs expire quickly.

2. **403 Errors (Issue #60)**: Sporadic rate limiting may occur. The client will return `Error::Forbidden` in these cases.

3. **Employer Search (Issue #52)**: Case-sensitive and exact-match only:
   - ✅ Works: `"Deutsche Bahn AG"`
   - ❌ Doesn't work: `"deutsche bahn"` or `"bahn"`

4. **Employer Logos (Issue #62)**: Many employers don't have logos. Expect frequent 404s.

5. **No Sorting (Issue #43)**: Results are always sorted oldest-to-newest. No way to change this.

6. **RefNr Encoding**: Reference numbers must be base64-encoded for the job details endpoint. This client handles this automatically.

## Architecture

This crate follows the architecture patterns from [gouqi](https://github.com/wunderfrucht/gouqi), featuring:

- **Modular design**: Separate modules for different API resources
- **Builder pattern**: Ergonomic query construction
- **Error handling**: Comprehensive error types with `thiserror`
- **Feature flags**: Optional async, caching, and metrics support
- **Type safety**: Strong typing throughout

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details

## Related Projects

- [gouqi](https://github.com/wunderfrucht/gouqi) - Rust interface for Jira (inspiration for this crate)
- [bundesAPI](https://github.com/bundesAPI) - Collection of German government APIs
- [jobsuche-api](https://github.com/bundesAPI/jobsuche-api) - Official API documentation

## Acknowledgments

- Built by [wunderfrucht](https://github.com/wunderfrucht)
- Architecture inspired by [gouqi](https://github.com/wunderfrucht/gouqi)
- Data provided by [Bundesagentur für Arbeit](https://www.arbeitsagentur.de/)
