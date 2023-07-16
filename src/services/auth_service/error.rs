use axum::{response::{IntoResponse, Response}, http::StatusCode};
use log::error;
use thiserror::Error;

use crate::data_layer_error::DataLayerError;

pub type Result<T> = std::result::Result<T, AuthServiceError>;

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("Refresh token cookie not found")]
    CookieNotFound,
    #[error("An internal server error has occurred")]
    DataLayerError(DataLayerError),
    #[error("The given email {0} doesn't exist")]
    EmailDoesNotExist(String),
    #[error("Password does not match for email {0}")]
    PasswordDoesNotMatch(String),
    #[error("Refresh token duplicate usage. duplicate ID `{dup_id}`, revoked ID `{revoked_id}`, user ID `{user_id}`")]
    DuplicateRefresh { user_id: i64, dup_id: i64, revoked_id: i64 },
    #[error("The token provided doesn't exist")]
    TokenDoesNotExist,
    #[error("The given user cannot be found")]
    UserNotFound(i64, i64)
}

impl Into<AuthServiceError> for DataLayerError {
    fn into(self) -> AuthServiceError {
        AuthServiceError::DataLayerError(self)
    }
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        return if let AuthServiceError::DataLayerError(e) = &self {
            error!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
        } else {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}
