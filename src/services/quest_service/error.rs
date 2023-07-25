use axum::{response::IntoResponse, http::StatusCode};
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

impl IntoResponse for QuestServiceError {
    fn into_response(self) -> axum::response::Response {
        return if let QuestServiceError::DataLayerError(e) = &self {
            (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
        } else {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        };
    }
}