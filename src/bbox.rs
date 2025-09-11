use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Bbox {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
}

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

        Ok(Bbox {
            xmin,
            xmax,
            ymin,
            ymax,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::bbox::Bbox;
    #[test]
    fn test_bbox_coords_are_within_ranges() {
        // Test valid coordinates
        let valid_bbox = Bbox::new(-67.2, -58.7, 70.9, 73.3);
        assert!(valid_bbox.is_ok());

        // Test longitude out of range
        let invalid_lon = Bbox::new(-200.0, 0.0, 0.0, 10.0);
        assert!(invalid_lon.is_err());

        let invalid_lon2 = Bbox::new(0.0, 200.0, 0.0, 10.0);
        assert!(invalid_lon2.is_err());

        // Test latitude out of range
        let invalid_lat = Bbox::new(0.0, 10.0, -100.0, 0.0);
        assert!(invalid_lat.is_err());

        let invalid_lat2 = Bbox::new(0.0, 10.0, 0.0, 100.0);
        assert!(invalid_lat2.is_err());

        // Test min > max
        let invalid_order_lon = Bbox::new(10.0, 0.0, 0.0, 10.0);
        assert!(invalid_order_lon.is_err());

        let invalid_order_lat = Bbox::new(0.0, 10.0, 10.0, 0.0);
        assert!(invalid_order_lat.is_err());
    }
}
