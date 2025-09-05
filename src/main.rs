mod config;
mod lut;
mod readers;

use lut::Lut;

use std::path::Path;

use config::Config;

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

    let file_name = Path::new(&"./data/sst_st_lawrence_river.tif");
    let reader = readers::create_reader(file_name.to_str().unwrap().to_string()).unwrap();

    // Step 3: Use the reader to read data
    let data = reader.read_data().unwrap();
    println!("{}", data);

    // lut

    let lut = Lut::from_file("./data/Ed0moins_LUT_5nm_v2.dat").unwrap();

    let ed0 = lut.ed0moins(5.0, 350.0, 16.0, 0.5, 0.05);

    print!("{:?}", ed0);
}
