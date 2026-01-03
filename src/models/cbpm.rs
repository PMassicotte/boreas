use crate::error::{BoreasError, Result};
use crate::traits::{DatasetType, PrimaryProduction};
use crate::utils::read_window_as_f32;
use gdal::Dataset;
use std::collections::HashMap;

/// Carbon-based Production Model (CbPM)
///
/// Alternative primary production model based on carbon-specific growth rates
/// and phytoplankton biomass.
#[allow(dead_code)]
pub struct CbpmModel;

#[allow(dead_code)]
impl CbpmModel {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CbpmModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimaryProduction for CbpmModel {
    fn name(&self) -> &str {
        "CbPM"
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
            model: "CbPM".to_string(),
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
            model: "CbPM".to_string(),
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
            model: "CbPM".to_string(),
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

            // Simplified CbPM calculation
            // CbPM = Cphyto * μ * Zeu
            // where μ is the phytoplankton growth rate

            // Estimate carbon-to-chlorophyll ratio (simplified)
            let c_to_chl = 50.0; // typical range 30-80
            let cphyto = chl * c_to_chl; // mg C m-3

            // Temperature-dependent growth rate
            let mu_max = 2.0;
            let mu = mu_max * (1.066_f64).powf(sst - 20.0);

            // Euphotic depth
            let zeu = 4.6 / kd;

            // Primary production
            let pp = cphyto * mu * zeu; // mg C m-2 d-1

            // Check for reasonable values
            if !pp.is_finite() || pp <= 0.0 || pp > 2000.0 {
                results.push(f64::NAN);
            } else {
                results.push(pp);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::BoreasError;

    #[test]
    fn test_cbpm_model_new() {
        let model = CbpmModel::new();
        assert_eq!(model.name(), "CbPM");
    }

    #[test]
    fn test_cbpm_model_default() {
        let model = CbpmModel;
        assert_eq!(model.name(), "CbPM");
    }

    #[test]
    fn test_required_datasets() {
        let model = CbpmModel::new();
        let datasets = model.required_datasets();
        assert_eq!(datasets.len(), 3);
        assert!(datasets.contains(&DatasetType::Chlorophyll));
        assert!(datasets.contains(&DatasetType::SeaSurfaceTemperature));
        assert!(datasets.contains(&DatasetType::Kd490));
    }

    #[test]
    fn test_calculate_missing_chlorophyll() {
        let model = CbpmModel::new();
        let datasets = HashMap::new();

        let result = model.calculate(&datasets, 0, 0, 1, 1);

        assert!(result.is_err());
        match result {
            Err(BoreasError::MissingDataset(msg)) => {
                assert_eq!(msg, "Chlorophyll");
            }
            _ => panic!("Expected MissingDataset error for Chlorophyll"),
        }
    }

    #[test]
    fn test_calculate_missing_sst() {
        use gdal::DriverManager;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        // Create a minimal in-memory dataset for chlorophyll
        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver
            .create_with_band_type::<f32, _>("", 10, 10, 1)
            .unwrap();
        datasets.insert(DatasetType::Chlorophyll, chl_ds);

        let result = model.calculate(&datasets, 0, 0, 1, 1);

        assert!(result.is_err());
        match result {
            Err(BoreasError::MissingDataset(msg)) => {
                assert_eq!(msg, "SeaSurfaceTemperature");
            }
            _ => panic!("Expected MissingDataset error for SeaSurfaceTemperature"),
        }
    }

    #[test]
    fn test_calculate_missing_kd490() {
        use gdal::DriverManager;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        // Create minimal in-memory datasets
        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver
            .create_with_band_type::<f32, _>("", 10, 10, 1)
            .unwrap();
        let sst_ds = driver
            .create_with_band_type::<f32, _>("", 10, 10, 1)
            .unwrap();
        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);

        let result = model.calculate(&datasets, 0, 0, 1, 1);

        assert!(result.is_err());
        match result {
            Err(BoreasError::MissingDataset(msg)) => {
                assert_eq!(msg, "Kd490");
            }
            _ => panic!("Expected MissingDataset error for Kd490"),
        }
    }

    #[test]
    fn test_calculate_with_negative_chlorophyll() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();

        // Write negative chlorophyll values
        let mut chl_data = Buffer::new((2, 2), vec![-1.0f32, -0.5, 0.0, 0.5]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut chl_data)
            .unwrap();

        // Write valid SST and Kd values
        let mut sst_data = Buffer::new((2, 2), vec![20.0f32; 4]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut sst_data)
            .unwrap();
        let mut kd_data = Buffer::new((2, 2), vec![0.2f32; 4]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 2, 2).unwrap();

        assert_eq!(result.len(), 4);
        // First three values should be NaN due to negative/zero chlorophyll
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());
        assert!(result[2].is_nan());
        // Last value should be valid
        assert!(result[3].is_finite());
        assert!(result[3] > 0.0);
    }

    #[test]
    fn test_calculate_with_zero_kd() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 2, 2, 1).unwrap();

        // Write valid chlorophyll values
        let mut chl_data = Buffer::new((2, 2), vec![0.5f32; 4]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut chl_data)
            .unwrap();

        // Write valid SST values
        let mut sst_data = Buffer::new((2, 2), vec![20.0f32; 4]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut sst_data)
            .unwrap();

        // Write zero and negative Kd values
        let mut kd_data = Buffer::new((2, 2), vec![0.0f32, -0.1, 0.0, 0.2]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (2, 2), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 2, 2).unwrap();

        assert_eq!(result.len(), 4);
        // First three values should be NaN due to non-positive Kd
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());
        assert!(result[2].is_nan());
        // Last value should be valid
        assert!(result[3].is_finite());
        assert!(result[3] > 0.0);
    }

    #[test]
    fn test_calculate_with_valid_inputs() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 3, 3, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 3, 3, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 3, 3, 1).unwrap();

        // Write valid values - using more reasonable values to avoid exceeding 2000 threshold
        let mut chl_data =
            Buffer::new((3, 3), vec![0.3f32, 0.5, 0.8, 0.1, 0.2, 0.6, 0.4, 0.5, 0.3]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 3), &mut chl_data)
            .unwrap();

        let mut sst_data = Buffer::new(
            (3, 3),
            vec![15.0f32, 18.0, 20.0, 10.0, 22.0, 19.0, 17.0, 21.0, 16.0],
        );
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 3), &mut sst_data)
            .unwrap();

        let mut kd_data = Buffer::new(
            (3, 3),
            vec![0.1f32, 0.15, 0.2, 0.12, 0.18, 0.15, 0.13, 0.16, 0.14],
        );
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 3), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 3, 3).unwrap();

        assert_eq!(result.len(), 9);

        // All values should be finite and positive, or NaN if they exceed the threshold
        for (i, &value) in result.iter().enumerate() {
            if value.is_finite() {
                assert!(value > 0.0, "Pixel {}: finite value should be positive", i);
                assert!(
                    value <= 2000.0,
                    "Pixel {}: value should be within reasonable range",
                    i
                );
            }
            // Some values might be NaN if calculation produces values > 2000, which is acceptable
        }

        // At least some values should be valid
        let valid_count = result.iter().filter(|v| v.is_finite()).count();
        assert!(valid_count > 0, "At least some values should be finite");
    }

    #[test]
    fn test_calculate_boundary_sst() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();

        // Write valid chlorophyll
        let mut chl_data = Buffer::new((3, 1), vec![1.0f32; 3]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut chl_data)
            .unwrap();

        // Write extreme SST values (cold, normal, hot)
        let mut sst_data = Buffer::new((3, 1), vec![-2.0f32, 20.0, 35.0]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut sst_data)
            .unwrap();

        let mut kd_data = Buffer::new((3, 1), vec![0.1f32; 3]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 3, 1).unwrap();

        assert_eq!(result.len(), 3);

        // All results should be finite (the algorithm should handle extreme temps)
        // Some might be NaN if calculation produces unreasonable values
        for (i, &value) in result.iter().enumerate() {
            if value.is_finite() {
                assert!(value > 0.0, "Pixel {}: finite value should be positive", i);
                assert!(value <= 2000.0, "Pixel {}: value should be within range", i);
            }
        }
    }

    #[test]
    fn test_calculate_extreme_kd_values() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 3, 1, 1).unwrap();

        let mut chl_data = Buffer::new((3, 1), vec![1.0f32; 3]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut chl_data)
            .unwrap();

        let mut sst_data = Buffer::new((3, 1), vec![20.0f32; 3]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut sst_data)
            .unwrap();

        // Very small, normal, and large Kd values
        let mut kd_data = Buffer::new((3, 1), vec![0.001f32, 0.1, 1.0]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (3, 1), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 3, 1).unwrap();

        assert_eq!(result.len(), 3);

        // Very small Kd should produce large euphotic depth and potentially large PP
        // but should still be validated
        for (i, &value) in result.iter().enumerate() {
            if value.is_finite() {
                assert!(value > 0.0, "Pixel {}: finite value should be positive", i);
                // The range check in the algorithm should catch values > 2000
            }
        }
    }

    #[test]
    fn test_calculate_nan_input_handling() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 4, 1, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 4, 1, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 4, 1, 1).unwrap();

        // Include NaN values in inputs
        let mut chl_data = Buffer::new((4, 1), vec![f32::NAN, 0.5, 0.5, 0.5]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (4, 1), &mut chl_data)
            .unwrap();

        let mut sst_data = Buffer::new((4, 1), vec![20.0f32, f32::NAN, 20.0, 20.0]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (4, 1), &mut sst_data)
            .unwrap();

        let mut kd_data = Buffer::new((4, 1), vec![0.2f32, 0.2, f32::NAN, 0.2]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (4, 1), &mut kd_data)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 4, 1).unwrap();

        assert_eq!(result.len(), 4);

        // NaN inputs should not produce finite results
        // First pixel has NaN chlorophyll, but will be caught by chl <= 0.0 check (NaN comparisons return false)
        // Actually, NaN <= 0.0 is false, so it won't be caught by that check
        // But the calculation will produce NaN, which will be caught by is_finite check
        assert!(
            result[0].is_nan(),
            "Pixel 0: NaN chlorophyll should produce NaN result"
        );
        assert!(
            result[1].is_nan(),
            "Pixel 1: NaN SST should produce NaN result"
        );
        assert!(
            result[2].is_nan(),
            "Pixel 2: NaN Kd should produce NaN result"
        );
        // Last pixel should be valid
        assert!(
            result[3].is_finite() && result[3] > 0.0,
            "Pixel 3: valid inputs should produce finite result"
        );
    }

    #[test]
    fn test_calculation_formula_spot_check() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();

        // Use specific values to manually verify calculation
        let chl = 1.0f32;
        let sst = 20.0f32;
        let kd = 0.1f32;

        let mut chl_buf = Buffer::new((1, 1), vec![chl]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut chl_buf)
            .unwrap();
        let mut sst_buf = Buffer::new((1, 1), vec![sst]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut sst_buf)
            .unwrap();
        let mut kd_buf = Buffer::new((1, 1), vec![kd]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut kd_buf)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 1, 1).unwrap();

        // Manual calculation:
        // c_to_chl = 50.0
        // cphyto = 1.0 * 50.0 = 50.0 mg C m-3
        // mu_max = 2.0
        // mu = 2.0 * (1.066^(20-20)) = 2.0 * 1.0 = 2.0
        // zeu = 4.6 / 0.1 = 46.0 m
        // pp = 50.0 * 2.0 * 46.0 = 4600.0 mg C m-2 d-1
        // This exceeds 2000, so should be NaN

        assert_eq!(result.len(), 1);
        assert!(
            result[0].is_nan(),
            "Result should be NaN due to exceeding 2000 threshold"
        );
    }

    #[test]
    fn test_calculation_within_reasonable_range() {
        use gdal::DriverManager;
        use gdal::raster::Buffer;

        let model = CbpmModel::new();
        let mut datasets = HashMap::new();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let chl_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();
        let sst_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();
        let kd_ds = driver.create_with_band_type::<f32, _>("", 1, 1, 1).unwrap();

        // Use values that should produce a reasonable result
        let chl = 0.5f32;
        let sst = 15.0f32;
        let kd = 0.2f32;

        let mut chl_buf = Buffer::new((1, 1), vec![chl]);
        chl_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut chl_buf)
            .unwrap();
        let mut sst_buf = Buffer::new((1, 1), vec![sst]);
        sst_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut sst_buf)
            .unwrap();
        let mut kd_buf = Buffer::new((1, 1), vec![kd]);
        kd_ds
            .rasterband(1)
            .unwrap()
            .write((0, 0), (1, 1), &mut kd_buf)
            .unwrap();

        datasets.insert(DatasetType::Chlorophyll, chl_ds);
        datasets.insert(DatasetType::SeaSurfaceTemperature, sst_ds);
        datasets.insert(DatasetType::Kd490, kd_ds);

        let result = model.calculate(&datasets, 0, 0, 1, 1).unwrap();

        // Manual calculation for reference:
        // c_to_chl = 50.0
        // cphyto = 0.5 * 50.0 = 25.0 mg C m-3
        // mu_max = 2.0
        // mu = 2.0 * (1.066^(15-20)) = 2.0 * (1.066^-5) = 2.0 * 0.719 = 1.438
        // zeu = 4.6 / 0.2 = 23.0 m
        // pp = 25.0 * 1.438 * 23.0 = 827.35 mg C m-2 d-1
        // This should be within the 0-2000 range

        assert_eq!(result.len(), 1);
        assert!(result[0].is_finite(), "Result should be finite");
        assert!(result[0] > 0.0, "Result should be positive");
        assert!(
            result[0] <= 2000.0,
            "Result should be within reasonable range"
        );
    }
}
