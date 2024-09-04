use super::{Data, DataReader, ReadError};

pub struct NcReader {
    pub file_name: String,
}

impl DataReader for NcReader {
    fn read_data(&self) -> Result<Data, ReadError> {
        Err(ReadError::NetCDF(
            "NetCDF reading not implemented".to_string(),
        ))
    }
}
