use axum::async_trait;
use chrono::{Duration, Utc};
use derive_more::Constructor;
use sqlx::SqlitePool;

use crate::{data_layer_error::Result, services::token_service::settings::TokenSettings};

use self::entities::{UserDbModel, RefrTokenDbModel};

pub mod entities;

#[async_trait]
pub trait AuthDataLayer : Send + Sync {
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserDbModel>>;
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserDbModel>>;

    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenDbModel>>;
    async fn get_refr_token_by_id(&self, token: i64) -> Result<Option<RefrTokenDbModel>>;
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
    async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserDbModel>> {
        let user = sqlx::query_as!(UserDbModel, r"
            SELECT id, email, pwd_hash FROM users
            WHERE id = ?
        ", user_id)
            .fetch_one(&self.db).await;

        match user {
            Ok(user) => Ok(Some(user)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        }
    }
    async fn get_user_by_email<'a>(&self, email: &'a str) -> Result<Option<UserDbModel>> {
        let user = sqlx::query_as!(UserDbModel, r"
            SELECT id, email, pwd_hash FROM users
            WHERE email = ?
        ", email)
            .fetch_one(&self.db).await;
        
        match user {
            Ok(user) => Ok(Some(user)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        }
    }

    async fn get_refr_token_by_token<'a>(&self, token: &'a str) -> Result<Option<RefrTokenDbModel>> {
        let refr_token = sqlx::query_as!(RefrTokenDbModel, r"
            SELECT id, user_id, token, replacement_id as repl_id, revoked_on FROM refresh_tokens
            WHERE token = ?
        ", token)
            .fetch_one(&self.db).await;

        match refr_token {
            Ok(refr_token) => Ok(Some(refr_token)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        }
    }
    async fn get_refr_token_by_id(&self, id: i64) -> Result<Option<RefrTokenDbModel>> {
        let refr_token = sqlx::query_as!(RefrTokenDbModel, r"
            SELECT id, user_id, token, replacement_id as repl_id, revoked_on FROM refresh_tokens
            WHERE id = ?
        ", id)
            .fetch_one(&self.db).await;

        match refr_token {
            Ok(refr_token) => Ok(Some(refr_token)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(Box::new(e))
        }
    }
    async fn create_refr_token<'a>(&self, user_id: i64, token: &'a str) -> Result<i64> {
        let now = Utc::now().naive_local();
        let expires = now + Duration::seconds(self.settings.refr_token_lifetime_s);
        let result = sqlx::query!(r"
                INSERT INTO refresh_tokens (user_id, created_on, expires, token)
                VALUES (?, ?, ?, ?)
            ", 
            user_id, now, expires, token
        ).execute(&self.db).await.map_err(|e| Box::new(e))?;

        Ok(result.last_insert_rowid())
    }
    async fn revoke_refr_token<'a>(&self, id: i64, repl_id: Option<i64>, revoked_by: &'a str) -> Result<()> {
        let now = Utc::now().naive_local();
        sqlx::query!(r"
                UPDATE refresh_tokens SET revoked_on = ?, revoked_by = ?, replacement_id = ?
                WHERE id = ?
            ", now, revoked_by, repl_id, id
        )
            .execute(&self.db).await
            .map_err(|e| Box::new(e))?;

        Ok(())
    }
    async fn create_user<'a>(&self, email: &'a str, pwd_hash: &'a str, card_idx: i64) -> Result<()> {
        sqlx::query!("INSERT INTO users (email, pwd_hash, card_idx) VALUES (?, ?, ?)", email, pwd_hash, card_idx)
            .execute(&self.db).await.map_err(|e| Box::new(e))?;

        Ok(())
    }
}