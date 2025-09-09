use gdal::Dataset;
use std::collections::HashMap;

// Oceanographic data for a single pixel
#[derive(Debug, Clone)]
pub struct PixelData {
    pub x: u32,
    pub y: u32,
    pub rrs_443: Option<f32>, // Remote sensing reflectance at 443nm
    pub rrs_490: Option<f32>, // Remote sensing reflectance at 490nm
    pub rrs_555: Option<f32>, // Remote sensing reflectance at 555nm
    pub kd_490: Option<f32>,  // Diffuse attenuation coefficient
    pub sst: Option<f32>,     // Sea surface temperature
    pub chlor_a: Option<f32>, // Chlorophyll-a concentration
}

impl PixelData {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            rrs_443: None,
            rrs_490: None,
            rrs_555: None,
            kd_490: None,
            sst: None,
            chlor_a: None,
        }
    }

    // Primary production calculation using Vertically Generalized Production Model (VGPM)
    pub fn calculate_primary_production(&self) -> Option<f32> {
        let chl = self.chlor_a?;
        let sst = self.sst?;
        let kd = self.kd_490?;

        if chl <= 0.0 || kd <= 0.0 {
            return None;
        }

        // Simplified VGPM calculation
        let pbopt =
            1.54 * 10_f32.powf(0.0275 * sst - 0.07 * sst.powf(2.0) + 0.0025 * sst.powf(3.0));
        let zeu = 4.6 / kd; // Euphotic depth
        let pp = 0.66125 * pbopt * chl * zeu; // mg C m-2 d-1

        Some(pp)
    }
}

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

        // Read data from each dataset for this pixel
        if let Some(dataset) = self.datasets.get("chlor_a") {
            let band = dataset.rasterband(1)?;
            let buffer = band.read_as::<f32>((x as isize, y as isize), (1, 1), (1, 1), None)?;
            let value = buffer[(0, 0)];
            // println!("Raw chlor_a value: {}", value);
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

        // println!(
        //     "Pixel data: chlor_a={:?}, sst={:?}, kd_490={:?}",
        //     pixel.chlor_a, pixel.sst, pixel.kd_490
        // );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_production_calculation() {
        let mut pixel = PixelData::new(0, 0);
        pixel.chlor_a = Some(1.0);
        pixel.sst = Some(15.0);
        pixel.kd_490 = Some(0.1);

        let pp = pixel.calculate_primary_production();
        assert!(pp.is_some());
        assert!(pp.unwrap() > 0.0);
    }
}
