use axum::async_trait;
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

use crate::{data_layer_error::Result, resources::game_resources::BaseStats};


#[async_trait]
pub trait DataLayer : Send + Sync { 
    ///
    /// Retrieves the last DateTime at which user stats were refreshed.
    /// Returns None if there is no active game
    /// 
    async fn get_last_user_refr(&self) -> Result<Option<NaiveDateTime>>;
    ///
    /// Resets all users stats to the given `base_stats`
    ///
    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<Vec<i64>>;
}

pub struct DbDataLayer {
    pub db: SqlitePool,
}

#[async_trait]
impl DataLayer for DbDataLayer {
    async fn get_last_user_refr(&self) -> Result<Option<NaiveDateTime>> {
        // Get the active game state
        let last_daily_refresh = sqlx::query!(
            "SELECT last_daily_refresh FROM game_states"
        ).fetch_optional(&self.db).await?
         .and_then(|row| Some(row.last_daily_refresh));

        match last_daily_refresh {
            // If there is no game active, return None
            None => Ok(None),
            // If there is, return its last_daily_refresh value
            Some(last_daily_refresh) => Ok(Some(last_daily_refresh))
        }
        
    }
    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<Vec<i64>> {
        // Get all user stats ids
        let stats_ids = sqlx::query!("
            SELECT s.id FROM stats s
            JOIN user_states us ON s.id = us.stats_id
            WHERE us.stats_id IS NOT NULL
        ")
            .fetch_all(&self.db).await?
            .iter().map(|row| row.id).collect::<Vec<i64>>();

        // Update all found stats
        let stats_fmt = stats_ids.iter().map(|id| format!("{}", id))
            .collect::<Vec<String>>().join(",");

        sqlx::query!("
            UPDATE stats SET health = ?, armor = ?, missing_next_turn = FALSE
            WHERE id IN (?)
            ", base_stats.health, base_stats.armor, stats_fmt
        )
            .execute(&self.db).await?;

        sqlx::query!("UPDATE users SET lvl = 1, exhausted = FALSE, riddle_quest_completed = FALSE")
            .execute(&self.db).await?;

        // Update the game_state's last refresh time to now
        let utc_now = Utc::now().naive_utc();
        sqlx::query!("UPDATE game_states SET last_daily_refresh = ?", utc_now)
            .execute(&self.db).await?;

        Ok(stats_ids)
    }
}