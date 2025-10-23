//! Test Search API field mapping
//!
//! Run with: cargo run --example test_search_mapping

use jobsuche::{Credentials, Jobsuche, SearchOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )?;

    println!("Testing Job Search API mapping...\n");

    let options = SearchOptions::builder()
        .was("Softwareentwickler")
        .wo("Wuppertal")
        .size(2)
        .build();

    let results = client.search().list(options)?;

    println!("✅ Found {} jobs\n", results.stellenangebote.len());

    for (i, job) in results.stellenangebote.iter().enumerate() {
        println!("Job {}:", i + 1);
        println!("  refnr: {}", job.refnr);
        println!("  beruf: {}", job.beruf);
        println!("  titel: {:?}", job.titel);
        println!("  arbeitgeber: {}", job.arbeitgeber);
        println!("  aktuelle_veroeffentlichungsdatum: {:?}", job.aktuelle_veroeffentlichungsdatum);
        println!("  eintrittsdatum: {:?}", job.eintrittsdatum);
        println!("  modifikations_timestamp: {:?}", job.modifikations_timestamp);
        println!("  kundennummer_hash: {:?}", job.kundennummer_hash);

        println!("  Location:");
        println!("    ort: {:?}", job.arbeitsort.ort);
        println!("    plz: {:?}", job.arbeitsort.plz);
        println!("    region: {:?}", job.arbeitsort.region);
        println!("    land: {:?}", job.arbeitsort.land);
        println!("    entfernung: {:?}", job.arbeitsort.entfernung);

        if let Some(coords) = &job.arbeitsort.koordinaten {
            println!("    koordinaten: lat={}, lon={}", coords.lat, coords.lon);
        }

        println!();
    }

    Ok(())
}
