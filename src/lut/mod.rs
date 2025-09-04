use std::fs::File;
use std::io::{BufRead, BufReader};
#[derive(Debug)]
pub struct Lut {
    xthetas: Vec<f32>,
    xozone: Vec<f32>,
    xtaucl: Vec<f32>,
    xalb: Vec<f32>,
    wavelengths: Vec<f32>,
    ed_lut: [[[[[f32; 7]; 8]; 10]; 19]; 83], // [wavelength][theta][ozone][taucl][albedo]
}

impl Lut {
    /// Creates the 5 vectors for LUT interpolation dimensions:
    /// 1. Wavelength = 290 : 700 : 5
    /// 2. ThetaS = 0 : 90 : 5
    /// 3. Ozone = 100 : 550 : 50
    /// 4. Cloud optical Thickness = 0 to 64 = c(0,1,2,4,8,16,32,64)
    /// 5. Surface Albedo = 0.05 : 0.9 : 0.15
    pub fn new() -> Self {
        // Thetas: 0 to 90, by 5
        let xthetas: Vec<f32> = (0..=90).map(|i| (i * 5) as f32).collect();

        // Ozone: 100 to 550, 50 steps (10 values)
        let xozone: Vec<f32> = (0..10).map(|i| 100.0 + (i * 50) as f32).collect();

        // Cloud optical depth (8 values)
        let xtaucl: Vec<f32> = vec![0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];

        // Surface albedo: 0.05 to 0.95, 0.15 steps (7 values)
        let xalb: Vec<f32> = vec![0.05, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95];

        // Wavelengths: 290 to 700 nm, 5 nm steps (83 values)
        let wavelengths: Vec<f32> = (0..83).map(|i| 290.0 + (i * 5) as f32).collect();

        Lut {
            xthetas,
            xozone,
            xtaucl,
            xalb,
            wavelengths,
            ed_lut: [[[[[0.0; 7]; 8]; 10]; 19]; 83],
        }
    }

    pub fn load_from_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut values: Vec<f32> = Vec::with_capacity(
            self.xthetas.len()
                * self.xozone.len()
                * self.xtaucl.len()
                * self.xalb.len()
                * self.wavelengths.len(),
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
        let mut idx = 0;
        for theta in 0..self.xthetas.len() {
            for ozone in 0..self.xozone.len() {
                for taucl in 0..self.xtaucl.len() {
                    for albedo in 0..self.xalb.len() {
                        for wavelength in 0..self.wavelengths.len() {
                            if idx < values.len() {
                                self.ed_lut[wavelength][theta][ozone][taucl][albedo] = values[idx];
                                idx += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_wavelength_values(
        &self,
        theta_idx: usize,
        ozone_idx: usize,
        taucl_idx: usize,
        albedo_idx: usize,
    ) -> Vec<f32> {
        (0..self.wavelengths.len())
            .map(|wavelength_idx| {
                self.ed_lut[wavelength_idx][theta_idx][ozone_idx][taucl_idx][albedo_idx]
            })
            .collect()
    }
}
