use axum::async_trait;
use chrono::{Duration, Utc};
use derive_more::Constructor;
use sqlx::SqlitePool;

use crate::{data_layer_error::Result, services::token_service::settings::TokenSettings};

use super::models::{UserModel, RefrTokenModel};

#[async_trait]
pub trait AuthDataLayer : Send + Sync {
    async fn get_all_users(&self) -> Result<Vec<(String, i64)>>;
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserModel>>;
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserModel>>;

    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenModel>>;
    async fn get_refr_token_by_id(&self, token: i64) -> Result<Option<RefrTokenModel>>;
    async fn create_refr_token<'a>(&self, user_id: i64, token: &'a str) -> Result<i64>;
    async fn revoke_refr_token<'a>(&self, id: i64, repl_id: Option<i64>, revoked_by: &'a str) -> Result<()>;

    async fn create_user<'a>(&self, email: &'a str, pwd_hash: &'a str, card_idx: i64) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbAuthDataLayer {
    db: SqlitePool,
    settings: TokenSettings
}

#[async_trait]
impl AuthDataLayer for DbAuthDataLayer {
    async fn get_all_users(&self) -> Result<Vec<(String, i64)>> {
        Ok(sqlx::query!("SELECT id, email FROM users").fetch_all(&self.db).await?.into_iter().map(|u| (u.email, u.id)).collect())
    }
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserModel>> {
        let user = sqlx::query_as!(UserModel, 
            "SELECT id, email, pwd_hash FROM users WHERE id = ?", user_id
        ).fetch_optional(&self.db).await?;

        Ok(user)
    }
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserModel>> {
        let user = sqlx::query_as!(UserModel,
            "SELECT id, email, pwd_hash FROM users WHERE email = ?", email
        ).fetch_optional(&self.db).await?;

        Ok(user)
    }
    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenModel>> {
        let refr_token = sqlx::query_as!(RefrTokenModel, "
            SELECT id, user_id, token, repl_id, revoked_on 
            FROM refresh_tokens WHERE token = ?
            ", token
        ).fetch_optional(&self.db).await?;

        Ok(refr_token)
    }
    async fn get_refr_token_by_id(&self, id: i64) -> Result<Option<RefrTokenModel>> {
        let refr_token = sqlx::query_as!(RefrTokenModel, "
            SELECT id, user_id, token, repl_id, revoked_on
            FROM refresh_tokens WHERE id = ?
            ", id
        ).fetch_optional(&self.db).await?;

        Ok(refr_token)
    }
    async fn create_refr_token<'a>(&self, user_id: i64, token: &'a str) -> Result<i64> {
        let now = Utc::now();
        let expires = now + Duration::seconds(self.settings.refr_token_lifetime_s);

        let res = sqlx::query!("
            INSERT INTO refresh_tokens (user_id, expires, token)
            VALUES (?, ?, ?)
            ", user_id, expires, token
        ).execute(&self.db).await?;

        Ok(res.last_insert_rowid())
    }
    async fn revoke_refr_token<'a>(&self, id: i64, repl_id: Option<i64>, revoked_by: &'a str) -> Result<()> {
        let now = Utc::now().fixed_offset();
        sqlx::query!("
            UPDATE refresh_tokens SET revoked_on = ?, revoked_by = ?, repl_id = ?
            WHERE id = ?
            ", now, revoked_by, repl_id, id
        ).execute(&self.db).await?;
        Ok(())
    }
    async fn create_user<'a>(&self, email: &'a str, pwd_hash: &'a str, card_idx: i64) -> Result<()> {
        sqlx::query!("
            INSERT INTO users (email, pwd_hash, card_idx) VALUES (?, ?, ?)
            ", email, pwd_hash, card_idx
        ).execute(&self.db).await?;
        Ok(())
    }
}