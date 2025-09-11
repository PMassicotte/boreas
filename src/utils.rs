use gdal::Dataset;

#[allow(dead_code)]
pub fn print_dataset_statistics(datasets: &[Dataset]) -> Result<(), Box<dyn std::error::Error>> {
    let total_pp_count = datasets
        .iter()
        .map(|dataset| {
            let (width, height) = dataset.raster_size();
            width * height
        })
        .sum::<usize>();

    println!(
        "PP values - Number of days: {}, Total PP values: {}",
        datasets.len(),
        total_pp_count
    );

    if let Some(first_dataset) = datasets.first() {
        let band = first_dataset.rasterband(1)?;
        let (width, height) = first_dataset.raster_size();
        let buffer = band.read_as::<f32>((0, 0), (width, height), (width, height), None)?;
        let all_values: Vec<f32> = buffer.data().to_vec();

        if !all_values.is_empty() {
            let valid_values: Vec<f32> = all_values
                .iter()
                .filter(|&&v| !v.is_nan())
                .cloned()
                .collect();

            println!(
                "  Min: {:.2} mg C m−2 d−1",
                valid_values.iter().fold(f32::INFINITY, |a, &b| a.min(b))
            );
            println!(
                "  Max: {:.2} mg C m−2 d−1",
                valid_values
                    .iter()
                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
            );
            println!(
                "  Mean: {:.2} mg C m−2 d−1",
                if valid_values.is_empty() {
                    f32::NAN
                } else {
                    valid_values.iter().sum::<f32>() / valid_values.len() as f32
                }
            );
            println!(
                "  Valid pixels: {} / {} ({:.1}%)",
                valid_values.len(),
                all_values.len(),
                100.0 * valid_values.len() as f32 / all_values.len() as f32
            );
            println!(
                "  First 10 non NaN values: {:?}",
                all_values
                    .iter()
                    .filter(|pp| !pp.is_nan())
                    .take(10)
                    .collect::<Vec<&f32>>()
            );
        }
    }

    Ok(())
}
