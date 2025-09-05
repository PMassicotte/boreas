mod config;
mod lut;
mod readers;

use chrono::Datelike;
use config::Config;
use lut::Lut;
use lut::SolarPosition;

fn main() {
    let config = match Config::from_file("./data/config/simple_config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    println!("{:#?}", config);

    // Just a test: take the first data, and make sunpos calcultion every 3 hours
    for date in config.take(1) {
        println!("{}", date);

        let hours: Vec<f32> = (0..8).map(|i| i as f32 * 3.0).collect();

        hours.iter().for_each(|hour| {
            let sun_position = SolarPosition::calculate(date.ordinal() as i16, *hour, 45., -50.);
            println!(
                "Sun position for {} at {}h: {:#?}",
                date, hour, sun_position
            );
        });

        // This function is a helper, gives the same info on zenith and azumuth, maybe use this
        // instead of sunpos()
        // let res2 = sunpos::sunpos_simple(date.ordinal() as i16, hour, 45., -50.);
        // println!("res2 {:?}", res2);
    }

    let reader = readers::create_reader("./data/sst_st_lawrence_river.tif".to_string()).unwrap();

    // Step 3: Use the reader to read data
    let data = reader.read_data().unwrap();
    println!("{}", data);

    // lut

    let lut = Lut::from_file("./data/Ed0moins_LUT_5nm_v2.dat").unwrap();

    let ed0 = lut.ed0moins(5.0, 350.0, 16.0, 0.5, 0.05);

    print!("{:?}", ed0);
}
