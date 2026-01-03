mod bbox;
mod config;
mod date_gen;
mod iop;
mod lut;
mod models;
mod oceanographic_model;
mod sat_bands;
mod traits;
mod utils;

use config::Config;
use models::VgpmModel;
use oceanographic_model::batch_runner::BatchRunner;
use std::time::Instant;

use colored::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!(
        "{}",
        "Starting oceanographic primary production processing...".bright_yellow()
    );

    let config = Config::from_file("./data/config/simple_config.json").unwrap();
    let runner = BatchRunner::new(config);

    // Run with VGPM model (default)
    // TODO: Maybe add the model to use via config file?
    println!("\n{}", "Running VGPM model...".bright_cyan());
    let vgpm = VgpmModel::new();
    let output_files = runner.run_algo(&vgpm)?;

    // Uncomment to run with CbPM model instead:
    // (Also add `use models::CbpmModel;` to imports)
    // println!("\n{}", "Running CbPM model...".bright_cyan());
    // let cbpm = CbpmModel::new();
    // let output_files = runner.run_algo(&cbpm)?;

    println!(
        "\n✅ Processing completed! Generated {} output files:",
        output_files.len()
    );

    for file in &output_files {
        println!("  📁 {}", file);
    }

    println!("\nTime elapsed {:>.2?}", Instant::now() - start);
    Ok(())
}
