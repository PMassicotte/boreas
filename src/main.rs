mod bbox;
mod config;
mod date_gen;
mod iop;
mod lut;
mod oceanographic_model;
mod sat_bands;
mod utils;

use config::Config;
use oceanographic_model::batch_runner::BatchRunner;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("Starting oceanographic primary production processing...");

    let config = Config::from_file("./data/config/simple_config.json").unwrap();

    let processor = BatchRunner::new(config);
    let output_files = processor.process()?;

    println!(
        "\n✅ Processing completed! Generated {} output files:",
        output_files.len()
    );

    for file in &output_files {
        println!("  📁 {}", file);
    }

    println!("Time elapsed {:>.2?}", Instant::now() - start);
    Ok(())
}
