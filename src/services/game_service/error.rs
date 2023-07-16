use axum::{response::{IntoResponse, Response}, http::StatusCode};
use log::error;
use thiserror::Error;

use crate::{data_layer_error::DataLayerError, services::auth_service::error::AuthServiceError};

pub type Result<T> = std::result::Result<T, GameServiceError>;

#[derive(Debug, Error)]
pub enum GameServiceError {
    #[error("Error with AuthService invocation")]
    AuthServiceError(AuthServiceError),
    #[error("Internal server error")]
    DataLayerError(DataLayerError),
    #[error("Game already running")]
    GameAlreadyRunning,
    #[error("Game is not running")]
    GameNotRunning
}

impl Into<GameServiceError> for DataLayerError {
    fn into(self) -> GameServiceError {
        GameServiceError::DataLayerError(self)
    }
}

impl Into<GameServiceError> for AuthServiceError {
    fn into(self) -> GameServiceError {
        GameServiceError::AuthServiceError(self)
    }
}

impl IntoResponse for GameServiceError {
    fn into_response(self) -> Response {
        match self {
            GameServiceError::DataLayerError(e) => {
                error!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal server error occured").into_response()
            },
            _ => (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}
