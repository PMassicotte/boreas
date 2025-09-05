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
        let mut ed_lut = Box::new([[[[[0.0; 7]; 8]; 10]; 19]; 83]);
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
            ed_lut,
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

        let mut result = Vec::with_capacity(self.wavelengths.len());
        for wavelength_idx in 0..self.wavelengths.len() {
            result.push(self.ed_lut[wavelength_idx][theta_idx][ozone_idx][taucl_idx][albedo_idx]);
        }

        Ok(result)
    }

    #[allow(dead_code)]
    fn get_lut_value(
        &self,
        wavelength: usize,
        theta: usize,
        ozone: usize,
        taucl: usize,
        albedo: usize,
    ) -> f32 {
        self.ed_lut[wavelength][theta][ozone][taucl][albedo]
    }

    fn get_indice(&self, vec: &[f32], mut target: f32) -> (usize, f32) {
        // Apply Fortran-style boundary clamping first
        if vec == self.xthetas && target >= 90.0 {
            target = 89.99;
        } else if vec == self.xozone && target >= 550.0 {
            target = 549.99;
        } else if vec == self.xtaucl && target >= 64.0 {
            target = 63.99;
        } else if vec == self.xalb {
            if target <= 0.05 {
                target = 0.051;
            } else if target >= 0.95 {
                target = 0.9499;
            }
        }

        // Fortran-style index finding
        if target < vec[0] {
            return (0, 0.0); // Special case: r = 0 when below range
        }

        // Find bracketing indices using manual search (like Fortran)
        let mut idx = 0;
        for i in 0..(vec.len() - 1) {
            if target >= vec[i] && target < vec[i + 1] {
                idx = i;
                break;
            }
        }

        let rr = (target - vec[idx]) / (vec[idx + 1] - vec[idx]);
        (idx, rr)
    }

    fn interpol_ed0moins(&self, thetas: f32, ozone: f32, taucl: f32, alb: f32) -> Vec<f32> {
        let nwl = self.wavelengths.len();

        let (ithetas, rthetas) = self.get_indice(&self.xthetas, thetas);
        let (iozone, rozone) = self.get_indice(&self.xozone, ozone);
        let (itaucl, rtaucl) = self.get_indice(&self.xtaucl, taucl);
        let (ialb, ralb) = self.get_indice(&self.xalb, alb);

        let ed_tmp4 = &mut [[[[0.0f32; 2]; 2]; 2]; 83];
        let ed_tmp3 = &mut [[[0.0f32; 2]; 2]; 83];
        let ed_tmp2 = &mut [[0.0f32; 2]; 83];
        let mut ed = Vec::with_capacity(nwl);
        ed.resize(nwl, 0.0);

        // Remove the dimension on Surface Albedo
        for i in 0..=1 {
            let zthetas = (ithetas + i).min(self.xthetas.len() - 1);

            for j in 0..=1 {
                let zozone = (iozone + j).min(self.xozone.len() - 1);

                for k in 0..=1 {
                    let ztaucl = (itaucl + k).min(self.xtaucl.len() - 1);

                    let albedo_high = (ialb + 1).min(self.xalb.len() - 1);
                    let blend_factor = 1.0 - ralb;

                    for l in 0..nwl {
                        unsafe {
                            let val1 = *self
                                .ed_lut
                                .get_unchecked(l)
                                .get_unchecked(zthetas)
                                .get_unchecked(zozone)
                                .get_unchecked(ztaucl)
                                .get_unchecked(ialb);
                            let val2 = *self
                                .ed_lut
                                .get_unchecked(l)
                                .get_unchecked(zthetas)
                                .get_unchecked(zozone)
                                .get_unchecked(ztaucl)
                                .get_unchecked(albedo_high);
                            *ed_tmp4
                                .get_unchecked_mut(l)
                                .get_unchecked_mut(i)
                                .get_unchecked_mut(j)
                                .get_unchecked_mut(k) = blend_factor * val1 + ralb * val2;
                        }
                    }
                }
            }
        }

        // Remove the dimension on taucl
        for i in 0..=1 {
            for j in 0..=1 {
                for l in 0..nwl {
                    ed_tmp3[l][i][j] =
                        (1.0 - rtaucl) * ed_tmp4[l][i][j][0] + rtaucl * ed_tmp4[l][i][j][1];
                }
            }
        }

        // Remove the dimension on ozone
        for i in 0..=1 {
            for l in 0..nwl {
                ed_tmp2[l][i] = (1.0 - rozone) * ed_tmp3[l][i][0] + rozone * ed_tmp3[l][i][1];
            }
        }

        // Remove the dimension on sunzenith angle
        for l in 0..nwl {
            unsafe {
                let mut val = (1.0 - rthetas) * ed_tmp2.get_unchecked(l).get_unchecked(0)
                    + rthetas * ed_tmp2.get_unchecked(l).get_unchecked(1);

                // Fortran-style overflow protection
                if val > 10000.0 {
                    val = 0.0;
                }

                *ed.get_unchecked_mut(l) = val;
            }
        }

        ed
    }

    /// Computes the downward irradiance (Ed0-) for given atmospheric conditions.
    ///
    /// # Parameters
    /// - `thetas`: Solar zenith angle in degrees (0-90)
    /// - `o3`: Ozone column in DU (100-550)
    /// - `tcl`: Cloud optical thickness (0-64)
    /// - `cf`: Cloud fraction (0-1)
    /// - `alb`: Surface albedo (0.05-0.95)
    ///
    /// # Returns
    /// Vector of Ed0- values for all wavelengths (290-700nm in 5nm steps)
    ///
    /// # Example
    /// ```
    /// use crate::lut::lookup_table::Lut;
    ///
    /// let lut = Lut::from_file("ed0moins.lut").unwrap();
    ///
    /// // Clear sky conditions at noon
    /// let ed_clear = lut.ed0moins(30.0, 300.0, 0.0, 0.0, 0.1);
    ///
    /// // Partly cloudy conditions
    /// let ed_cloudy = lut.ed0moins(45.0, 350.0, 16.0, 0.5, 0.2);
    ///
    /// // Print Ed0- at 400nm (wavelength index 22)
    /// println!("Ed0- at 400nm: {:.4}", ed_cloudy[22]);
    /// ```
    pub fn ed0moins(&self, thetas: f32, o3: f32, tcl: f32, cf: f32, alb: f32) -> Vec<f32> {
        let ed_cloud = self.interpol_ed0moins(thetas, o3, tcl, alb);
        let ed_clear = self.interpol_ed0moins(thetas, o3, 0.0, alb);

        let mut ed_inst = Vec::with_capacity(ed_cloud.len());

        if thetas < 90.0 {
            for i in 0..ed_cloud.len() {
                ed_inst.push(ed_cloud[i] * cf + ed_clear[i] * (1.0 - cf));
            }
        } else {
            ed_inst.resize(ed_cloud.len(), 0.0);
        }

        ed_inst
    }
}
