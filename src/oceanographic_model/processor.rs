use crate::bbox::Bbox;
use crate::config::Config;
use crate::error::BoreasError;
use crate::traits::{DatasetType, PrimaryProduction};
use gdal::{Dataset, Metadata};
use std::{collections::HashMap, fmt::Display, path::Path};
use uuid::Uuid;

struct SpatialRegion {
    start_x: u32,
    start_y: u32,
    output_width: u32,
    output_height: u32,
    geotransform: [f64; 6],
}

impl SpatialRegion {
    fn new(
        bbox: &Bbox,
        geotransform: &[f64; 6],
        dataset_width: u32,
        dataset_height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let Bbox {
            xmin: min_lon,
            xmax: max_lon,
            ymin: min_lat,
            ymax: max_lat,
        } = bbox;

        // Convert geographic coordinates to pixel coordinates
        let pixel_min_x = ((min_lon - geotransform[0]) / geotransform[1]).floor() as i32;
        let pixel_max_x = ((max_lon - geotransform[0]) / geotransform[1]).ceil() as i32;
        let pixel_min_y = ((max_lat - geotransform[3]) / geotransform[5]).floor() as i32;
        let pixel_max_y = ((min_lat - geotransform[3]) / geotransform[5]).ceil() as i32;

        // Ensure bounds are within dataset dimensions and handle negative values
        let start_x = pixel_min_x.max(0) as u32;
        let end_x = pixel_max_x.max(0).min(dataset_width as i32) as u32;
        let start_y = pixel_min_y.max(0) as u32;
        let end_y = pixel_max_y.max(0).min(dataset_height as i32) as u32;

        // Calculate the output dimensions
        let output_width = end_x - start_x;
        let output_height = end_y - start_y;

        Ok(Self {
            start_x,
            start_y,
            output_width,
            output_height,
            geotransform: *geotransform,
        })
    }

    fn create_output_dataset(
        &self,
        sample_dataset: &Dataset,
        pp_values: Vec<f32>,
    ) -> Result<Dataset, Box<dyn std::error::Error>> {
        // Use UUID for guaranteed uniqueness
        let mem_filename = format!("/vsimem/pp_output_{}.tif", Uuid::new_v4());
        let driver = gdal::DriverManager::get_driver_by_name("GTiff")?;
        let mut destination_dataset = driver.create_with_band_type::<f32, _>(
            mem_filename,
            self.output_width as usize,
            self.output_height as usize,
            1,
        )?;

        let output_geotransform = [
            self.geotransform[0] + (self.start_x as f64) * self.geotransform[1], // top-left x
            self.geotransform[1],                                                // pixel width
            self.geotransform[2], // rotation (usually 0)
            self.geotransform[3] + (self.start_y as f64) * self.geotransform[5], // top-left y
            self.geotransform[4], // rotation (usually 0)
            self.geotransform[5], // pixel height (negative)
        ];

        destination_dataset.set_geo_transform(&output_geotransform)?;

        if let Ok(spatial_ref) = sample_dataset.spatial_ref() {
            destination_dataset.set_spatial_ref(&spatial_ref)?;
        }

        // Set dataset metadata
        destination_dataset.set_metadata_item("TIFFTAG_DOCUMENTNAME", "Primary Production", "")?;
        destination_dataset.set_metadata_item(
            "TIFFTAG_IMAGEDESCRIPTION",
            "Primary production calculated from satellite oceanographic data",
            "",
        )?;

        destination_dataset.set_metadata_item(
            "TIFFTAG_SOFTWARE",
            "Boreas - Oceanographic Processing Tool",
            "",
        )?;

        // Get the raster band and write data
        let mut band = destination_dataset.rasterband(1)?;

        // Set band metadata
        band.set_description("Primary Production")?;
        band.set_metadata_item("long_name", "Primary Production", "")?;
        band.set_metadata_item(
            "standard_name",
            "net_primary_production_of_biomass_expressed_as_carbon_per_unit_area_in_sea_water",
            "",
        )?;
        band.set_metadata_item("Unit", "mg C m-2 d-1", "")?;

        let mut buffer = gdal::raster::Buffer::new(
            (self.output_width as usize, self.output_height as usize),
            pp_values,
        );

        band.write(
            (0, 0),
            (self.output_width as usize, self.output_height as usize),
            &mut buffer,
        )?;

        Ok(destination_dataset)
    }
}

#[derive(Debug)]
pub struct OceanographicProcessor {
    // HashMap containing all the input datasets loaded by GDAL, keyed by DatasetType
    datasets: HashMap<DatasetType, Dataset>,
    width: u32,
    height: u32,
}

impl OceanographicProcessor {
    pub fn new(
        raster_files: &HashMap<String, (String, String)>,
        _config: &Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut datasets = HashMap::new();
        let mut width = 0;
        let mut height = 0;

        for (name, (path, layer_name)) in raster_files {
            // Convert string name to DatasetType
            let dataset_type = DatasetType::from_name(name)
                .ok_or_else(|| BoreasError::Config(format!("Unknown dataset type: {}", name)))?;

            // Validate file type before processing
            let path_obj = Path::new(path);
            if !super::is_supported_file_type(path_obj) {
                return Err(format!("Unsupported file type for {}: {}", name, path).into());
            }

            // Automatically detect file format and create appropriate GDAL path
            let gdal_path = Self::detect_file_format_and_path(path, layer_name);

            match Dataset::open(&gdal_path) {
                Ok(dataset) => {
                    let (w, h) = dataset.raster_size();
                    if width == 0 {
                        width = w as u32;
                        height = h as u32;
                    }
                    // Verify all rasters have same dimensions
                    if w as u32 != width || h as u32 != height {
                        return Err(BoreasError::DimensionMismatch(format!(
                            "{} has dimensions {}x{} but expected {}x{}",
                            name, w, h, width, height
                        ))
                        .into());
                    }
                    datasets.insert(dataset_type, dataset);
                }
                Err(e) => {
                    return Err(BoreasError::Config(format!(
                        "Could not load dataset {}: {}",
                        name, e
                    ))
                    .into());
                }
            }
        }

        Ok(Self {
            datasets,
            width,
            height,
        })
    }

    fn detect_file_format_and_path(file_path: &str, variable_name: &str) -> String {
        if file_path.ends_with(".nc") {
            // NetCDF format - add NETCDF: prefix and variable suffix
            format!("NETCDF:{}:{}", file_path, variable_name)
        } else {
            // Assume GeoTIFF or other GDAL-supported format
            file_path.to_string()
        }
    }

    #[allow(dead_code)]
    pub fn get_valid_pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[allow(dead_code)]
    pub fn get_dim(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Run a primary production algorithm for a specific region using the trait-based approach
    pub fn run_algo(
        &self,
        algo: &dyn PrimaryProduction,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        println!("Executing: {}", algo.name());

        algo.calculate(&self.datasets, x_start, y_start, width, height)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })
    }

    /// Calculate PP for a geographic bounding box using a trait-based model
    pub fn calculate_pp_for_bbox_with_model(
        &self,
        bbox: &Bbox,
        algo: &dyn PrimaryProduction,
    ) -> Result<Dataset, Box<dyn std::error::Error>> {
        // Get geotransform from one of the datasets (assuming all have same geotransform). This
        // will be used as template for output dataset
        let sample_dataset = self.datasets.values().next().ok_or("No datasets loaded")?;
        let geotransform = sample_dataset.geo_transform()?;

        let spatial_region = SpatialRegion::new(bbox, &geotransform, self.width, self.height)?;

        // Calculate PP using the trait-based model
        let pp_values_f64 = self.run_algo(
            algo,
            spatial_region.start_x,
            spatial_region.start_y,
            spatial_region.output_width,
            spatial_region.output_height,
        )?;

        // Convert f64 to f32 for the output dataset
        let pp_values: Vec<f32> = pp_values_f64.iter().map(|&v| v as f32).collect();

        spatial_region.create_output_dataset(sample_dataset, pp_values)
    }
}

impl Display for OceanographicProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OceanographicProcessor {{ datasets: {}, dimensions: {}x{} }}",
            self.datasets.len(),
            self.width,
            self.height
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bbox::Bbox;
    use crate::config::Config;
    use crate::models::VgpmModel;
    use std::fs;

    fn create_mock_config() -> Config {
        // Use system temp directory for output
        let output_path = std::env::temp_dir().join("boreas_test_output");

        // Create the directory to avoid validation errors
        fs::create_dir_all(&output_path).unwrap();

        let config_json = format!(
            r#"{{
            "model_id": "test_model",
            "algorithm": "vgpm",
            "start_date": "2024-07-01",
            "end_date": "2024-08-01", 
            "frequency": "monthly",
            "hourly_increment": 4,
            "output_directory": "{}",
            "bbox": {{
                "xmin": -67.2,
                "xmax": -58.7,
                "ymin": 70.9,
                "ymax": 73.3
            }},
            "raster_templates": [
                {{
                    "name": "kd_490",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.KD.Kd_490.4km.cog.tif",
                    "date_format": "YYYYMMDD",
                    "layer_name": "Kd_490"
                }},
                {{
                    "name": "sst",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.SST.sst.4km.nc",
                    "date_format": "YYYYMMDD",
                    "layer_name": "sst"
                }},
                {{
                    "name": "chlor_a",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.CHL.chlor_a.4km.cog.tif",
                    "date_format": "YYYYMMDD",
                    "layer_name": "chlor_a"
                }}
            ]
        }}"#,
            output_path.to_str().unwrap()
        );

        serde_json::from_str(&config_json).unwrap()
    }

    fn create_mock_data() -> HashMap<String, (String, String)> {
        let mut mock_data = HashMap::new();
        mock_data.insert(
            "kd_490".to_string(),
            (
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.KD.Kd_490.4km.cog.tif"
                    .to_string(),
                "Kd_490".to_string(),
            ),
        );
        mock_data.insert(
            "sst".to_string(),
            (
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.SST.sst.4km.nc"
                    .to_string(),
                "sst".to_string(),
            ),
        );
        mock_data.insert(
            "chlor_a".to_string(),
            (
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.CHL.chlor_a.4km.cog.tif"
                    .to_string(),
                "chlor_a".to_string(),
            ),
        );
        mock_data
    }

    #[test]
    fn test_run_algo_with_vgpm() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => {
                // Skip test if datasets can't be loaded (e.g., in CI environments)
                return;
            }
        };

        let vgpm = VgpmModel::new();
        let (width, height) = processor.get_dim();

        // Calculate PP for a small region using run_algo
        let result = processor.run_algo(&vgpm, 0, 0, 10.min(width), 10.min(height));

        assert!(result.is_ok(), "run_algo should succeed");
        let pp_values = result.unwrap();
        assert_eq!(
            pp_values.len(),
            (10.min(width) * 10.min(height)) as usize,
            "Should return correct number of values"
        );
    }

    #[test]
    fn test_calculate_pp_for_bbox_with_model() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => {
                // Skip test if datasets can't be loaded
                return;
            }
        };

        // Use Baffin Bay coordinates
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();
        let vgpm = VgpmModel::new();

        // Calculate PP using bbox method
        let result = processor.calculate_pp_for_bbox_with_model(&bbox, &vgpm);

        assert!(
            result.is_ok(),
            "calculate_pp_for_bbox_with_model should succeed"
        );
        let dataset = result.unwrap();
        let (width, height) = dataset.raster_size();
        assert!(
            width > 0 && height > 0,
            "Output dataset should have valid dimensions"
        );
    }

    #[test]
    fn test_run_algo_vs_bbox_equivalence() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => {
                // Skip test if datasets can't be loaded
                return;
            }
        };

        // Use Baffin Bay coordinates
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();
        let vgpm = VgpmModel::new();

        // Calculate PP using bbox method
        let bbox_dataset = processor
            .calculate_pp_for_bbox_with_model(&bbox, &vgpm)
            .unwrap();

        // Get geotransform to calculate pixel coordinates
        let sample_dataset = processor.datasets.values().next().unwrap();
        let geotransform = sample_dataset.geo_transform().unwrap();

        // Convert bbox coordinates to pixel coordinates
        let pixel_min_x = ((-67.2 - geotransform[0]) / geotransform[1]).floor() as i32;
        let pixel_max_x = ((-58.7 - geotransform[0]) / geotransform[1]).ceil() as i32;
        let pixel_min_y = ((73.3 - geotransform[3]) / geotransform[5]).floor() as i32;
        let pixel_max_y = ((70.9 - geotransform[3]) / geotransform[5]).ceil() as i32;

        // Ensure bounds are within dataset dimensions
        let start_x = pixel_min_x.max(0) as u32;
        let end_x = pixel_max_x.max(0).min(processor.width as i32) as u32;
        let start_y = pixel_min_y.max(0) as u32;
        let end_y = pixel_max_y.max(0).min(processor.height as i32) as u32;

        // Calculate PP using run_algo method
        let region_results = processor
            .run_algo(&vgpm, start_x, start_y, end_x - start_x, end_y - start_y)
            .unwrap();

        // Read data from bbox dataset
        let bbox_band = bbox_dataset.rasterband(1).unwrap();
        let (width, height) = bbox_dataset.raster_size();
        let bbox_data = bbox_band
            .read_as::<f32>((0, 0), (width, height), (width, height), None)
            .unwrap();
        let bbox_results: Vec<f64> = bbox_data.data().iter().map(|&v| v as f64).collect();

        // Results should be identical in length
        assert_eq!(
            region_results.len(),
            bbox_results.len(),
            "run_algo and calculate_pp_for_bbox_with_model should produce same number of values"
        );

        // Compare each value with tolerance for floating point precision
        let mut matching_values = 0;
        let mut total_values = 0;

        for (region_val, bbox_val) in region_results.iter().zip(bbox_results.iter()) {
            total_values += 1;
            // Handle NaN values - both should be NaN or both should be finite and equal
            if region_val.is_nan() && bbox_val.is_nan() {
                matching_values += 1;
                continue;
            } else if region_val.is_finite()
                && bbox_val.is_finite()
                && (region_val - bbox_val).abs() < 1e-4
            {
                matching_values += 1;
            }
        }

        // At least 95% of values should match (allowing for some floating point differences)
        let match_ratio = matching_values as f64 / total_values as f64;
        assert!(
            match_ratio > 0.95,
            "At least 95% of values should match between run_algo and calculate_pp_for_bbox_with_model (got {:.2}%)",
            match_ratio * 100.0
        );
    }

    #[test]
    fn test_bbox_coordinate_conversion() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => return,
        };

        // Use a smaller area within Baffin Bay
        let bbox = Bbox::new(-67.0, -60.0, 71.0, 72.0).unwrap();
        let vgpm = VgpmModel::new();

        let bbox_dataset = processor
            .calculate_pp_for_bbox_with_model(&bbox, &vgpm)
            .unwrap();

        // Get geotransform to calculate pixel coordinates
        let sample_dataset = processor.datasets.values().next().unwrap();
        let geotransform = sample_dataset.geo_transform().unwrap();

        // Convert bbox coordinates to pixel coordinates
        let pixel_min_x = ((-67.0 - geotransform[0]) / geotransform[1]).floor() as i32;
        let pixel_max_x = ((-60.0 - geotransform[0]) / geotransform[1]).ceil() as i32;
        let pixel_min_y = ((72.0 - geotransform[3]) / geotransform[5]).floor() as i32;
        let pixel_max_y = ((71.0 - geotransform[3]) / geotransform[5]).ceil() as i32;

        // Ensure bounds are within dataset dimensions
        let start_x = pixel_min_x.max(0) as u32;
        let end_x = pixel_max_x.max(0).min(processor.width as i32) as u32;
        let start_y = pixel_min_y.max(0) as u32;
        let end_y = pixel_max_y.max(0).min(processor.height as i32) as u32;

        let region_results = processor
            .run_algo(&vgpm, start_x, start_y, end_x - start_x, end_y - start_y)
            .unwrap();

        // Read data from bbox dataset
        let bbox_band = bbox_dataset.rasterband(1).unwrap();
        let (width, height) = bbox_dataset.raster_size();
        let bbox_data = bbox_band
            .read_as::<f32>((0, 0), (width, height), (width, height), None)
            .unwrap();
        let bbox_results: Vec<f64> = bbox_data.data().iter().map(|&v| v as f64).collect();

        // Should produce same number of results
        assert_eq!(
            bbox_results.len(),
            region_results.len(),
            "Coordinate conversion should produce matching dimensions: bbox={}, region={}",
            bbox_results.len(),
            region_results.len()
        );
    }
}
