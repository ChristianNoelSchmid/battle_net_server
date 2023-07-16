

use std::sync::Arc;

use axum::{async_trait, extract::{FromRequestParts, State}, http::{request::Parts, StatusCode, Request}, TypedHeader, headers::{Authorization, authorization::Bearer}, middleware::Next, response::Response};

use dotenvy::dotenv;
use dotenv_codegen::dotenv;

use lazy_static::lazy_static;

use crate::services::token_service::TokenService;

lazy_static! {
    static ref ADMIN_SECRET: &'static str = {
        dotenv().ok();
        dotenv!("ADMIN_SECRET")
    };
}

#[derive(Clone)]
pub struct AuthContext { pub user_id: i64 }

#[derive(Clone)]
pub struct AdminContext;

#[async_trait]
impl <S : Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        return if let Some(info) = parts.extensions.get::<AuthContext>() {
            Ok(info.clone())
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized. Please sign in".to_string()))
        };
    }
}

#[async_trait]
impl <S : Send + Sync> FromRequestParts<S> for AdminContext {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        return if let Some(info) = parts.extensions.get::<AdminContext>() {
            Ok(info.clone())
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized. Admin only".to_string()))
        }
    }
}

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
            let result = token_service.verify_access_token(access_token);

            if let Ok(user_id) = result {
                request.extensions_mut().insert(AuthContext { user_id });
            }
        }
    }
    next.run(request).await
}