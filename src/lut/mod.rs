use std::fs::File;
use std::io::{BufRead, BufReader};

// [wavelength][theta][ozone][taucl][albedo]
type LutArray = Box<[[[[[f32; 7]; 8]; 10]; 19]; 83]>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Lut {
    xthetas: Vec<f32>,
    xozone: Vec<f32>,
    xtaucl: Vec<f32>,
    xalb: Vec<f32>,
    wavelengths: Vec<f32>,
    ed_lut: LutArray,
}

impl Lut {
    /// Creates the 5 vectors for LUT interpolation dimensions:
    /// 1. Wavelength = 290 : 700 : 5
    /// 2. ThetaS = 0 : 90 : 5
    /// 3. Ozone = 100 : 550 : 50
    /// 4. Cloud optical Thickness = 0 to 64 = c(0,1,2,4,8,16,32,64)
    /// 5. Surface Albedo = 0.05 : 0.9 : 0.15
    pub fn from_file(filename: &str) -> Result<Self, std::io::Error> {
        let xthetas: Vec<f32> = (0..19).map(|i| (i * 5) as f32).collect();
        let xozone: Vec<f32> = (0..10).map(|i| 100.0 + (i * 50) as f32).collect();
        let xtaucl: Vec<f32> = vec![0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
        let xalb: Vec<f32> = vec![0.05, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95];
        let wavelengths: Vec<f32> = (0..83).map(|i| 290.0 + (i * 5) as f32).collect();

        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut values: Vec<f32> = Vec::with_capacity(
            xthetas.len() * xozone.len() * xtaucl.len() * xalb.len() * wavelengths.len(),
        );

        // Read all values from file
        for line in reader.lines() {
            let line = line?;
            for value_str in line.split_whitespace() {
                if let Ok(value) = value_str.parse::<f32>() {
                    values.push(value);
                }
            }
        }

        // Fill the lookup table following C++ order: theta, ozone, taucl, albedo, wavelength
        let mut ed_lut = [[[[[0.0; 7]; 8]; 10]; 19]; 83];
        let mut idx = 0;

        #[allow(clippy::needless_range_loop)]
        for theta in 0..xthetas.len() {
            for ozone in 0..xozone.len() {
                for taucl in 0..xtaucl.len() {
                    for albedo in 0..xalb.len() {
                        for wavelength in 0..wavelengths.len() {
                            if idx < values.len() {
                                ed_lut[wavelength][theta][ozone][taucl][albedo] = values[idx];
                                idx += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(Lut {
            xthetas,
            xozone,
            xtaucl,
            xalb,
            wavelengths,
            ed_lut: Box::new(ed_lut),
        })
    }

    pub fn get_wavelength_values(
        &self,
        theta_idx: usize,
        ozone_idx: usize,
        taucl_idx: usize,
        albedo_idx: usize,
    ) -> Result<Vec<f32>, String> {
        if theta_idx >= self.xthetas.len() {
            return Err(format!(
                "theta_idx {} out of bounds (max: {})",
                theta_idx,
                self.xthetas.len() - 1
            ));
        }
        if ozone_idx >= self.xozone.len() {
            return Err(format!(
                "ozone_idx {} out of bounds (max: {})",
                ozone_idx,
                self.xozone.len() - 1
            ));
        }
        if taucl_idx >= self.xtaucl.len() {
            return Err(format!(
                "taucl_idx {} out of bounds (max: {})",
                taucl_idx,
                self.xtaucl.len() - 1
            ));
        }
        if albedo_idx >= self.xalb.len() {
            return Err(format!(
                "albedo_idx {} out of bounds (max: {})",
                albedo_idx,
                self.xalb.len() - 1
            ));
        }

        Ok((0..self.wavelengths.len())
            .map(|wavelength_idx| {
                self.ed_lut[wavelength_idx][theta_idx][ozone_idx][taucl_idx][albedo_idx]
            })
            .collect())
    }
}
