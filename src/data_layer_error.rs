use std::error::Error;

pub type Result<T> = std::result::Result<T, DataLayerError>;
pub type DataLayerError = Box<dyn Error + Send + Sync>;