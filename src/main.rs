mod config;
mod lut;
mod readers;

use chrono::Datelike;
use config::Config;
use lut::Lut;
use lut::sunpos;

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

        // Use sunpos with the current date
        let hour = 12.0;
        let sun_position = sunpos(date.ordinal() as i16, hour, 45., -50.);
        println!(
            "Sun position for {} at {}h: {:#?}",
            date, hour, sun_position
        );

        // This function is a helper, gives the same info on zenith and azumuth, maybe use this
        // instead of sunpos()
        let res2 = sunpos::sunpos_simple(date.ordinal() as i16, hour, 45., -50.);
        println!("res2 {:?}", res2);
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
