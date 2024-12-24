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
    async fn reset_user_stats(&self, base_stats: &BaseStats) -> Result<()>;
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
    async fn reset_user_stats(&self, base_stats: &BaseStats) -> Result<()> {
        // Reset all stats to their base level
        sqlx::query!(
            "UPDATE stats SET health = ?, armor = ?, missing_next_turn = FALSE",
            base_stats.health, base_stats.armor
        )
            .execute(&self.db).await?;

        // Set all player levels to 1
        sqlx::query!("UPDATE users SET lvl = 1, exhausted = FALSE, riddle_quest_completed = FALSE, guessed_today = FALSE")
            .execute(&self.db).await?;

        // Complete all uncompleted quests
        sqlx::query!("UPDATE quests SET completed = TRUE WHERE completed = FALSE")
            .execute(&self.db).await?;

        // Update the game_state's last refresh time to now
        let utc_now = Utc::now().naive_utc();
        sqlx::query!("UPDATE game_states SET last_daily_refresh = ?", utc_now)
            .execute(&self.db).await?;

        Ok(())
    }
}