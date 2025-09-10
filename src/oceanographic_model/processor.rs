use super::pixel::PixelData;
use gdal::Dataset;
use std::{collections::HashMap, fmt::Display};

pub struct Bbox {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

impl Bbox {
    // TODO: should return an option, checking for bounds
    pub fn new(xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> Self {
        Bbox {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }
}

#[derive(Debug)]
pub struct OceanographicProcessor {
    datasets: HashMap<String, Dataset>,
    width: u32,
    height: u32,
}

impl OceanographicProcessor {
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
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_443.4km.cog.tif",
            ),
            (
                "rrs_490",
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_488.4km.cog.tif",
            ),
            (
                "rrs_555",
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.RRS.Rrs_555.4km.cog.tif",
            ),
            (
                "kd_490",
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.KD.Kd_490.4km.cog.tif",
            ),
            (
                "sst",
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.SST.sst.4km.cog.tif",
            ),
            (
                "chlor_a",
                "./data/geotiff/modia_aqua/AQUA_MODIS.20250701_20250731.L3m.MO.CHL.chlor_a.4km.cog.tif",
            ),
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

// TODO: Add tests comparing result of region_pp and bbox_pp, they should be the same if provided similar region in index or coord
