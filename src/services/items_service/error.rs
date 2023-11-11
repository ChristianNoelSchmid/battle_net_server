use thiserror::Error;

use crate::data_layer_error::DataLayerError;

pub type Result<T> = std::result::Result<T, ItemsServiceError>;

#[derive(Debug, Error)]
pub enum ItemsServiceError {
    #[error("Internal server error")]
    DataLayerError(DataLayerError),
    #[error("No unequipped item found in inventory")]
    NotInInventory,
}

impl Into<ItemsServiceError> for DataLayerError {
    fn into(self) -> ItemsServiceError {
        ItemsServiceError::DataLayerError(self)
    }
}