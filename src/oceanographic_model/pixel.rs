use std::fmt::Display;

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
        let chl = self.chlor_a?; // mg/m3
        let sst = self.sst?; // °C (auto-scaled by processor)
        let kd = self.kd_490?; // m−1 (auto-scaled by processor)

        if chl <= 0.0 || kd <= 0.0 || !(-5.0..=50.0).contains(&sst) {
            return None;
        }

        // Simplified VGPM calculation
        let exponent = 0.0275 * sst - 0.07 * sst.powf(2.0) + 0.0025 * sst.powf(3.0);
        let pbopt = 1.54 * 10_f32.powf(exponent);
        let zeu = 4.6 / kd; // Euphotic depth
        let pp = 0.66125 * pbopt * chl * zeu; // mg C m-2 d-1

        // Check for reasonable values (typical range: 10-2000 mg C m-2 d-1)
        if !pp.is_finite() || pp <= 0.0 || pp > 2000.0 {
            return None;
        }

        Some(pp)
    }
}

impl Display for PixelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pixel ({}, {})", self.x, self.y)?;
        writeln!(f, "  RRS 443nm: {:?}", self.rrs_443)?;
        writeln!(f, "  RRS 490nm: {:?}", self.rrs_490)?;
        writeln!(f, "  RRS 555nm: {:?}", self.rrs_555)?;
        writeln!(f, "  Kd 490nm: {:?}", self.kd_490)?;
        writeln!(f, "  SST: {:?}", self.sst)?;
        writeln!(f, "  Chlor-a: {:?}", self.chlor_a)?;
        if let Some(pp) = self.calculate_primary_production() {
            writeln!(f, "  Primary Production: {:.2} mg C m-2 d-1", pp)?;
        }
        Ok(())
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
