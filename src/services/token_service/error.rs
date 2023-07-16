use axum::{response::{IntoResponse, Response}, http::StatusCode};
use log::error;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, TokenError>;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Refresh token stale - please login again")]
    TokenStale,
    #[error("An error has occurred")]
    JwtError(jwt::Error)
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        error!("{:?}", self);
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}