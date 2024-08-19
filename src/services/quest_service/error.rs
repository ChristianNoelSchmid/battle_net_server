use axum::{response::IntoResponse, http::StatusCode};
use log::error;
use thiserror::Error;

use crate::{data_layer_error::DataLayerError, services::game_service::error::GameServiceError};

pub type Result<T> = std::result::Result<T, QuestServiceError>;

#[derive(Debug, Error)]
pub enum QuestServiceError {
    #[error("An internal server error occured")]
    DataLayerError(DataLayerError),
    #[error("An internal server error occured")]
    GameServiceError(GameServiceError),
    #[error("User is not currently on a quest")]
    UserNotOnQuest,
    #[error("User is not currently on a riddle quest")]
    UserNotOnRiddleQuest,
    #[error("User has an active, daily quest")]
    QuestAlreadyActive,
    #[error("User has completed all riddles")]
    AllRiddlesCompleted,
    #[error("Only one riddle quest can be completed a day")]
    PlayerAlreadyCompletedRiddle,
    #[error("Player is exhausted, and cannot start a battle quest today.")]
    PlayerIsExhausted
}

impl Into<QuestServiceError> for DataLayerError {
    fn into(self) -> QuestServiceError {
        QuestServiceError::DataLayerError(self)
    }
}

impl Into<QuestServiceError> for GameServiceError {
    fn into(self) -> QuestServiceError {
        QuestServiceError::GameServiceError(self)
    }
}

impl IntoResponse for QuestServiceError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            QuestServiceError::DataLayerError(qse) => {
                error!("DataLayerError: {:?}", qse);
                return (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response();
            },
            QuestServiceError::GameServiceError(gse) => {
                if let GameServiceError::DataLayerError(e) = gse {
                    error!("DataLayerError: {:?}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response();
                } else {
                    return (StatusCode::BAD_REQUEST, gse.to_string()).into_response();
                }
            },
            QuestServiceError::UserNotOnQuest => return (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            _ => return (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}
