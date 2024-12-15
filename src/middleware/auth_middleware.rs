

use std::sync::Arc;

use axum::{async_trait, extract::{FromRequestParts, State}, http::{request::Parts, StatusCode, Request}, TypedHeader, headers::{Authorization, authorization::Bearer}, middleware::Next, response::Response};

use dotenvy::dotenv;
use dotenv_codegen::dotenv;

use lazy_static::lazy_static;

use crate::services::token_service::TokenService;

lazy_static! {
    /// 
    /// The secret for admin functionality. Admin routes
    /// will need to provide this secret using "Authorization: Bearer"
    /// 
    static ref ADMIN_SECRET: &'static str = {
        dotenv().ok();
        dotenv!("ADMIN_SECRET")
    };
}

///
/// Context for a specific authorized user
/// 
#[derive(Copy, Clone)]
pub struct AuthContext { pub user_id: i64 }

///
/// Context for admin functionality.
/// The context itself provides access, there is
/// no data associated with the context.
/// 
#[derive(Clone)]
pub struct AdminContext;

#[async_trait]
impl <S : Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Check if the request has an AuthContext in the extensions, and return it if so
        return if let Some(info) = parts.extensions.get::<AuthContext>() {
            Ok(info.clone())
        // Otherwise return an unauthorized message
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized. Please sign in".to_string()))
        };
    }
}

#[async_trait]
impl <S : Send + Sync> FromRequestParts<S> for AdminContext {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Check if the request has an AdminContext in the extensions, and return it if so
        return if let Some(info) = parts.extensions.get::<AdminContext>() {
            Ok(info.clone())
        // Otherwise return an unauthorized message
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized. Admin only".to_string()))
        }
    }
}

///
/// Middleware that handles access token authentication.
/// Expects token to be provided with header `"Authorization: bearer"`.
/// Allows both JWT authentication for a specific user, and admin
/// authentication when provided the admin secret.
/// 
pub async fn auth_middleware<B : Send> (
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(token_service): State<Arc<dyn TokenService>>,
    mut request: Request<B>,
    next: Next<B>
) -> Response {
    if let Some(bearer) = bearer {
        let access_token = bearer.token().to_string();

        if access_token == ADMIN_SECRET.to_string() {
            request.extensions_mut().insert(AdminContext);
        } else {
            let result = token_service.verify_access_token(&access_token);

            if let Ok(user_id) = result {
                request.extensions_mut().insert(AuthContext { user_id });
            }
        }
    }
    next.run(request).await
}