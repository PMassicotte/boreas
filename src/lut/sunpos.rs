/// Solar position calculation module
///
/// Rust implementation of the FORTRAN sunpos subroutine
/// Calculates solar zenith angle and azimuth angle for given time and location
/// Result struct containing solar position calculations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolarPosition {
    pub zenith_angle_deg: f32,
    pub azimuth_angle_deg: f32,
    pub altitude_angle_deg: f32,
    pub declination_deg: f32,
    pub local_solar_noon: f32,
    pub hour_angle_deg: f32,
    pub atmospheric_mass: f32,
}

impl SolarPosition {
    /// Calculate solar position using the original FORTRAN algorithm
    ///
    /// # Arguments
    /// * `jday` - Julian day of year (1-365/366)
    /// * `hour` - Hour in decimal format (0.0-24.0, UTC time)
    /// * `latitude` - Latitude in decimal degrees (-90 to +90)
    /// * `longitude` - Longitude in decimal degrees (-180 to +180)
    ///
    /// # Returns
    /// * `SolarPosition` struct with zenith angle and azimuth angle in degrees
    pub fn calculate(jday: i16, hour: f32, latitude: f32, longitude: f32) -> Self {
        // Constants
        let pi = std::f32::consts::PI;
        let d2r = pi / 180.0;
        let r2d = 180.0 / pi;

        // Local time meridian (set to 0 for GMT/UTC time, as per original)
        let ltm = 0;

        // Extract hour and minute components
        let hr = hour as i16;
        let min = ((hour - hr as f32) * 60.0) as i16;

        // Calculate local solar noon
        let lsn = 12.0 + ((ltm as f32 - longitude) / 15.0);

        // Convert latitude to radians
        let latrad = latitude * d2r;

        // Calculate solar declination (angle of sun relative to equatorial plane)
        let decrad = 23.45 * d2r * (d2r * 360.0 * (284.0 + jday as f32) / 365.0).sin();
        let decdeg = decrad * r2d;

        // Convert hour to floating point
        let ha = hr as f32 + (min as f32 / 60.0);

        // Calculate hour angle in minutes, then convert to radians
        let hangle = (lsn - ha) * 60.0;
        let harad = hangle * 0.0043633; // This equals hangle * (15.0 * d2r) / 60.0

        // Calculate solar altitude angle
        let saltrad =
            ((latrad.sin() * decrad.sin()) + (latrad.cos() * decrad.cos() * harad.cos())).asin();

        let saltdeg = saltrad * r2d;

        // Calculate solar azimuth angle
        let sazirad = (decrad.cos() * harad.sin() / saltrad.cos()).asin();
        let sazideg = sazirad * r2d;

        // Calculate zenith angle and atmospheric mass
        let (szendeg, _szenrad, mass) = if saltdeg < 0.0 || saltrad > 180.0 {
            // Sun is below horizon
            (90.0, 90.0 * d2r, 1229_f32.sqrt())
        } else {
            // Sun is above horizon
            let szendeg = 90.0 - saltdeg;
            let szenrad = szendeg * d2r;
            let mass = (1229.0 + (614.0 * saltrad.sin()).powi(2)).sqrt() - (614.0 * saltrad.sin());
            (szendeg, szenrad, mass)
        };

        // Calculate base transmittance (not used in return but calculated in original)
        let _tbbase = (-0.65 * mass).exp() + (-0.09 * mass).exp();

        SolarPosition {
            zenith_angle_deg: szendeg,
            azimuth_angle_deg: sazideg,
            altitude_angle_deg: saltdeg,
            declination_deg: decdeg,
            local_solar_noon: lsn,
            hour_angle_deg: hangle / 60.0 * 15.0, // Convert back to degrees
            atmospheric_mass: mass,
        }
    }

    /// Convenience method that returns only zenith and azimuth angles
    /// matching the original FORTRAN subroutine signature
    #[allow(dead_code)]
    pub fn zenith_azimuth(&self) -> (f32, f32) {
        (self.zenith_angle_deg, self.azimuth_angle_deg)
    }

    /// Convenience function that returns only zenith and azimuth angles
    /// matching the original FORTRAN subroutine signature
    #[allow(dead_code)]
    pub fn simple(jday: i16, hour: f32, latitude: f32, longitude: f32) -> (f32, f32) {
        let pos = Self::calculate(jday, hour, latitude, longitude);
        pos.zenith_azimuth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sunpos_main_example() {
        // Test case matching the main.rs example
        let pos = SolarPosition::calculate(100, 12.0, 45.0, -75.0);

        // These values should match the Fortran output exactly
        assert!(
            (pos.zenith_angle_deg - 74.09).abs() < 0.01,
            "Expected zenith ~74.09°, got {:.2}°",
            pos.zenith_angle_deg
        );
        assert!(
            (pos.azimuth_angle_deg - 84.71).abs() < 0.01,
            "Expected azimuth ~84.71°, got {:.2}°",
            pos.azimuth_angle_deg
        );
        assert!(
            (pos.altitude_angle_deg - 15.91).abs() < 0.01,
            "Expected altitude ~15.91°, got {:.2}°",
            pos.altitude_angle_deg
        );
    }

    #[test]
    fn test_sunpos_noon() {
        // Test at solar noon, summer solstice, 45N latitude
        let pos = SolarPosition::calculate(172, 12.0, 45.0, 0.0); // Day 172 ≈ June 21

        // At solar noon, zenith angle should be minimal
        // At 45N on summer solstice, sun zenith ≈ 21.55 degrees
        assert!((pos.zenith_angle_deg - 21.55).abs() < 1.0);

        // Azimuth should be close to 0 (due south) at solar noon
        assert!(pos.azimuth_angle_deg.abs() < 5.0);

        // Altitude + zenith should equal 90 degrees
        assert!((pos.altitude_angle_deg + pos.zenith_angle_deg - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_sunpos_winter() {
        // Test at winter solstice
        let pos = SolarPosition::calculate(355, 12.0, 45.0, 0.0); // Day 355 ≈ December 21

        // At 45N on winter solstice, sun zenith should be much higher
        assert!(pos.zenith_angle_deg > 60.0);

        // Declination should be negative (southern hemisphere favored)
        assert!(pos.declination_deg < 0.0);
    }

    #[test]
    fn test_sunpos_equator_equinox() {
        // Test at equator during equinox
        let pos = SolarPosition::calculate(80, 12.0, 0.0, 0.0); // Day 80 ≈ March 21 (vernal equinox)

        // At equator on equinox at solar noon, sun should be nearly overhead
        assert!(
            pos.zenith_angle_deg < 5.0,
            "Zenith angle should be very small at equator/equinox"
        );

        // Declination should be close to 0 at equinox
        assert!(
            pos.declination_deg.abs() < 5.0,
            "Declination should be near 0° at equinox"
        );
    }

    #[test]
    fn test_sunpos_arctic_summer() {
        // Test at high latitude during summer
        let pos = SolarPosition::calculate(172, 12.0, 70.0, 0.0); // 70°N, summer solstice

        // At high latitude in summer, sun should be visible (altitude > 0)
        assert!(
            pos.altitude_angle_deg > 0.0,
            "Sun should be above horizon in arctic summer"
        );

        // Zenith angle should be less than 90° (sun visible)
        assert!(pos.zenith_angle_deg < 90.0);
    }

    #[test]
    fn test_sunpos_different_longitudes() {
        // Test same time at different longitudes
        let pos_west = SolarPosition::calculate(100, 12.0, 45.0, -120.0); // West coast US
        let pos_east = SolarPosition::calculate(100, 12.0, 45.0, -75.0); // East coast US

        // Solar angles should be different due to longitude difference
        assert!(
            (pos_west.zenith_angle_deg - pos_east.zenith_angle_deg).abs() > 5.0,
            "Different longitudes should give different solar angles"
        );
    }

    #[test]
    fn test_sunpos_below_horizon() {
        // Test during night time
        let pos = SolarPosition::calculate(172, 0.0, 45.0, 0.0); // Midnight

        // Should return valid range for all angles
        assert!(pos.zenith_angle_deg >= 0.0 && pos.zenith_angle_deg <= 180.0);
        assert!(pos.azimuth_angle_deg >= -180.0 && pos.azimuth_angle_deg <= 180.0);
    }

    #[test]
    fn test_sunpos_extreme_latitudes() {
        // Test at extreme latitudes
        let pos_north = SolarPosition::calculate(172, 12.0, 89.0, 0.0); // Near North Pole
        let pos_south = SolarPosition::calculate(172, 12.0, -89.0, 0.0); // Near South Pole

        // Results should be valid
        assert!(pos_north.zenith_angle_deg >= 0.0 && pos_north.zenith_angle_deg <= 180.0);
        assert!(pos_south.zenith_angle_deg >= 0.0 && pos_south.zenith_angle_deg <= 180.0);

        // In summer, North Pole should have lower zenith angle than South Pole
        assert!(pos_north.zenith_angle_deg < pos_south.zenith_angle_deg);
    }

    #[test]
    fn test_sunpos_atmospheric_mass() {
        // Test atmospheric mass calculation
        let pos_overhead = SolarPosition::calculate(172, 12.0, 23.45, 0.0); // Sun nearly overhead
        let pos_horizon = SolarPosition::calculate(172, 6.0, 45.0, 0.0); // Sun near horizon

        // Atmospheric mass should be higher when sun is lower (higher zenith angle)
        assert!(
            pos_horizon.atmospheric_mass > pos_overhead.atmospheric_mass,
            "Atmospheric mass should be higher near horizon"
        );

        // Atmospheric mass should be reasonable (> 1 for any sun position)
        assert!(pos_overhead.atmospheric_mass >= 1.0);
        assert!(pos_horizon.atmospheric_mass >= 1.0);
    }

    #[test]
    fn test_sunpos_simple_wrapper() {
        // Test the convenience function
        let (zenith, azimuth) = SolarPosition::simple(100, 12.0, 45.0, -75.0);
        let full_pos = SolarPosition::calculate(100, 12.0, 45.0, -75.0);

        // Should return same values as full function
        assert!((zenith - full_pos.zenith_angle_deg).abs() < 0.001);
        assert!((azimuth - full_pos.azimuth_angle_deg).abs() < 0.001);
    }

    #[test]
    fn test_sunpos_declination_range() {
        // Test declination throughout the year
        let mut min_dec = f32::MAX;
        let mut max_dec = f32::MIN;

        for day in (1..365).step_by(30) {
            let pos = SolarPosition::calculate(day, 12.0, 0.0, 0.0);
            min_dec = min_dec.min(pos.declination_deg);
            max_dec = max_dec.max(pos.declination_deg);
        }

        // Declination should range approximately ±23.45° throughout year
        assert!(
            min_dec < -20.0 && min_dec > -25.0,
            "Min declination should be ~-23.45°"
        );
        assert!(
            max_dec > 20.0 && max_dec < 25.0,
            "Max declination should be ~+23.45°"
        );
    }
}
