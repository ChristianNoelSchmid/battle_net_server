pub mod error;
pub mod data_layer;
pub mod dtos;

use std::sync::Arc;

use argon2::Config;
use axum::async_trait;
use derive_more::Constructor;
use dotenvy::dotenv;
use dotenv_codegen::dotenv;
use lazy_static::lazy_static;

use crate::{data_layer_error, models::auth_models::RefrTokenModel};

use self::{error::{Result, AuthServiceError}, data_layer::AuthDataLayer};

use super::token_service::{TokenService, dtos::TokensDto};

lazy_static! {
    static ref SALT: String = {
        dotenv().ok().expect(".env file must be provided");
        dotenv!("SALT").to_string()
    };
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn try_accept_creds(&self, email: String, pwd: String) -> Result<TokensDto>;
    async fn try_accept_refresh(&self, refr_token: String) -> Result<TokensDto>;
    async fn create_new_user(&self, email: String, pwd: String, card_idx: i32) -> Result<()>;
}

#[derive(Clone, Constructor)]
pub struct CoreAuthService {
    data_layer: Arc<dyn AuthDataLayer>,
    token_service: Arc<dyn TokenService>,
}

#[async_trait]
impl AuthService for CoreAuthService {
    async fn try_accept_creds(&self, email: String, pwd: String) -> Result<TokensDto> {
        // Get the user associated with the email (if exists)
        let user = self.data_layer.get_user_by_email(email.clone()).await
            .map_err(|e| e.into())?;

        if let Some(user) = user {
            // Verify that the password given matches the user's
            let matches = argon2::verify_encoded(&user.pwd_hash, pwd.as_bytes()).unwrap();

            // If matches, add the new refresh token and return the tokens
            if matches {
                let tokens = self.token_service.generate_auth_tokens(user.id);
                self.data_layer.create_refr_token(user.id, tokens.refr_token.value().to_string())
                    .await.map_err(|e| AuthServiceError::DataLayerError(e))?;
                
                return Ok(tokens);
            } else {
                return Err(AuthServiceError::PasswordDoesNotMatch(user.email))
            };
        }

        Err(AuthServiceError::EmailDoesNotExist(email))
    }

    async fn try_accept_refresh(&self, token: String) -> Result<TokensDto> {
        // Attempt to query the refresh token that matches the token given
        let refr_token = self.data_layer.get_refr_token_by_token(token).await
            .map_err(|e| AuthServiceError::DataLayerError(e))?;

        // Ensure the refresh token in question exists
        if let Some(refr_token) = refr_token {
            // Ensure the refresh token hasn't already been revoked
            if refr_token.revoked_on.is_some() {
                // If it has, revoke it's descendent refresh token,
                // and return an error
                let revoked_id = revoke_token(refr_token.clone(), &self.data_layer).await.map_err(|e| e.into())?;

                let error = Err(
                    AuthServiceError::DuplicateRefresh { 
                        user_id: refr_token.user_id.clone(), 
                        dup_id: refr_token.id,
                        revoked_id
                    }
                );
                return error;
            }

            // Get the user associated with the refresh token
            let user = self.data_layer.get_user_by_id(refr_token.user_id).await.map_err(|e| e.into())?;
                

            if let Some(user) = user {
                // Generate a new access and refresh token
                let tokens = self.token_service.generate_auth_tokens(user.id);

                // Add the new refresh token to the db
                let repl_id = self.data_layer.create_refr_token(user.id, tokens.refr_token.value().to_string())
                    .await.map_err(|e| e.into())?;

                // Update the old token's replacement to this one
                self.data_layer.revoke_refr_token(refr_token.id, Some(repl_id), "CLIENT".to_string())
                    .await.map_err(|e| e.into())?;

                return Ok(tokens);
            } else {
                return Err(AuthServiceError::UserNotFound(refr_token.user_id, refr_token.id));
            }
        }
        return Err(AuthServiceError::TokenDoesNotExist);
    }
    
    async fn create_new_user(&self, email: String, pwd: String, card_idx: i32) -> Result<()> {
        let pwd_hash = argon2::hash_encoded(pwd.as_bytes(), SALT.as_bytes(), &Config::default()).unwrap();
        self.data_layer.create_user(email, pwd_hash, card_idx).await.map_err(|e| e.into())?;

        Ok(())
    }
}

async fn revoke_token(refr_token: RefrTokenModel, data_layer: &Arc<dyn AuthDataLayer>) -> data_layer_error::Result<i32> {
    let mut desc_token = refr_token;

    // Traverse down the descendent token line, finding the
    // current valid token (if it is still valid) 
    while let Some(next_token_id) = desc_token.repl_id  {
        desc_token = data_layer.get_refr_token_by_id(next_token_id).await?.unwrap();
    }
    
    data_layer.revoke_refr_token(desc_token.id, None, "SERVER (DUPL. USAGE)".to_string()).await?;

    Ok(desc_token.id)
}