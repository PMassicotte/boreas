use geo_types::{Coord, Point, Rect};
use proj::Proj;

#[derive(Debug, Clone)]
pub struct Bbox(Rect<f64>);

#[allow(dead_code)]
impl Bbox {
    pub fn new(xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> Result<Self, String> {
        if !(-180.0..=180.0).contains(&xmin) || !(-180.0..=180.0).contains(&xmax) {
            return Err("Longitude values must be between -180 and 180".to_string());
        }

        if !(-90.0..=90.0).contains(&ymin) || !(-90.0..=90.0).contains(&ymax) {
            return Err("Latitude values must be between -90 and 90".to_string());
        }

        if xmin > xmax || ymin > ymax {
            return Err("Min values must be <= max values".to_string());
        }

        Ok(Bbox(Rect::new(
            Coord { x: xmin, y: ymin },
            Coord { x: xmax, y: ymax },
        )))
    }

    pub fn xmin(&self) -> f64 {
        self.0.min().x
    }

    pub fn xmax(&self) -> f64 {
        self.0.max().x
    }

    pub fn ymin(&self) -> f64 {
        self.0.min().y
    }

    pub fn ymax(&self) -> f64 {
        self.0.max().y
    }

    pub fn as_rect(&self) -> &Rect<f64> {
        &self.0
    }

    /// Reproject this WGS-84 bounding box into `target_crs` (EPSG code, WKT, or PROJ string).
    /// Only the four corners are transformed; for highly-distorted projections (e.g. polar
    /// stereographic at very high latitudes) the caller should consider sampling more boundary
    /// points to obtain a tight bounding box in the target CRS.
    pub fn transform_to_crs(&self, target_crs: &str) -> Result<Bbox, Box<dyn std::error::Error>> {
        let transformer = Proj::new_known_crs("EPSG:4326", target_crs, None)?;
        let min_t = transformer.convert(Point::new(self.xmin(), self.ymin()))?;
        let max_t = transformer.convert(Point::new(self.xmax(), self.ymax()))?;

        Ok(Bbox(Rect::new(
            Coord {
                x: min_t.x(),
                y: min_t.y(),
            },
            Coord {
                x: max_t.x(),
                y: max_t.y(),
            },
        )))
    }
}

#[cfg(test)]
mod test {
    use crate::bbox::Bbox;

    #[test]
    fn test_bbox_coords_are_within_ranges() {
        let valid_bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3);
        assert!(valid_bbox.is_ok());

        let invalid_lon = Bbox::new(-200.0, 0.0, 0.0, 10.0);
        assert!(invalid_lon.is_err());

        let invalid_lon2 = Bbox::new(0.0, 200.0, 0.0, 10.0);
        assert!(invalid_lon2.is_err());

        let invalid_lat = Bbox::new(0.0, 10.0, -100.0, 0.0);
        assert!(invalid_lat.is_err());

        let invalid_lat2 = Bbox::new(0.0, 10.0, 0.0, 100.0);
        assert!(invalid_lat2.is_err());

        let invalid_order_lon = Bbox::new(10.0, 0.0, 0.0, 10.0);
        assert!(invalid_order_lon.is_err());

        let invalid_order_lat = Bbox::new(0.0, 10.0, 10.0, 0.0);
        assert!(invalid_order_lat.is_err());
    }

    #[test]
    fn test_accessors_round_trip() {
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();
        assert_eq!(bbox.xmin(), -67.2);
        assert_eq!(bbox.xmax(), -58.7);
        assert_eq!(bbox.ymin(), 70.9);
        assert_eq!(bbox.ymax(), 73.3);
    }

    #[test]
    fn test_projection_epsg3411() {
        let bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3).unwrap();
        let projected = bbox.transform_to_crs("EPSG:3411").unwrap();
        insta::assert_snapshot!(format!(
            "xmin={:.2} xmax={:.2} ymin={:.2} ymax={:.2}",
            projected.xmin(),
            projected.xmax(),
            projected.ymin(),
            projected.ymax()
        ));
    }
}
