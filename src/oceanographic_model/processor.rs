use crate::bbox::Bbox;
use crate::config::Config;
use crate::error::BoreasError;
use crate::traits::{DatasetType, PrimaryProduction};
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
        // Use unique filename to avoid conflicts
        let mem_filename = format!(
            "/vsimem/pp_output_{}_{}.tif",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
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
            let dataset_type = DatasetType::from_name(name).ok_or_else(|| {
                BoreasError::Config(format!("Unknown dataset type: {}", name))
            })?;

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
