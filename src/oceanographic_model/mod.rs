use std::path::Path;
pub mod batch_process;
pub mod pixel;
pub mod processor;

pub use processor::OceanographicProcessor;

pub fn is_supported_file_type(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("tif") | Some("nc")
    )
}
