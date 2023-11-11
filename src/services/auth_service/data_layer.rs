use std::sync::Arc;

use axum::async_trait;
use chrono::{Duration, Utc};
use derive_more::Constructor;

use crate::{data_layer_error::Result, services::token_service::settings::TokenSettings, prisma::{PrismaClient, user, refresh_token}};

use super::models::{UserModel, RefrTokenModel};

#[async_trait]
pub trait AuthDataLayer : Send + Sync {
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<UserModel>>;
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserModel>>;

    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenModel>>;
    async fn get_refr_token_by_id(&self, token: i32) -> Result<Option<RefrTokenModel>>;
    async fn create_refr_token<'a>(&self, user_id: i32, token: &'a str) -> Result<i32>;
    async fn revoke_refr_token<'a>(&self, id: i32, repl_id: Option<i32>, revoked_by: &'a str) -> Result<()>;

    async fn create_user<'a>(&self, email: &'a str, pwd_hash: &'a str, card_idx: i32) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbAuthDataLayer {
    db: Arc<PrismaClient>,
    settings: TokenSettings
}

#[async_trait]
impl AuthDataLayer for DbAuthDataLayer {
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<UserModel>> {
        let user = self.db.user().find_first(vec![user::id::equals(user_id)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(user.and_then(|user| Some(UserModel { id: user.id, email: user.email, pwd_hash: user.pwd_hash })))
    }
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserModel>> {
        let user = self.db.user().find_first(vec![user::email::equals(email.to_string())])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(user.and_then(|user| Some(UserModel { id: user.id, email: user.email, pwd_hash: user.pwd_hash })))
    }
    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenModel>> {
        let refr_token = self.db.refresh_token().find_first(vec![refresh_token::token::equals(token.to_string())])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(refr_token.and_then(|tkn| Some(RefrTokenModel { 
            id: tkn.id, user_id: tkn.user_id, token: tkn.token, 
            repl_id: tkn.replacement_id, revoked_on: tkn.revoked_on 
        })))
    }
    async fn get_refr_token_by_id(&self, id: i32) -> Result<Option<RefrTokenModel>> {
        let refr_token = self.db.refresh_token().find_first(vec![refresh_token::id::equals(id)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(refr_token.and_then(|tkn| Some(RefrTokenModel { 
            id: tkn.id, user_id: tkn.user_id, token: tkn.token, 
            repl_id: tkn.replacement_id, revoked_on: tkn.revoked_on 
        })))
    }
    async fn create_refr_token<'a>(&self, user_id: i32, token: &'a str) -> Result<i32> {
        let now = Utc::now();
        let expires = now + Duration::seconds(self.settings.refr_token_lifetime_s);

        let refr_token = self.db.refresh_token().create(
            token.to_string(), 
            user::id::equals(user_id), 
            vec![refresh_token::expires::set(Some(expires.fixed_offset()))]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(refr_token.id)
    }
    async fn revoke_refr_token<'a>(&self, id: i32, repl_id: Option<i32>, revoked_by: &'a str) -> Result<()> {
        let now = Utc::now().fixed_offset();
        self.db.refresh_token().update(
            refresh_token::UniqueWhereParam::IdEquals(id),
            vec![
                refresh_token::revoked_on::set(Some(now)), 
                refresh_token::revoked_by::set(Some(revoked_by.to_string())), 
                refresh_token::replacement_id::set(repl_id)
            ]
        )
            .exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }
    async fn create_user<'a>(&self, email: &'a str, pwd_hash: &'a str, card_idx: i32) -> Result<()> {
        self.db.user().create(email.to_string(), pwd_hash.to_string(), card_idx, vec![]).exec().await.map_err(|e| Box::new(e))?;
        Ok(())
    }
}