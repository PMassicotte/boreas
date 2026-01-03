use gdal::Dataset;
use std::collections::HashMap;

pub trait PrimaryProduction {
    /// Calculate primary production for a specified region
    ///
    /// # Arguments
    /// * `datasets` - HashMap of datasets by name (e.g., "chlor_a", "sst", "kd_490")
    /// * `x_start` - Starting x coordinate (column)
    /// * `y_start` - Starting y coordinate (row)
    /// * `width` - Width of the region to process
    /// * `height` - Height of the region to process
    fn calculate(
        &self,
        datasets: &HashMap<String, Dataset>,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f64>, String>;

    fn name(&self) -> &str;
}
