use boreas::iop::constants;
use boreas::iop::qaa_v6;
use boreas::sat_bands::SatBands;
use boreas::sat_bands::Satellites;
use boreas::utils::add_maps;
use std::collections::BTreeMap;

fn main() {
    let r = constants::APHSTAR_ALL.get(&443);
    let t = constants::BBW_ALL.get(&443);

    println!("{:?} {:?}", r, t);

    println!(
        "{:?}",
        add_maps(&constants::APHSTAR_ALL, &constants::BBW_ALL)
    );

    // Testing some Satellites
    let modis = SatBands::new(Satellites::Modis);
    println!("{}", modis);
    println!("Closest band to 440 in modis: {}", modis.closest_band(440));

    let seawifs = SatBands::new(Satellites::SeaWiFS);
    println!("{}", seawifs);
    println!(
        "Closest band to 700 in seawifs: {}",
        seawifs.closest_band(700)
    );

    let rrs = BTreeMap::from([
        (412, 0.001974),
        (443, 0.002570),
        (469, 0.003086),
        (488, 0.002974),
        (531, 0.002174),
        (547, 0.001862),
        (555, 0.001670),
        (645, 0.000324),
        (667, 0.000324),
        (678, 0.000324),
    ]);

    let res = qaa_v6(&rrs, Satellites::Modis);

    println!("{:#?}", res);
    println!("{:?}", res.get_messages());
}
