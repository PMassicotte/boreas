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