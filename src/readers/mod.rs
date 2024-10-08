// TODO: Remove eventually
#![allow(dead_code)]

pub mod geotiff;
pub mod nc;
pub mod types;
pub mod utils;
pub mod zarr;

pub use geotiff::GeoTiffReader;
pub use nc::NcReader;
pub use types::{Data, DataReader, FileError, FileType, ReadError};
pub use utils::reader_from_filetype;
pub use zarr::ZarrReader;

pub fn create_reader(file_name: String) -> Result<Box<dyn DataReader>, FileError> {
    match reader_from_filetype(file_name.as_ref()) {
        Ok(FileType::GeoTiff) => Ok(Box::new(GeoTiffReader { file_name })),
        Ok(FileType::NetCDF) => Ok(Box::new(NcReader { file_name })),
        Ok(FileType::Zarr) => Ok(Box::new(ZarrReader { file_name })),
        Err(e) => Err(e),
    }
}
