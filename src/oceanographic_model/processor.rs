use super::pixel::PixelData;
use gdal::Dataset;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub struct OceanographicProcessor {
    datasets: HashMap<String, Dataset>,
    width: u32,
    height: u32,
}

impl OceanographicProcessor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load all required rasters
        let raster_files = vec![
            ("rrs_443", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
            ("rrs_490", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
            ("rrs_555", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
            ("kd_490", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
            ("sst", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
            ("chlor_a", "./data/geotiff/AQUA_MODIS_chlor_a.tif"),
        ];

        let mut datasets = HashMap::new();
        let mut width = 0;
        let mut height = 0;

        for (name, path) in raster_files {
            match Dataset::open(path) {
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
        if let Some(dataset) = self.datasets.get("chlor_a") {
            // Get the first raster band from the chlorophyll-a dataset
            let band = dataset.rasterband(1)?;
            // Read a single pixel value at coordinates (x, y) as f32
            let buffer = band.read_as::<f32>((x as isize, y as isize), (1, 1), (1, 1), None)?;
            let value = buffer[(0, 0)];
            // println!("Raw chlor_a value: {}", value);
            // Handle missing data sentinel value (-32767.0) by converting to None
            pixel.chlor_a = if value == -32767.0 { None } else { Some(value) };
        }

        if let Some(dataset) = self.datasets.get("sst") {
            let band = dataset.rasterband(1)?;
            let buffer = band.read_as::<f32>((x as isize, y as isize), (1, 1), (1, 1), None)?;
            let value = buffer[(0, 0)];
            // println!("Raw sst value: {}", value);
            pixel.sst = if value == -32767.0 { None } else { Some(value) };
        }

        if let Some(dataset) = self.datasets.get("kd_490") {
            let band = dataset.rasterband(1)?;
            let buffer = band.read_as::<f32>((x as isize, y as isize), (1, 1), (1, 1), None)?;
            let value = buffer[(0, 0)];
            // println!("Raw kd_490 value: {}", value);
            pixel.kd_490 = if value == -32767.0 { None } else { Some(value) };
        }

        Ok(pixel.calculate_primary_production())
    }

    // Calculate PP for a small region
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
