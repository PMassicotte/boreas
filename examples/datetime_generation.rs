use boreas::config::Config;
use boreas::date_gen::DateTimeGenerator;

fn main() {
    let config = match Config::from_file("./data/config/simple_config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    println!("{:#?}", config);

    let generator = DateTimeGenerator::new(config.clone());
    let datetime_series = generator.generate_datetime_series();

    println!("\nGenerated {} datetime points:", datetime_series.len());
    for (i, datetime) in datetime_series.iter().take(10).enumerate() {
        println!("  {}: {}", i + 1, datetime);
    }

    println!("{:?}", generator.generate_date_series());
}
