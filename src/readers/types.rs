use std::fmt;

pub trait DataReader {
    fn read_data(&self) -> Result<Data, ReadError>;
}

#[derive(Debug)]
pub enum ReadError {
    GeoTiff(String),
    NetCDF(String),
    Zarr(String),
}

#[derive(Debug)]
pub enum FileError {
    UnknownFileType,
}

#[derive(Debug)]
pub struct Data {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<f32>,
}

pub enum FileType {
    GeoTiff,
    NetCDF,
    Zarr,
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min_value = self
            .buffer
            .iter()
            .filter(|&&x| !x.is_nan())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&f32::NAN);

        let max_value = self
            .buffer
            .iter()
            .filter(|&&x| !x.is_nan())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(&f32::NAN);

        write!(
            f,
            "Width: {}\nHeight: {}\nBuffer Length: {}\nMin value: {}\nMax value: {}",
            self.width,
            self.height,
            self.buffer.len(),
            min_value,
            max_value,
        )
    }
}
