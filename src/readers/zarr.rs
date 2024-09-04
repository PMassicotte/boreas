use super::{Data, DataReader, ReadError};

pub struct ZarrReader {
    pub file_name: String,
}

impl DataReader for ZarrReader {
    fn read_data(&self) -> Result<Data, ReadError> {
        Err(ReadError::Zarr("Zarr reading not implemented".to_string()))
    }
}
