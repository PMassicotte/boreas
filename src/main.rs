mod bbox;
mod config;
mod date_gen;
mod iop;
mod lut;
mod oceanographic_model;
mod sat_bands;

use config::Config;
use oceanographic_model::batch_process::BatchProcessor;
// use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting oceanographic primary production processing...");

    let config = Config::from_file("./data/config/simple_config.json").unwrap();

    // TODO: I think process should return a gdal Dataset (1 per day/step)
    let processor = BatchProcessor::new(config);
    let pp_values = processor.process()?;

    // WARNING: We get the first day since we only generate 1 day of results for now
    let pp1 = pp_values.first().ok_or("No processing results available")?;

    // Read data from the datasets to get statistics
    let mut total_pp_count = 0;

    for (i, dataset) in pp_values.iter().enumerate() {
        let band = dataset.rasterband(1)?;
        let (width, height) = dataset.raster_size();
        let buffer = band.read_as::<f32>((0, 0), (width, height), (width, height), None)?;
        let values: Vec<f32> = buffer.data().to_vec();
        total_pp_count += values.len();

        // Dataset is already properly georeferenced and ready to use
        // To save it, you would typically use GDAL command line tools or a different method
        // For now, the dataset remains in memory with all spatial information intact
        // Example of how to save dataset to file:
        let driver = gdal::DriverManager::get_driver_by_name("GTiff")?;
        let filename = format!("/home/filoche/Desktop/test_{}.tif", i);
        let options = gdal::cpl::CslStringList::new();
        let _copy = dataset.create_copy(&driver, &filename, &options)?;
    }

    // Read data from the first dataset for detailed statistics
    let band = pp1.rasterband(1)?;
    let (width, height) = pp1.raster_size();
    let buffer = band.read_as::<f32>((0, 0), (width, height), (width, height), None)?;
    let all_values: Vec<f32> = buffer.data().to_vec();

    println!(
        "Baffin Bay PP values - Number of days: {}, Total PP values: {}",
        pp_values.len(),
        total_pp_count
    );

    if !all_values.is_empty() {
        println!(
            "  Min: {:.2} mg C m−2 d−1",
            all_values.iter().fold(f32::INFINITY, |a, &b| a.min(b))
        );
        println!(
            "  Max: {:.2} mg C m−2 d−1",
            all_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        );
        println!(
            "  Mean: {:.2} mg C m−2 d−1",
            all_values.iter().sum::<f32>() / all_values.len() as f32
        );
        println!(
            "  First 10 values: {:?}",
            all_values.iter().take(10).collect::<Vec<&f32>>()
        );
    }

    Ok(())
}
