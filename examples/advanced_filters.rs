//! Advanced filtering example
//!
//! This example shows how to use type-safe filters for precise job searches.
//!
//! Run with: cargo run --example advanced_filters

use jobsuche::{Angebotsart, Arbeitszeit, Befristung, Credentials, Jobsuche, SearchOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )?;

    println!("🎯 Searching with advanced filters...\n");

    // Search for full-time, permanent positions as a Data Scientist in Munich area
    let options = SearchOptions::builder()
        .was("Data Scientist")
        .wo("München")
        .umkreis(50) // 50km radius
        .angebotsart(Angebotsart::Arbeit) // Regular employment (not self-employment)
        .befristung(vec![Befristung::Unbefristet]) // Permanent contract only
        .arbeitszeit(vec![
            Arbeitszeit::Vollzeit,       // Full-time
            Arbeitszeit::HeimTelearbeit, // Or remote
        ])
        .veroeffentlichtseit(14) // Posted in last 2 weeks
        .zeitarbeit(false) // Exclude temporary employment agencies
        .behinderung(false) // General positions (not specifically for people with disabilities)
        .size(20)
        .build();

    let results = client.search().list(options)?;

    println!(
        "Found {} matching positions:\n",
        results.stellenangebote.len()
    );

    for job in results.stellenangebote.iter().take(10) {
        println!("• {} - {}", job.beruf, job.arbeitgeber);
        println!(
            "  📍 {}, {} ({}km away)",
            job.arbeitsort.ort.as_deref().unwrap_or("unknown"),
            job.arbeitsort.region.as_deref().unwrap_or("unknown"),
            job.arbeitsort.entfernung.as_deref().unwrap_or("?")
        );

        if let Some(date) = &job.aktuelle_veroeffentlichungsdatum {
            println!("  📅 Published: {}", date);
        }

        println!();
    }

    // Show facets (if available) for further filtering ideas
    if let Some(facets) = results.facetten {
        println!("\n📊 Available facets for further filtering:");
        println!("{:#?}", facets);
    }

    Ok(())
}
