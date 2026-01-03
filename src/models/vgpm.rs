use crate::error::{BoreasError, Result};
use crate::traits::{DatasetType, PrimaryProduction};
use crate::utils::read_window_as_f32;
use gdal::Dataset;
use std::collections::HashMap;

/// Vertically Generalized Production Model (VGPM)
///
/// Standard VGPM algorithm for calculating ocean primary production
/// based on chlorophyll-a, SST, and light attenuation.
pub struct VgpmModel;

impl VgpmModel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VgpmModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimaryProduction for VgpmModel {
    fn name(&self) -> &str {
        "VGPM"
    }

    fn required_datasets(&self) -> Vec<DatasetType> {
        vec![
            DatasetType::Chlorophyll,
            DatasetType::SeaSurfaceTemperature,
            DatasetType::Kd490,
        ]
    }

    fn calculate(
        &self,
        datasets: &HashMap<DatasetType, Dataset>,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f64>> {
        // Get required datasets
        let chl_ds = datasets
            .get(&DatasetType::Chlorophyll)
            .ok_or_else(|| BoreasError::MissingDataset("Chlorophyll".to_string()))?;
        let sst_ds = datasets
            .get(&DatasetType::SeaSurfaceTemperature)
            .ok_or_else(|| BoreasError::MissingDataset("SeaSurfaceTemperature".to_string()))?;
        let kd_ds = datasets
            .get(&DatasetType::Kd490)
            .ok_or_else(|| BoreasError::MissingDataset("Kd490".to_string()))?;

        let num_pixels = (width * height) as usize;

        let chl_data = read_window_as_f32(
            chl_ds,
            x_start as isize,
            y_start as isize,
            width as usize,
            height as usize,
        )
        .map_err(|e| BoreasError::Calculation {
            model: "VGPM".to_string(),
            reason: e,
        })?;
        let sst_data = read_window_as_f32(
            sst_ds,
            x_start as isize,
            y_start as isize,
            width as usize,
            height as usize,
        )
        .map_err(|e| BoreasError::Calculation {
            model: "VGPM".to_string(),
            reason: e,
        })?;
        let kd_data = read_window_as_f32(
            kd_ds,
            x_start as isize,
            y_start as isize,
            width as usize,
            height as usize,
        )
        .map_err(|e| BoreasError::Calculation {
            model: "VGPM".to_string(),
            reason: e,
        })?;

        let mut results = Vec::with_capacity(num_pixels);

        for i in 0..num_pixels {
            let chl = chl_data[i] as f64;
            let sst = sst_data[i] as f64;
            let kd = kd_data[i] as f64;

            if chl <= 0.0 || kd <= 0.0 {
                results.push(f64::NAN);
                continue;
            }

            // Simplified VGPM calculation
            let exponent = 0.0275 * sst - 0.07 * sst.powi(2) + 0.0025 * sst.powi(3);
            let pbopt = 1.54 * 10_f64.powf(exponent);
            let zeu = 4.6 / kd; // Euphotic depth
            let pp = 0.66125 * pbopt * chl * zeu; // mg C m-2 d-1

            // Check for reasonable values (typical range: 10-2000 mg C m-2 d-1)
            if !pp.is_finite() || pp <= 0.0 || pp > 2000.0 {
                results.push(f64::NAN);
            } else {
                results.push(pp);
            }
        }

        Ok(results)
    }
}
