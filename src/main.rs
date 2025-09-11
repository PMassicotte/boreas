mod bbox;
mod config;
mod date_gen;
mod iop;
mod lut;
mod oceanographic_model;
mod sat_bands;

use bbox::Bbox;
use config::Config;
// use date_gen::DateTimeGenerator;
use oceanographic_model::batch_process::BatchProcessor;
// use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting oceanographic primary production processing...");

    let config = Config::from_file("./data/config/simple_config.json").unwrap();
    // let date_gen = DateTimeGenerator::new(config);
    // let dates = date_gen.generate_date_series();
    // println!("{:?}", dates);
    //
    let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3)?;

    // TODO: I think process should return a gdal Dataset (1 per day/step)
    let processor = BatchProcessor::new(config);
    let pp_values = processor.process(bbox);

    // WARNING: We get the fist day since we only generate 1 day of results for now
    let pp1 = pp_values.first().unwrap();

    println!("Baffin Bay PP values - Count: {}", pp_values.len());
    if !pp1.is_empty() {
        println!(
            "  Min: {:.2} mg C m−2 d−1",
            pp1.iter().fold(f32::INFINITY, |a, &b| a.min(b))
        );
        println!(
            "  Max: {:.2} mg C m−2 d−1",
            pp1.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        );
        println!(
            "  Mean: {:.2} mg C m−2 d−1",
            pp1.iter().sum::<f32>() / pp1.len() as f32
        );
        println!(
            "  First 10 values: {:?}",
            pp1.iter().take(10).collect::<Vec<&f32>>()
        );
    }

    Ok(())
}
