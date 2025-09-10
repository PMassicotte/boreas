use super::pixel::PixelData;
use crate::utils;
use gdal::Dataset;
use std::{collections::HashMap, fmt::Display, path::Path};

pub struct Bbox {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

impl Bbox {
    pub fn new(xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> Result<Self, String> {
        if !(-180.0..=180.0).contains(&xmin) || !(-180.0..=180.0).contains(&xmax) {
            return Err("Longitude values must be between -180 and 180".to_string());
        }

        if !(-90.0..=90.0).contains(&ymin) || !(-90.0..=90.0).contains(&ymax) {
            return Err("Latitude values must be between -90 and 90".to_string());
        }

        if xmin > xmax || ymin > ymax {
            return Err("Min values must be <= max values".to_string());
        }

        Ok(Bbox {
            xmin,
            xmax,
            ymin,
            ymax,
        })
    }
}

#[derive(Debug)]
pub struct OceanographicProcessor {
    datasets: HashMap<String, Dataset>,
    width: u32,
    height: u32,
}

impl OceanographicProcessor {
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

    // TODO: Pass a Config for the file paths?
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let raster_files = vec![
            (
                "rrs_443",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_443.4km.cog.tif",
            ),
            (
                "rrs_490",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_488.4km.cog.tif",
            ),
            (
                "rrs_555",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_555.4km.cog.tif",
            ),
            (
                "kd_490",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.KD.Kd_490.4km.cog.tif",
            ),
            (
                "sst",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.SST.sst.4km.nc",
            ),
            (
                "chlor_a",
                "./data/geotiff/modis_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.CHL.chlor_a.4km.cog.tif",
            ),
        ];

        let mut datasets = HashMap::new();
        let mut width = 0;
        let mut height = 0;

        for (name, path) in raster_files {
            // Validate file type before processing
            let path_obj = Path::new(path);
            if !utils::is_supported_file_type(path_obj) {
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
            width,
            height,
        })
    }

    // Simple method to calculate primary production for a single pixel
    pub fn calculate_pixel_pp(
        &self,
        x: u32,
        y: u32,
    ) -> Result<Option<f32>, Box<dyn std::error::Error>> {
        let mut pixel = PixelData::new(x, y);

        // Read data from each dataset for this pixel.
        pixel.chlor_a = self.read_pixel_value("chlor_a", x, y)?;
        pixel.sst = self.read_pixel_value("sst", x, y)?;
        pixel.kd_490 = self.read_pixel_value("kd_490", x, y)?;

        Ok(pixel.calculate_primary_production())
    }

    pub fn calculate_region_pp(
        &self,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for y in y_start..(y_start + height).min(self.height) {
            for x in x_start..(x_start + width).min(self.width) {
                if let Some(pp) = self.calculate_pixel_pp(x, y)? {
                    results.push(pp);
                }
            }
        }

        Ok(results)
    }

    pub fn get_valid_pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn get_dim(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    // Calculate PP for a geographic bounding box
    pub fn calculate_pp_for_bbox(
        &self,
        bbox: Bbox,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Get a reference dataset to determine spatial properties, using first as template
        let sample_dataset = self.datasets.values().next().ok_or("No datasets loaded")?;

        // Get the geotransform to convert lon/lat to pixel coordinates
        let geotransform = sample_dataset.geo_transform()?;

        // Destruct values
        let Bbox {
            xmin: min_lon,
            xmax: max_lon,
            ymin: min_lat,
            ymax: max_lat,
        } = bbox;

        // Convert geographic coordinates to pixel coordinates
        // geotransform: [top_left_x, pixel_width, 0, top_left_y, 0, -pixel_height]
        let pixel_min_x = ((min_lon - geotransform[0]) / geotransform[1]).floor() as i32;
        let pixel_max_x = ((max_lon - geotransform[0]) / geotransform[1]).ceil() as i32;
        let pixel_min_y = ((max_lat - geotransform[3]) / geotransform[5]).floor() as i32;
        let pixel_max_y = ((min_lat - geotransform[3]) / geotransform[5]).ceil() as i32;

        // Ensure bounds are within dataset dimensions and handle negative values
        let start_x = pixel_min_x.max(0) as u32;
        let end_x = pixel_max_x.max(0).min(self.width as i32) as u32;
        let start_y = pixel_min_y.max(0) as u32;
        let end_y = pixel_max_y.max(0).min(self.height as i32) as u32;

        self.calculate_region_pp(start_x, start_y, end_x - start_x, end_y - start_y)
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

    #[test]
    fn test_bbox_coords_are_within_ranges() {
        // Test valid coordinates
        let valid_bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3);
        assert!(valid_bbox.is_ok());

        // Test longitude out of range
        let invalid_lon = Bbox::new(-200.0, 0.0, 0.0, 10.0);
        assert!(invalid_lon.is_err());

        let invalid_lon2 = Bbox::new(0.0, 200.0, 0.0, 10.0);
        assert!(invalid_lon2.is_err());

        // Test latitude out of range
        let invalid_lat = Bbox::new(0.0, 10.0, -100.0, 0.0);
        assert!(invalid_lat.is_err());

        let invalid_lat2 = Bbox::new(0.0, 10.0, 0.0, 100.0);
        assert!(invalid_lat2.is_err());

        // Test min > max
        let invalid_order_lon = Bbox::new(10.0, 0.0, 0.0, 10.0);
        assert!(invalid_order_lon.is_err());

        let invalid_order_lat = Bbox::new(0.0, 10.0, 10.0, 0.0);
        assert!(invalid_order_lat.is_err());
    }

    #[test]
    fn test_region_pp_vs_bbox_pp_equivalence() {
        let processor = match OceanographicProcessor::new() {
            Ok(p) => p,
            Err(_) => {
                // Skip test if datasets can't be loaded (e.g., in CI environments)
                return;
            }
        };

        // Use Baffin Bay coordinates (same as main.rs) which should have data
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();

        // Calculate PP using bbox method first
        let bbox_results = processor.calculate_pp_for_bbox(bbox).unwrap();

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

        // Results should be identical
        assert_eq!(region_results.len(), bbox_results.len());

        // Compare each value with small tolerance for floating point precision
        for (region_val, bbox_val) in region_results.iter().zip(bbox_results.iter()) {
            assert!(
                (region_val - bbox_val).abs() < 1e-6,
                "Values differ: region={}, bbox={}",
                region_val,
                bbox_val
            );
        }
    }

    #[test]
    fn test_bbox_coordinate_conversion() {
        let processor = match OceanographicProcessor::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        // Use a smaller area within Baffin Bay that should have data
        let bbox = Bbox::new(-67.0, -60.0, 71.0, 72.0).unwrap();

        let bbox_results = processor.calculate_pp_for_bbox(bbox).unwrap();

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
