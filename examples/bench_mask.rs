use boreas::aoi::{Aoi, PolygonAoi};
use std::time::Instant;

fn main() {
    let t = Instant::now();
    let poly = PolygonAoi::from_file("./data/baffin_bay.gpkg", None).unwrap();
    println!("load polygon: {:?}", t.elapsed());

    let aoi = Aoi::Polygon(poly);

    // North-up geotransform at ~4 km, covering the envelope.
    let gt = [-83.28, 0.0417, 0.0, 80.0, 0.0, -0.0417];
    let w = 808u32;
    let h = 384u32;

    let t = Instant::now();
    let mask = aoi.mask(&gt, 0, 0, w, h);
    let inside = mask.iter().filter(|&&b| b).count();
    println!(
        "mask {}x{} = {} px in {:?} ({} inside)",
        w,
        h,
        mask.len(),
        t.elapsed(),
        inside
    );
}
