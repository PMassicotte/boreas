use super::types::{FileError, FileType};
use std::path::Path;

pub fn reader_from_filetype(path: &Path) -> Result<FileType, FileError> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("tif") => Ok(FileType::GeoTiff),
        Some("nc") => Ok(FileType::NetCDF),
        Some("zarr") => Ok(FileType::Zarr),
        _ => Err(FileError::UnknownFileType),
    }
}
