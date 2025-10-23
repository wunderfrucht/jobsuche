//! Test API field mapping
//!
//! Run with: cargo run --example test_api_mapping

use jobsuche::{Credentials, Jobsuche};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Jobsuche::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )?;

    let refnr = "12265-443677_JB4946295-S";
    println!("Testing job_details for: {}", refnr);

    match client.job_details(refnr) {
        Ok(details) => {
            println!("\n✅ SUCCESS! Got job details:");
            println!("  refnr: {:?}", details.refnr);
            println!("  titel: {:?}", details.titel);
            println!("  arbeitgeber: {:?}", details.arbeitgeber);
            println!("  hauptberuf: {:?}", details.hauptberuf);

            if let Some(desc) = &details.stellenbeschreibung {
                println!("  stellenbeschreibung: {} chars", desc.len());
                println!("  First 100 chars: {}", &desc[..100.min(desc.len())]);
            } else {
                println!("  stellenbeschreibung: NONE");
            }

            println!("  arbeitsorte: {} locations", details.arbeitsorte.len());
            for (i, loc) in details.arbeitsorte.iter().enumerate() {
                if let Some(addr) = &loc.adresse {
                    println!("    Location {}: {:?}, {:?}", i + 1, addr.ort, addr.plz);
                }
            }

            println!("  verguetung: {:?}", details.verguetung);
            println!("  vertragsdauer: {:?}", details.vertragsdauer);
            println!("  allianzpartner: {:?}", details.allianzpartner);
            println!(
                "  ist_arbeitnehmer_ueberlassung: {:?}",
                details.ist_arbeitnehmer_ueberlassung
            );

            Ok(())
        }
        Err(e) => {
            println!("\n❌ ERROR: {}", e);
            Err(e.into())
        }
    }
}
