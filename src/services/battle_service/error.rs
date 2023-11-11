use axum::{response::{IntoResponse, Response}, http::StatusCode};
use log::error;
use thiserror::Error;

use crate::{data_layer_error::DataLayerError, services::quest_service::error::QuestServiceError};

pub type Result<T> = std::result::Result<T, BattleServiceError>;

#[derive(Debug, Error)]
pub enum BattleServiceError {
    #[error("An internal server error has occurred")]
    DataLayerError(DataLayerError),
    #[error("An internal server error has occurred")]
    QuestServiceError(QuestServiceError),
    #[error("Quest not found for user {0}")]
    QuestNotFound(i32)
}

impl Into<BattleServiceError> for DataLayerError {
    fn into(self) -> BattleServiceError {
        BattleServiceError::DataLayerError(self)
    }
}

impl IntoResponse for BattleServiceError {
    fn into_response(self) -> Response {
        return if let BattleServiceError::DataLayerError(e) = &self {
            error!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
        } else if let BattleServiceError::QuestServiceError(e) = &self {
            error!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
        } else {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}
