//! Async stream pagination example
//!
//! This example demonstrates memory-efficient job searching using streams.
//! Unlike collecting all results into a Vec, streams process jobs one at a time.
//!
//! Run with: cargo run --example async_stream --features async

use futures::StreamExt;
use jobsuche::{Credentials, JobsucheAsync, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let client = JobsucheAsync::new(
        "https://rest.arbeitsagentur.de/jobboerse/jobsuche-service",
        Credentials::default(),
    )
    .await?;

    println!("🌊 Stream-based job search - constant memory usage!\n");

    // Example 1: Process all jobs with streaming
    println!("Example 1: Processing jobs one at a time\n");

    let options = SearchOptions::builder()
        .was("Developer")
        .wo("Deutschland")
        .veroeffentlichtseit(7) // Last 7 days
        .size(10) // Fetch 10 per page
        .build();

    let mut stream = client.search().stream(options);
    let mut count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(job) => {
                count += 1;
                println!(
                    "{}. {} at {} ({})",
                    count,
                    job.beruf,
                    job.arbeitgeber,
                    job.arbeitsort.ort.as_deref().unwrap_or("unknown")
                );
            }
            Err(e) => {
                eprintln!("Error fetching job: {}", e);
                break;
            }
        }
    }

    println!("\nProcessed {} jobs total\n", count);

    // Example 2: Early termination (stop after finding 5 jobs)
    println!("Example 2: Early termination - only process first 5\n");

    let options = SearchOptions::builder().was("Rust").wo("Berlin").build();

    let mut stream = client.search().stream(options).take(5);
    let mut count = 0;

    while let Some(result) = stream.next().await {
        if let Ok(job) = result {
            count += 1;
            println!("{}. {}", count, job.beruf);
        }
    }

    println!("\nStopped after {} jobs (didn't fetch all pages!)\n", count);

    // Example 3: Filter with stream combinators
    println!("Example 3: Filter for Senior positions\n");

    let options = SearchOptions::builder()
        .was("Software Engineer")
        .wo("München")
        .build();

    let mut senior_jobs = client
        .search()
        .stream(options)
        .filter(|result| {
            // Filter for jobs containing "Senior"
            futures::future::ready(matches!(result, Ok(job) if job.beruf.contains("Senior")))
        })
        .take(10); // Only take first 10 senior positions

    let mut count = 0;
    while let Some(result) = senior_jobs.next().await {
        if let Ok(job) = result {
            count += 1;
            println!("{}. {} at {}", count, job.beruf, job.arbeitgeber);
        }
    }

    println!("\nFound {} senior positions\n", count);

    // Example 4: Concurrent streaming from multiple searches
    println!("Example 4: Concurrent streams\n");

    let searches = vec![
        (
            "Python",
            SearchOptions::builder().was("Python").size(5).build(),
        ),
        ("Java", SearchOptions::builder().was("Java").size(5).build()),
        ("Go", SearchOptions::builder().was("Go").size(5).build()),
    ];

    for (lang, opts) in searches {
        let mut stream = client.search().stream(opts);
        let mut count = 0;

        while let Some(result) = stream.next().await {
            if result.is_ok() {
                count += 1;
            }
        }

        println!("{} jobs: {}", lang, count);
    }

    // Example 5: Collect specific number of results
    println!("\nExample 5: Collect exactly 25 jobs\n");

    let options = SearchOptions::builder()
        .was("JavaScript")
        .wo("Hamburg")
        .build();

    let jobs: Vec<_> = client
        .search()
        .stream(options)
        .take(25)
        .filter_map(|result| async move { result.ok() }) // Filter out errors
        .collect()
        .await;

    println!("Collected {} jobs", jobs.len());
    for (i, job) in jobs.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, job.beruf);
    }
    if jobs.len() > 5 {
        println!("  ... and {} more", jobs.len() - 5);
    }

    println!("\n✅ Stream examples complete!");
    println!("\nMemory efficiency:");
    println!("  - Stream: O(1) memory - constant, just current page");
    println!("  - iter(): O(n) memory - all results loaded at once");

    Ok(())
}
