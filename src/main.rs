mod config;
mod solar;

use config::Config;
use std::path::Path;
mod readers;

use readers::*;
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

    let file_name =
        Path::new(&"/media/LaCie16TB/work/projects/workshops/ppr/data/sst_st_lawrence_river.tif");

    // Step 2: Create the appropriate reader, will be based on the file extension
    let reader = readers::create_reader(file_name.to_str().unwrap().to_string()).unwrap();

    // Step 3: Use the reader to read data
    let data = reader.read_data().unwrap();
    println!("{}", data);
}
