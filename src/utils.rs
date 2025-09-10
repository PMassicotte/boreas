use std::collections::BTreeMap;
use std::path::Path;

/// Adds two BTreeMaps element-wise, returning a new map with the sum of values for each key.
///
/// # Panics
/// Panics if the two maps don't have identical key sets.
///
/// # Arguments
/// * `a` - First map to add
/// * `b` - Second map to add
///
/// # Returns
/// A new BTreeMap containing the element-wise sum of the input maps
pub fn add_maps(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>) -> BTreeMap<u32, f64> {
    // Ensure same keys (wavelengths)
    assert!(
        a.keys().eq(b.keys()),
        "Maps must have the same wavelength keys"
    );

    a.iter()
        .map(|(k, v)| {
            let b_val = b.get(k).unwrap();
            (*k, v + b_val)
        })
        .collect()
}

pub fn is_supported_file_type(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("tif") | Some("nc")
    )
}
