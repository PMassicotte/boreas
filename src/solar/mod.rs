// https://github.com/pyenergyplus/pysunNOAA/tree/main/pysunnoaa
// TODO: This is not giving the right answer...

use chrono::{DateTime, Datelike, Timelike, Utc};
use std::f64::consts::PI;

/// Convert UTC time to Julian Day
fn utc_to_julian_day(datetime: DateTime<Utc>) -> f64 {
    let year = datetime.year();
    let month = datetime.month() as i32;
    let day = datetime.day() as i32;
    let hour = datetime.hour() as f64;
    let minute = datetime.minute() as f64;
    let second = datetime.second() as f64;

    367.0 * year as f64 - ((7.0 * ((year + ((month + 9) / 12)) as f64)) / 4.0).floor()
        + ((275.0 * month as f64) / 9.0).floor()
        + day as f64
        + 1721013.5
        + (hour + minute / 60.0 + second / 3600.0) / 24.0
}

/// Calculate solar declination
fn solar_declination(julian_day: f64) -> f64 {
    let n = julian_day - 2451545.0;
    let mean_longitude = (280.46 + 0.9856474 * n) % 360.0;
    let mean_anomaly = (357.528 + 0.9856003 * n) % 360.0;
    let lambda = mean_longitude
        + 1.915 * (mean_anomaly.to_radians()).sin()
        + 0.02 * (2.0 * mean_anomaly.to_radians()).sin();

    (lambda.to_radians().sin() * 23.44_f64.to_radians()).asin()
}

/// Calculate hour angle using Julian Day
fn hour_angle(julian_day: f64, longitude: f64, time: DateTime<Utc>) -> f64 {
    // Convert UTC time into fractional hours
    let utc_hours =
        time.hour() as f64 + time.minute() as f64 / 60.0 + time.second() as f64 / 3600.0;

    // Julian Century from the epoch J2000.0 (used for solar calculations)
    let julian_century = (julian_day - 2451545.0) / 36525.0;

    // Greenwich Mean Sidereal Time (GMST) at 0h UT in degrees
    let gmst = (280.46061837
        + 360.98564736629 * (julian_day - 2451545.0)
        + 0.000387933 * julian_century.powi(2)
        - julian_century.powi(3) / 38710000.0)
        % 360.0;

    // Local Sidereal Time (LST) = GMST + longitude (in degrees)
    let local_sidereal_time = (gmst + longitude + (utc_hours * 15.0)) % 360.0;
    // println!("{}", local_sidereal_time);

    // Calculate the hour angle (in degrees)
    let hour_angle = (local_sidereal_time - 180.0) % 360.0; // 180° is noon

    // Return hour angle in radians
    hour_angle.to_radians()
}

/// Calculate solar zenith angle
pub fn solar_zenith_angle(utc_time: DateTime<Utc>, lat: f64, long: f64) -> f64 {
    let jd = utc_to_julian_day(utc_time);
    let declination = solar_declination(jd);
    let h_angle = hour_angle(jd, long, utc_time);

    let lat_rad = lat.to_radians();
    let cos_theta =
        lat_rad.sin() * declination.sin() + lat_rad.cos() * declination.cos() * h_angle.cos();
    cos_theta.acos() * 180.0 / PI
}
