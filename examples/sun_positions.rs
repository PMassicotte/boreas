use boreas::config::Config;
use boreas::date_gen::DateTimeGenerator;
use boreas::lut::sunpos::SolarPosition;
use chrono::{Datelike, Timelike};

fn main() {
    let config = match Config::from_file("./data/config/simple_config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let generator = DateTimeGenerator::new(config.clone());
    let datetime_series = generator.generate_datetime_series();

    for dt in datetime_series {
        // Extract Julian day and hour from datetime
        let julian_day = dt.ordinal() as i16;
        let hour = dt.hour() as f32 + (dt.minute() as f32 / 60.0);

        // Using placeholder coordinates (Montreal, Canada as example)
        let latitude = 45.5017;
        let longitude = -73.5673;

        let sun_position = SolarPosition::calculate(julian_day, hour, latitude, longitude);

        println!(
            "DateTime: {}, Julian Day: {}, Hour: {:.2}, Zenith: {:.2}°, Azimuth: {:.2}°",
            dt.format("%Y-%m-%d %H:%M"),
            julian_day,
            hour,
            sun_position.zenith_angle_deg,
            sun_position.azimuth_angle_deg
        );
    }
}
