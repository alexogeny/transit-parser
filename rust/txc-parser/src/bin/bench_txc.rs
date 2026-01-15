//! Simple benchmark for TXC parsing.

use std::env;
use std::time::Instant;
use txc_parser::TxcDocument;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("MET20250428(Schedule)v1-1.xml");

    println!("Benchmarking TXC parsing: {}", path);

    // Warmup
    println!("Warming up...");
    let _ = TxcDocument::from_path(path).expect("Failed to parse");

    // Benchmark
    let iterations = 5;
    let mut times = Vec::new();

    println!("\nRunning {} iterations...", iterations);
    for i in 0..iterations {
        let start = Instant::now();
        let doc = TxcDocument::from_path(path).expect("Failed to parse");
        let elapsed = start.elapsed();
        times.push(elapsed.as_millis());
        println!(
            "  [{}] Parsed {} services, {} stops, {} journeys, {} JP sections, {} JPs in {} ms",
            i + 1,
            doc.services.len(),
            doc.stop_points.len(),
            doc.vehicle_journeys.len(),
            doc.journey_pattern_sections.len(),
            doc.journey_patterns.len(),
            elapsed.as_millis()
        );
    }

    let mean: u128 = times.iter().sum::<u128>() / times.len() as u128;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();

    println!("\nResults:");
    println!("  Mean: {} ms", mean);
    println!("  Min:  {} ms", min);
    println!("  Max:  {} ms", max);
}
