use boreas::readers;
fn main() {
    let reader = readers::create_reader("./data/sst_st_lawrence_river.tif".to_string()).unwrap();

    // Step 3: Use the reader to read data
    let data = reader.read_data().unwrap();
    println!("{}", data);
}
