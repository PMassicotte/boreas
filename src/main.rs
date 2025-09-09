mod oceanographic_model;

use oceanographic_model::{OceanographicProcessor, processor::Bbox};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting oceanographic primary production processing...");

    let start = Instant::now();
    let processor = OceanographicProcessor::new()?;

    println!("{}", processor);

    println!("Loaded rasters in {:?}", start.elapsed());

    let start = Instant::now();
    println!("=== Starting the processos ===");
    let dims = processor.get_dim();
    println!("Original area: {}x{}", dims.0, dims.1);

    // let pp = processor.calculate_region_pp(0, 0, dims.0, dims.1).unwrap();
    let pp = processor.calculate_region_pp(0, 0, 100, 100).unwrap();

    println!("Processed {} pixels", pp.len());
    println!(
        "Number of valid pixels {}",
        processor.get_valid_pixel_count()
    );

    println!("PP calculated in {:?}", start.elapsed());

    // ----------------
    let bbox = Bbox::new(-90.0, 90.0, 45.0, 90.0);
    let pp_values = processor.calculate_pp_for_bbox(bbox)?;

    println!("test {:?}", pp_values);

    Ok(())
}
