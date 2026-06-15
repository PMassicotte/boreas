mod bbox;
mod config;
mod date_gen;
mod error;
mod iop;
mod lut;
mod models;
mod oceanographic_model;
mod sat_bands;
mod traits;
mod utils;

use crate::models::{CbpmModel, VgpmModel};
use config::Config;
use oceanographic_model::batch_runner::BatchRunner;
use std::time::Instant;
use traits::PrimaryProduction;

use colored::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!(
        "{}",
        "Starting oceanographic primary production processing...".bright_yellow()
    );

    let config = Config::from_file("./data/config/simple_config.json").unwrap();

    // Select algorithm from config
    let model: Box<dyn PrimaryProduction> = match config.algorithm.as_str() {
        "vgpm" => {
            println!("\n{}", "Running VGPM model...".bright_cyan());
            Box::new(VgpmModel::new())
        }
        "cbpm" => {
            println!("\n{}", "Running CbPM model...".bright_cyan());
            Box::new(CbpmModel::new())
        }
        _ => {
            return Err(format!("Unknown algorithm: {}", config.algorithm).into());
        }
    };

    let runner = BatchRunner::new(&config);
    let output_files = runner.run_algo(model.as_ref())?;

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
