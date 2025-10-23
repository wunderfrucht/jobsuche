//! Test Facets parsing
//!
//! Run with: cargo run --example test_facets

use jobsuche::{Credentials, Jobsuche, SearchOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )?;

    let options = SearchOptions::builder()
        .was("Softwareentwickler")
        .size(1)
        .build();

    let results = client.search().list(options)?;

    println!("Response Metadata:");
    println!("  max_ergebnisse: {:?}", results.max_ergebnisse);
    println!("  page: {:?}", results.page);
    println!("  size: {:?}", results.size);
    println!("  facetten: {}", if results.facetten.is_some() { "Present" } else { "None" });

    if let Some(facetten) = &results.facetten {
        println!("\nFacets data:");
        println!("{}", serde_json::to_string_pretty(facetten)?);
    }

    Ok(())
}
