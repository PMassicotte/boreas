use crate::bbox::Bbox;
use std::collections::HashMap;

use crate::oceanographic_model::OceanographicProcessor;

#[derive(Debug)]
pub struct BatchProcessor {
    datasets: Vec<HashMap<String, String>>,
}

impl BatchProcessor {
    pub fn new() -> Self {
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

        let mut datasets = Vec::new();
        let mut rasters = HashMap::new();

        for (name, path) in raster_files {
            rasters.insert(name.to_string(), path.to_string());
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

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}
