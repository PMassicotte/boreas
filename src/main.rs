mod oceanographic_model;

use oceanographic_model::{OceanographicProcessor, processor::Bbox};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting oceanographic primary production processing...");

    let start = Instant::now();
    let processor = OceanographicProcessor::new()?;
    println!("{}", processor);

    println!("=== Starting the processos ===");
    let dims = processor.get_dim();
    println!("Original area: {}x{}", dims.0, dims.1);

    // ----------------
    let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3);
    let pp_values = processor.calculate_pp_for_bbox(bbox)?;

    println!("Baffin Bay PP values - Count: {}", pp_values.len());
    if !pp_values.is_empty() {
        println!(
            "  Min: {:.2} mg C m−2 d−1",
            pp_values.iter().fold(f32::INFINITY, |a, &b| a.min(b))
        );
        println!(
            "  Max: {:.2} mg C m−2 d−1",
            pp_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        );
        println!(
            "  Mean: {:.2} mg C m−2 d−1",
            pp_values.iter().sum::<f32>() / pp_values.len() as f32
        );
        println!(
            "  First 10 values: {:?}",
            pp_values.iter().take(10).collect::<Vec<&f32>>()
        );
    }

    println!("Time elapsed {:?}", start.elapsed());

    Ok(())
}
