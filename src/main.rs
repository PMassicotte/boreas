mod config;
mod readers;
mod solar;

use chrono::{TimeZone, Utc};
use std::path::Path;

use config::Config;
use solar::solar_zenith_angle;

fn main() {
    let config = match Config::from_file("./data/config/simple_config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    println!("{:#?}", config);

    for date in config {
        println!("{}", date);
    }

    let file_name = Path::new(
        &"/media/LaCie16TB/work/projects/workshops/journee_ppr_ulaval_2024/data/sst_st_lawrence_river.tif",
    );

    // Step 2: Create the appropriate reader, will be based on the file extension
    let reader = readers::create_reader(file_name.to_str().unwrap().to_string()).unwrap();

    // Step 3: Use the reader to read data
    let data = reader.read_data().unwrap();
    println!("{}", data);

    // Example: 2024-01-01T12:00:00Z, latitude 40.7128 (NYC), longitude -74.0060 (NYC)
    let utc_time = Utc.with_ymd_and_hms(2024, 8, 11, 16, 38, 55);
    let latitude = 40.7128;
    let longitude = -74.0060;
    let zenith_angle = solar_zenith_angle(utc_time.unwrap(), latitude, longitude);
    println!("Solar Zenith Angle: {:.2} degrees", zenith_angle);
}
