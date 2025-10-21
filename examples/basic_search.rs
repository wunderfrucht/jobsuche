//! Basic job search example
//!
//! This example demonstrates how to search for jobs using the Jobsuche API.
//!
//! Run with: cargo run --example basic_search

use jobsuche::{Arbeitszeit, Credentials, Jobsuche, SearchOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (optional - uncomment if you want debug output)
    // tracing_subscriber::fmt::init();

    // Create a client with the default API key
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )?;

    println!("🔍 Searching for Rust Developer jobs in Germany...\n");

    // Build search options
    let options = SearchOptions::builder()
        .was("Rust Developer") // Job title
        .wo("Deutschland") // Location: Germany
        .arbeitszeit(vec![Arbeitszeit::Vollzeit]) // Full-time only
        .veroeffentlichtseit(30) // Posted in last 30 days
        .size(10) // Get 10 results
        .build();

    // Perform the search
    let results = client.search().list(options)?;

    println!("Found {} jobs:\n", results.stellenangebote.len());

    // Display results
    for (i, job) in results.stellenangebote.iter().enumerate() {
        println!("{}. {}", i + 1, job.beruf);
        println!("   Company: {}", job.arbeitgeber);
        println!(
            "   Location: {}, {}",
            job.arbeitsort.ort.as_deref().unwrap_or("unknown"),
            job.arbeitsort.region.as_deref().unwrap_or("unknown")
        );
        if let Some(date) = &job.aktuelle_veroeffentlichungsdatum {
            println!("   Published: {}", date);
        }
        println!("   Reference: {}", job.refnr);
        println!();
    }

    // Get details for the first job
    if let Some(first_job) = results.stellenangebote.first() {
        println!("\n📄 Getting details for first job...\n");

        match client.job_details(&first_job.refnr) {
            Ok(details) => {
                if let Some(title) = &details.titel {
                    println!("Title: {}", title);
                }
                if let Some(employer) = &details.arbeitgeber {
                    println!("Employer: {}", employer);
                }

                if let Some(desc) = details.stellenbeschreibung {
                    let preview = if desc.len() > 200 {
                        format!("{}...", &desc[..200])
                    } else {
                        desc
                    };
                    println!("Description: {}", preview);
                }

                if let Some(salary) = details.verguetung {
                    println!("Salary: {}", salary);
                }

                println!("\nWork locations:");
                for location in &details.arbeitsorte {
                    println!(
                        "  - {}, {} {}",
                        location.ort.as_deref().unwrap_or("unknown"),
                        location.plz.as_deref().unwrap_or(""),
                        location.region.as_deref().unwrap_or("unknown")
                    );
                }

                if !details.fertigkeiten.is_empty() {
                    println!("\nRequired skills:");
                    for skill in &details.fertigkeiten {
                        println!("  - {}", skill.hierarchie_name);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Could not get job details: {}", e);
                println!("(This is common - jobs expire quickly)");
            }
        }
    }

    Ok(())
}
