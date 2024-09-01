mod config;
mod solar;

// use chrono::Duration;
use config::Config;
// use solar::sun_zenithal_angle;

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

    // let start_time = config.start_date().and_hms_opt(0, 0, 0).unwrap();
    //
    // let time_vec: Vec<_> = (0..24)
    //     .map(|h| start_time + Duration::hours(h as i64))
    //     .collect();
    //
    // let res: Vec<f64> = time_vec
    //     .iter()
    //     .map(|t| sun_zenithal_angle(*t, 105.0, 40.0))
    //     .collect();
    //
    // time_vec
    //     .iter()
    //     .zip(res.iter())
    //     .for_each(|(time, sza)| println!("{:<20} {:>8.4}", time, sza));
}
