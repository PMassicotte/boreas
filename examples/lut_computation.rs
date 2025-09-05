use boreas::lut::lookup_table::Lut;

fn main() {
    let lut = Lut::from_file("./data/Ed0moins_LUT_5nm_v2.dat").unwrap();

    let ed0 = lut.ed0moins(5.0, 350.0, 16.0, 0.5, 0.05);

    print!("{:?}", ed0);
}
