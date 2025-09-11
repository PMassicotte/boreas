use crate::bbox::Bbox;
use std::collections::HashMap;

use crate::config::Config;
use crate::oceanographic_model::OceanographicProcessor;

#[derive(Debug)]
pub struct BatchProcessor {
    datasets: Vec<HashMap<String, String>>,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Self {
        let mut datasets = Vec::new();
        let mut rasters = HashMap::new();

        if let Some(raster_files) = config.raster_files() {
            for raster_file in raster_files {
                rasters.insert(raster_file.name.clone(), raster_file.path.clone());
            }
        }

        datasets.push(rasters);
        BatchProcessor { datasets }
    }

    pub fn process(&self, bbox: Bbox) -> Vec<Vec<f32>> {
        let mut all_pp = Vec::new();
        for raster in self.datasets.clone() {
            let proc = OceanographicProcessor::new(&raster).unwrap();
            all_pp.push(proc.calculate_pp_for_bbox(&bbox).unwrap());
        }

        all_pp
    }
}
