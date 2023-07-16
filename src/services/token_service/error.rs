use axum::{response::{IntoResponse, Response}, http::StatusCode};

pub type Result<T> = std::result::Result<T, TokenError>;

#[derive(Debug)]
pub enum TokenError {
    TokenStale,
    JwtError(jwt::Error)
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        return match &self {
            TokenError::JwtError(error) => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            TokenError::TokenStale => {
                (StatusCode::UNAUTHORIZED, "Refresh token stale - please login again").into_response()
            }
        }
    }
}