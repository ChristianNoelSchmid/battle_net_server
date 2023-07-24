use thiserror::Error;

use crate::data_layer_error::DataLayerError;

pub type Result<T> = std::result::Result<T, QuestServiceError>;

#[derive(Debug, Error)]
pub enum QuestServiceError {
    #[error("An internal server error occured")]
    DataLayerError(DataLayerError),
    #[error("User is not currently on a riddle quest")]
    UserNotOnRiddleQuest
}

impl Into<QuestServiceError> for DataLayerError {
    fn into(self) -> QuestServiceError {
        QuestServiceError::DataLayerError(self)
    }
}