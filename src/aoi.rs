use crate::bbox::Bbox;
use gdal::spatial_ref::{AxisMappingStrategy, CoordTransform, SpatialRef};
use gdal::vector::{Geometry, LayerAccess, OGRwkbGeometryType};
use geo::{BoundingRect, Contains};
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon};

pub enum Aoi {
    Bbox(Bbox),
    Polygon(PolygonAoi),
}

#[derive(Debug)]
pub struct PolygonAoi {
    geometry: MultiPolygon<f64>,
    envelope: Bbox,
}

impl Aoi {
    pub fn bounding_box(&self) -> &Bbox {
        match self {
            Aoi::Bbox(b) => b,
            Aoi::Polygon(b) => &b.envelope,
        }
    }

    pub fn mask(
        &self,
        geotransform: &[f64; 6],
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
    ) -> Vec<bool> {
        match self {
            // bbox: every pixel in the window is kept. All pixels set to true.
            Aoi::Bbox(_) => vec![true; (width * height) as usize],

            Aoi::Polygon(p) => {
                let mut mask = Vec::with_capacity((width * height) as usize);

                for row in 0..height {
                    for col in 0..width {
                        // Pixel center in raster coordinates. Get the pixel position, and convert
                        // that into lon/lat
                        let px = (start_x + col) as f64 + 0.5;
                        let py = (start_y + row) as f64 + 0.5;
                        let lon = geotransform[0] + px * geotransform[1] + py * geotransform[2];
                        let lat = geotransform[3] + px * geotransform[4] + py * geotransform[5];

                        mask.push(p.geometry.contains(&Point::new(lon, lat)));
                    }
                }

                mask
            }
        }
    }
}

impl PolygonAoi {
    pub fn from_file(
        path: &str,
        layer_name: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dataset = gdal::Dataset::open(path)?;

        let mut layer = match layer_name {
            Some(name) => dataset.layer_by_name(name)?,
            None => dataset.layer(0)?,
        };

        let layer_csr = layer.spatial_ref();

        let mut polygons: Vec<geo_types::Polygon<f64>> = Vec::new();

        let mut wgs84 = SpatialRef::from_epsg(4326)?;
        // Force x=lon, y=lat output (otherwise GDAL 3+ returns lat,lon for EPSG:4326).
        wgs84.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);

        for feature in layer.features() {
            if let Some(geom) = feature.geometry() {
                let geom_wgs84: Geometry = match layer_csr.as_ref() {
                    Some(src) => {
                        let mut src = src.clone();
                        src.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);

                        let transform = CoordTransform::new(&src, &wgs84)?;

                        let mut g = geom.clone();
                        g.transform_inplace(&transform)?;
                        g
                    }

                    // No source CRS on the layer: assume it is already WGS84 lon/lat.
                    None => geom.clone(),
                };

                walk(&geom_wgs84, &mut polygons);
            }
        }

        if polygons.is_empty() {
            return Err("Vector file contains no polygons geometry".into());
        }

        let geometry = MultiPolygon(polygons);

        let rect = geometry
            .bounding_rect()
            .ok_or("Poygon geometry hoas no bounding rectangle/bbox")?;

        let envelope = Bbox::new(rect.min().x, rect.max().x, rect.min().y, rect.max().y)?;

        Ok(Self { geometry, envelope })
    }
}

/// Recursively pull every POLYGON out of a geometry (handles MULTIPOLYGON / collections).
fn walk(geom: &Geometry, out: &mut Vec<Polygon<f64>>) {
    match geom.geometry_type() {
        OGRwkbGeometryType::wkbPolygon => {
            out.push(gdal_polygon_to_geo(geom));
        }
        // MultiPolygon / GeometryCollection: recurse into each child geometry.
        _ => {
            for i in 0..geom.geometry_count() {
                walk(&geom.get_geometry(i), out);
            }
        }
    }
}

/// A GDAL polygon's child geometries are its rings: ring 0 = exterior, rest = holes.
fn gdal_polygon_to_geo(poly: &Geometry) -> Polygon<f64> {
    let ring_to_line = |ring: &Geometry| -> LineString<f64> {
        // get_points appends (x, y, z); we only need x/y.
        let mut pts = Vec::new();
        ring.get_points(&mut pts);
        pts.into_iter().map(|(x, y, _z)| Coord { x, y }).collect()
    };

    let exterior = ring_to_line(&poly.get_geometry(0));

    let holes: Vec<LineString<f64>> = (1..poly.geometry_count())
        .map(|i| ring_to_line(&poly.get_geometry(i)))
        .collect();

    Polygon::new(exterior, holes)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_shape_import_baffin_bay() {
        let path = "./data/baffin_bay.gpkg";

        let poly1 = PolygonAoi::from_file(path, None).unwrap();
        insta::assert_debug_snapshot!("no_layer_name", poly1);

        let poly2 = PolygonAoi::from_file(path, Some("baffin_bay")).unwrap();
        insta::assert_debug_snapshot!("with_layer_name", poly2);

        assert_eq!(poly1.geometry, poly2.geometry);

        let area = geo::centroid::Centroid::centroid(&poly1.geometry);
        println!("Area: {:?}", area);
    }
}
