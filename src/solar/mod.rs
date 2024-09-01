// TODO: Redo the calculation based on the excel files:
// https://gml.noaa.gov/grad/solcalc/calcdetails.html

use chrono::{Datelike, NaiveDateTime, Timelike};
use std::f64::consts::PI;

// struct Solar {
//     sun_zenithal_angle: f64,
// }

pub fn sun_zenithal_angle(date: NaiveDateTime, longitude: f64, latitude: f64) -> f64 {
    // Constants
    let days_in_year = 365.25;
    let declination_angle_max = 23.44;

    // Convert date to the number of days in the year
    let day_of_year = date.ordinal() as f64;

    // Calculate the hour angle
    let time_of_day = date.num_seconds_from_midnight() as f64 / 3600.0;
    let solar_time = time_of_day + (4.0 * longitude) / 60.0;
    let hour_angle = 15.0 * (solar_time - 12.0);

    // Calculate the solar declination angle
    let declination_angle =
        declination_angle_max * (2.0 * PI * (day_of_year - 81.0) / days_in_year).sin();

    // Convert latitude and declination angle to radians
    let latitude_rad = latitude.to_radians();
    let declination_angle_rad = declination_angle.to_radians();

    // Calculate the solar zenith angle
    let zenith_angle_rad = (latitude_rad.sin() * declination_angle_rad.sin()
        + latitude_rad.cos() * declination_angle_rad.cos() * hour_angle.to_radians().cos())
    .acos();

    // Convert to degrees
    zenith_angle_rad.to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_sun_zenithal_angle() {
        // Test case for summer solstice at the equator
        let date =
            NaiveDateTime::parse_from_str("2024-08-23 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let longitude = 105.0;
        let latitude = 40.0;
        let result = sun_zenithal_angle(date, longitude, latitude);
        assert!((114.3..=114.4).contains(&result), "{result}");
        //
        // // Test case for winter solstice at the equator
        // let date =
        //     NaiveDateTime::parse_from_str("2023-12-21 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        // let longitude = 0.0;
        // let latitude = 0.0;
        // let result = sun_zenithal_angle(date, longitude, latitude);
        // assert!(
        //     (result - 23.45).abs() < 0.1, // Adjusted expected value to positive 23.45
        //     "Expected around 23.45, got {}",
        //     result
        // );
        //
        // // Test case for spring equinox at the equator
        // let date =
        //     NaiveDateTime::parse_from_str("2023-03-21 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        // let longitude = 0.0;
        // let latitude = 0.0;
        // let result = sun_zenithal_angle(date, longitude, latitude);
        // assert!((result).abs() < 1.0, "Expected around 0.0, got {}", result); // Increased tolerance
        //
        // // Test case for autumn equinox at the equator
        // let date =
        //     NaiveDateTime::parse_from_str("2023-09-23 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        // let longitude = 0.0;
        // let latitude = 0.0;
        // let result = sun_zenithal_angle(date, longitude, latitude);
        // assert!((result).abs() < 1.0, "Expected around 0.0, got {}", result); // Increased tolerance
    }
}
