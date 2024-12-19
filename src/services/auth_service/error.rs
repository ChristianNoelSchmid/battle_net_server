use axum::{response::{IntoResponse, Response}, http::StatusCode};
use log::error;
use thiserror::Error;

use crate::{data_layer_error::DataLayerError, services::token_service::{self, error::TokenError}};

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
    UserNotFound(i64, i64),
    #[error("An error has occured")]
    TokenServiceError(token_service::error::TokenError),
}

impl From<DataLayerError> for AuthServiceError {
    fn from(value: DataLayerError) -> Self {
        AuthServiceError::DataLayerError(value)
    }
}

impl From<TokenError> for AuthServiceError {
    fn from(value: TokenError) -> Self {
        AuthServiceError::TokenServiceError(value)
    }
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        println!("{:?}", self);
        return if let AuthServiceError::DataLayerError(e) = &self {
            (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
        } else {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}
