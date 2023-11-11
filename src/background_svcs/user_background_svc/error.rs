use crate::data_layer_error::DataLayerError;

pub type Result<T> = std::result::Result<T, BackgroundUserServiceError>;

#[derive(Debug)]
pub enum BackgroundUserServiceError {
    DataLayerError(DataLayerError),
    GameNotRunning,
}

impl Into<BackgroundUserServiceError> for DataLayerError {
    fn into(self) -> BackgroundUserServiceError {
        BackgroundUserServiceError::DataLayerError(self)
    }
}