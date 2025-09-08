use boreas::iop::qaa_v6;
use boreas::sat_bands::Satellites;
use std::collections::BTreeMap;

fn main() {
    // Simulate rrs values are SeaWiFS wavelengths
    let rrs = BTreeMap::from([
        (410, 0.001974),
        (443, 0.002570),
        (490, 0.002974),
        (555, 0.001670),
        (670, 0.000324),
    ]);

    let result = qaa_v6(&rrs, Satellites::Modis);

    println!("{}", result);
}
