use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Satellites {
    SeaWiFS,
    Modis,
}

#[derive(Debug)]
pub struct SatBands {
    sensor: Satellites,
    wavelengths: &'static [u32],
}

impl SatBands {
    pub fn new(sensor: Satellites) -> Self {
        let wavelengths: &'static [u32] = match sensor {
            // Bands 1, 2, 3, 4, 5 and 6
            Satellites::SeaWiFS => &[412, 443, 490, 510, 555, 670],
            // Bands 8, 9, 10, 11, 12 and 13
            Satellites::Modis => &[412, 443, 488, 531, 547, 667],
        };
        Self {
            sensor,
            wavelengths,
        }
    }

    pub fn wavelengths(&self) -> &[u32] {
        self.wavelengths
    }

    pub fn closest_band(&self, target: u32) -> u32 {
        self.wavelengths
            .iter()
            .copied()
            .min_by_key(|w| (*w as i32 - target as i32).abs())
            .unwrap()
    }
}

impl Display for Satellites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Satellites::SeaWiFS => write!(f, "SeaWiFS"),
            Satellites::Modis => write!(f, "MODIS"),
        }
    }
}

impl Display for SatBands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Satellite: {}, Wavelengths: {:?}",
            self.sensor, self.wavelengths
        )
    }
}
