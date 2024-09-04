use super::{Data, DataReader, ReadError};
use std::fs::File;
use std::io::BufReader;
use tiff::decoder::{Decoder, DecodingResult};

pub struct GeoTiffReader {
    pub file_name: String,
}

impl DataReader for GeoTiffReader {
    fn read_data(&self) -> Result<Data, ReadError> {
        let file = File::open(&self.file_name)
            .map_err(|e| ReadError::GeoTiff(format!("Failed to open file: {}", e)))?;

        let reader = BufReader::new(file);

        let mut decoder = Decoder::new(reader)
            .map_err(|e| ReadError::GeoTiff(format!("Failed to decode TIFF: {}", e)))?;

        let (width, height) = decoder
            .dimensions()
            .map_err(|e| ReadError::GeoTiff(format!("Failed to get dimensions: {}", e)))?;

        let image_data: Vec<f32> = match decoder
            .read_image()
            .map_err(|e| ReadError::GeoTiff(format!("Failed to read image: {}", e)))?
        {
            DecodingResult::U8(data) => data.iter().map(|&x| x as f32).collect(),
            DecodingResult::U16(data) => data.iter().map(|&x| x as f32).collect(),
            DecodingResult::U32(data) => data.iter().map(|&x| x as f32).collect(),
            DecodingResult::F32(data) => data,
            DecodingResult::F64(data) => data.iter().map(|&x| x as f32).collect(),
            _ => return Err(ReadError::GeoTiff("Unsupported pixel format".to_string())),
        };

        Ok(Data {
            width,
            height,
            buffer: image_data,
        })
    }
}
