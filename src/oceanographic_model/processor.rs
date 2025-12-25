use super::pixel::PixelData;
use crate::bbox::Bbox;
use crate::config::{Config, RasterFile};
use gdal::{Dataset, Metadata};
use std::{collections::HashMap, fmt::Display, path::Path};

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
        let mem_filename = "/vsimem/pp_output.tif";
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
    // HashMap containing all the input datasets loaded by GDAL
    datasets: HashMap<String, Dataset>,
    // Reference to config for name-to-layer_name mapping
    raster_templates: Vec<RasterFile>,
    width: u32,
    height: u32,
}

impl OceanographicProcessor {
    pub fn new(
        raster_files: &HashMap<String, String>,
        config: &Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut datasets = HashMap::new();
        let mut width = 0;
        let mut height = 0;

        for (name, path) in raster_files {
            // Validate file type before processing
            let path_obj = Path::new(&path);
            if !super::is_supported_file_type(path_obj) {
                return Err(format!("Unsupported file type for {}: {}", name, path).into());
            }

            // Automatically detect file format and create appropriate GDAL path
            let gdal_path = Self::detect_file_format_and_path(path, name);

            match Dataset::open(&gdal_path) {
                Ok(dataset) => {
                    let (w, h) = dataset.raster_size();
                    if width == 0 {
                        width = w as u32;
                        height = h as u32;
                    }
                    // Verify all rasters have same dimensions
                    if w as u32 != width || h as u32 != height {
                        eprintln!("Warning: {} has different dimensions", name);
                    }
                    datasets.insert(name.to_string(), dataset);
                }
                Err(e) => eprintln!("Could not load {}: {}", name, e),
            }
        }

        Ok(Self {
            datasets,
            raster_templates: config.raster_templates().clone(),
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

    fn read_pixel_value(
        &self,
        dataset_name: &str,
        x: u32,
        y: u32,
    ) -> Result<Option<f32>, Box<dyn std::error::Error>> {
        // dataset_name is the
        if let Some(dataset) = self.datasets.get(dataset_name) {
            let band = dataset.rasterband(1)?;
            let buffer = band.read_as::<f32>((x as isize, y as isize), (1, 1), (1, 1), None)?;
            let raw_value = buffer[(0, 0)];
            let scale = band.scale().unwrap_or(1.0);
            let missing_value = band.no_data_value();

            if missing_value.is_some_and(|mv| raw_value == mv as f32) {
                Ok(None)
            } else {
                Ok(Some(raw_value * scale as f32))
            }
        } else {
            Ok(None)
        }
    }

    // Simple method to calculate primary production for a single pixel
    pub fn calculate_pixel_pp(
        &self,
        x: u32,
        y: u32,
    ) -> Result<Option<f32>, Box<dyn std::error::Error>> {
        let mut pixel = PixelData::new(x, y);

        // Read data from each dataset for this pixel using config mapping
        pixel.chlor_a = self.read_pixel_value(&self.get_layer_name("chlor_a"), x, y)?;
        pixel.sst = self.read_pixel_value(&self.get_layer_name("sst"), x, y)?;
        pixel.kd_490 = self.read_pixel_value(&self.get_layer_name("kd_490"), x, y)?;

        Ok(pixel.calculate_primary_production())
    }

    // Get the layer name for a given field name from config. For example, if the layer name in
    // the nc file is Kd_490, it will retirn kd_490 (lower case). Useful when the layer name is
    // not exactly matching the name of the expected name. I could use that to read rrs488 and
    // assign that to rrs490
    fn get_layer_name(&self, name: &str) -> String {
        self.raster_templates
            .iter()
            .find(|template| template.name == name)
            .map(|template| template.layer_name.clone())
            .unwrap_or_else(|| name.to_string())
    }

    pub fn calculate_region_pp(
        &self,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut results = Vec::with_capacity((width * height) as usize);

        for y in y_start..(y_start + height).min(self.height) {
            for x in x_start..(x_start + width).min(self.width) {
                let pp_value = match self.calculate_pixel_pp(x, y)? {
                    Some(pp) => pp,
                    None => f32::NAN, // Use NaN for missing/no-data pixels
                };
                results.push(pp_value);
            }
        }

        Ok(results)
    }

    #[allow(dead_code)]
    pub fn get_valid_pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[allow(dead_code)]
    pub fn get_dim(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    // Calculate PP for a geographic bounding box
    pub fn calculate_pp_for_bbox(
        &self,
        bbox: &Bbox,
    ) -> Result<Dataset, Box<dyn std::error::Error>> {
        let sample_dataset = self.datasets.values().next().ok_or("No datasets loaded")?;
        let geotransform = sample_dataset.geo_transform()?;

        let spatial_region = SpatialRegion::new(bbox, &geotransform, self.width, self.height)?;

        // Based on bbox, we calculated the starting pixel position and the width, height of the
        // window where to calculate pp
        let pp_values = self.calculate_region_pp(
            spatial_region.start_x,
            spatial_region.start_y,
            spatial_region.output_width,
            spatial_region.output_height,
        )?;

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
    use std::fs;
    use tempfile::tempdir;

    fn create_mock_config() -> Config {
        // Create a temporary directory for the output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();

        // Create the directory to avoid validation errors
        fs::create_dir_all(output_path).unwrap();

        let config_json = format!(
            r#"{{
            "model_id": "test_model",
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
                    "name": "rrs_443",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.RRS.Rrs_443.4km.cog.tif",
                    "date_format": "YYYYMMDD",
                    "layer_name": "Rrs_443"
                }},
                {{
                    "name": "rrs_490",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.RRS.Rrs_488.4km.cog.tif",
                    "date_format": "YYYYMMDD",
                    "layer_name": "Rrs_488"
                }},
                {{
                    "name": "rrs_555",
                    "base_directory": "./data/geotiff/modis_aqua/",
                    "filename_pattern": "AQUA_MODIS.{{}}*.L3m.MO.RRS.Rrs_555.4km.cog.tif",
                    "date_format": "YYYYMMDD",
                    "layer_name": "Rrs_555"
                }},
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
            output_path
        );

        serde_json::from_str(&config_json).unwrap()
    }

    fn create_mock_data() -> HashMap<String, String> {
        let mut mock_data = HashMap::new();
        mock_data.insert(
            "rrs_443".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_443.4km.cog.tif"
                .to_string(),
        );
        mock_data.insert(
            "rrs_490".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_488.4km.cog.tif"
                .to_string(),
        );
        mock_data.insert(
            "rrs_555".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_555.4km.cog.tif"
                .to_string(),
        );
        mock_data.insert(
            "kd_490".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.KD.Kd_490.4km.cog.tif"
                .to_string(),
        );
        mock_data.insert(
            "sst".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.SST.sst.4km.nc"
                .to_string(),
        );
        mock_data.insert(
            "chlor_a".to_string(),
            "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.CHL.chlor_a.4km.cog.tif"
                .to_string(),
        );
        mock_data
    }

    #[test]
    fn test_region_pp_vs_bbox_pp_equivalence() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => {
                // Skip test if datasets can't be loaded (e.g., in CI environments)
                return;
            }
        };

        // Use Baffin Bay coordinates (same as main.rs) which should have data
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();

        // Calculate PP using bbox method first - now returns Dataset
        let bbox_dataset = processor.calculate_pp_for_bbox(&bbox).unwrap();

        // Get dataset reference to calculate geotransform for region method
        let sample_dataset = processor.datasets.values().next().unwrap();
        let geotransform = sample_dataset.geo_transform().unwrap();

        // Convert bbox coordinates to pixel coordinates for region method
        let pixel_min_x = ((-67.2 - geotransform[0]) / geotransform[1]).floor() as i32;
        let pixel_max_x = ((-58.7 - geotransform[0]) / geotransform[1]).ceil() as i32;
        let pixel_min_y = ((73.3 - geotransform[3]) / geotransform[5]).floor() as i32;
        let pixel_max_y = ((70.9 - geotransform[3]) / geotransform[5]).ceil() as i32;

        // Ensure bounds are within dataset dimensions
        let start_x = pixel_min_x.max(0) as u32;
        let end_x = pixel_max_x.max(0).min(processor.width as i32) as u32;
        let start_y = pixel_min_y.max(0) as u32;
        let end_y = pixel_max_y.max(0).min(processor.height as i32) as u32;

        // Calculate PP using region method
        let region_results = processor
            .calculate_region_pp(start_x, start_y, end_x - start_x, end_y - start_y)
            .unwrap();

        // Read data from bbox dataset for comparison
        let bbox_band = bbox_dataset.rasterband(1).unwrap();
        let (width, height) = bbox_dataset.raster_size();
        let bbox_data = bbox_band
            .read_as::<f32>((0, 0), (width, height), (width, height), None)
            .unwrap();
        let bbox_results: Vec<f32> = bbox_data.data().to_vec();

        // Results should be identical
        assert_eq!(region_results.len(), bbox_results.len());

        // Compare each value with small tolerance for floating point precision
        for (region_val, bbox_val) in region_results.iter().zip(bbox_results.iter()) {
            // Handle NaN values - both should be NaN or both should be finite and equal
            if region_val.is_nan() && bbox_val.is_nan() {
                continue; // Both NaN, this is expected for missing data
            } else if region_val.is_finite() && bbox_val.is_finite() {
                assert!(
                    (region_val - bbox_val).abs() < 1e-6,
                    "Values differ: region={}, bbox={}",
                    region_val,
                    bbox_val
                );
            } else {
                panic!(
                    "Inconsistent NaN handling: region={}, bbox={}",
                    region_val, bbox_val
                );
            }
        }
    }

    #[test]
    fn test_bbox_coordinate_conversion() {
        let rasters = create_mock_data();
        let config = create_mock_config();
        let processor = match OceanographicProcessor::new(&rasters, &config) {
            Ok(p) => p,
            Err(_) => return,
        };

        // Use a smaller area within Baffin Bay that should have data
        let bbox = Bbox::new(-67.0, -60.0, 71.0, 72.0).unwrap();

        let bbox_dataset = processor.calculate_pp_for_bbox(&bbox).unwrap();

        // Get dataset reference to calculate corresponding pixel coordinates
        let sample_dataset = processor.datasets.values().next().unwrap();
        let geotransform = sample_dataset.geo_transform().unwrap();

        // Convert bbox coordinates to pixel coordinates for region method
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
            .calculate_region_pp(start_x, start_y, end_x - start_x, end_y - start_y)
            .unwrap();

        // Read data from bbox dataset
        let bbox_band = bbox_dataset.rasterband(1).unwrap();
        let (width, height) = bbox_dataset.raster_size();
        let bbox_data = bbox_band
            .read_as::<f32>((0, 0), (width, height), (width, height), None)
            .unwrap();
        let bbox_results: Vec<f32> = bbox_data.data().to_vec();

        // Should produce similar number of results
        let diff = (bbox_results.len() as i32 - region_results.len() as i32).abs();
        assert!(
            bbox_results.len() == region_results.len(),
            "The number of produced PP values are not the same: bbox_results.len() = {}, region_results.len() = {}, diff = {}",
            bbox_results.len(),
            region_results.len(),
            diff
        )
    }
}
