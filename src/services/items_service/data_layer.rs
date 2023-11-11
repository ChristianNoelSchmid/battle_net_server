use axum::async_trait;
use crate::services::game_service::models::Stats;

use super::error::Result;

#[async_trait]
pub trait DataLayer : Send + Sync {
    async fn get_user_stats_id(&self, user_id: i32) -> Result<i32>;
    async fn get_user_stats(&self, user_id: i32) -> Result<Stats>;
}

pub struct DbDataLayer {

}