//! Async job search example
//!
//! This example demonstrates how to use the async client to search for jobs.
//!
//! Run with: cargo run --example async_search --features async

use jobsuche::{Arbeitszeit, Credentials, JobsucheAsync, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    // Create an async client with the default API key
    let client = JobsucheAsync::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .await?;

    println!("🔍 Searching for Rust Developer jobs in Germany (async)...\n");

    // Build search options
    let options = SearchOptions::builder()
        .was("Rust Developer") // Job title
        .wo("Deutschland") // Location: Germany
        .arbeitszeit(vec![Arbeitszeit::Vollzeit]) // Full-time only
        .veroeffentlichtseit(30) // Posted in last 30 days
        .size(10) // Get 10 results
        .build();

    // Perform the async search
    let results = client.search().list(options).await?;

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

    // Get details for the first job (async)
    if let Some(first_job) = results.stellenangebote.first() {
        println!("\n📄 Getting details for first job (async)...\n");

        match client.job_details(&first_job.refnr).await {
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

    // Demonstrate concurrent requests (async advantage!)
    println!("\n🚀 Making 3 concurrent searches...\n");

    let searches = vec![
        SearchOptions::builder()
            .was("Python Developer")
            .size(5)
            .build(),
        SearchOptions::builder()
            .was("Java Developer")
            .size(5)
            .build(),
        SearchOptions::builder().was("Go Developer").size(5).build(),
    ];

    let handles: Vec<_> = searches
        .into_iter()
        .map(|opts| {
            let client_clone = client.clone();
            tokio::spawn(async move { client_clone.search().list(opts).await })
        })
        .collect();

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await? {
            Ok(result) => {
                let lang = ["Python", "Java", "Go"][i];
                println!("{} jobs: {}", lang, result.stellenangebote.len());
            }
            Err(e) => {
                eprintln!("Search failed: {}", e);
            }
        }
    }

    Ok(())
}
