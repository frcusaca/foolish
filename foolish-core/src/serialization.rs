use crate::fir::Fir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerdeError {
    #[error("encode failed: {0}")]
    Encode(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("decode failed: {0}")]
    Decode(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub trait FirSerializer: Send + Sync {
    fn encode(&self, fir: &Fir) -> Result<Vec<u8>, SerdeError>;
    fn decode(&self, data: &[u8]) -> Result<Fir, SerdeError>;
    fn encode_to_string(&self, fir: &Fir) -> Result<String, SerdeError>;
    fn decode_from_str(&self, text: &str) -> Result<Fir, SerdeError>;
}

#[derive(Debug, Default)]
pub struct JsonSerializer;

impl FirSerializer for JsonSerializer {
    fn encode(&self, fir: &Fir) -> Result<Vec<u8>, SerdeError> {
        serde_json::to_vec(fir).map_err(|e| SerdeError::Encode(Box::new(e)))
    }

    fn decode(&self, data: &[u8]) -> Result<Fir, SerdeError> {
        serde_json::from_slice(data).map_err(|e| SerdeError::Decode(Box::new(e)))
    }

    fn encode_to_string(&self, fir: &Fir) -> Result<String, SerdeError> {
        serde_json::to_string_pretty(fir).map_err(|e| SerdeError::Encode(Box::new(e)))
    }

    fn decode_from_str(&self, text: &str) -> Result<Fir, SerdeError> {
        serde_json::from_str(text).map_err(|e| SerdeError::Decode(Box::new(e)))
    }
}

pub type DefaultSerializer = JsonSerializer;

pub fn fir_to_json(fir: &Fir) -> Result<String, SerdeError> {
    JsonSerializer.encode_to_string(fir)
}

pub fn fir_from_json(text: &str) -> Result<Fir, SerdeError> {
    JsonSerializer.decode_from_str(text)
}
