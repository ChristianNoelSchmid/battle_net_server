use std::sync::Arc;

use axum::async_trait;
use chrono::{FixedOffset, DateTime, Utc};

use crate::{data_layer_error::Result, resources::game_resources::BaseStats, prisma::{PrismaClient, user, stats, game_state}};


#[async_trait]
pub trait DataLayer : Send + Sync { 
    ///
    /// Retrieves the last DateTime at which user stats were refreshed.
    /// Returns None if there is no active game
    /// 
    async fn get_last_user_refr(&self) -> Result<Option<DateTime<FixedOffset>>>;
    ///
    /// Resets all users stats to the given `base_stats`
    ///
    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<Vec<i32>>;
}

pub struct DbDataLayer {
    pub db: Arc<PrismaClient>,
}

#[async_trait]
impl DataLayer for DbDataLayer {
    async fn get_last_user_refr(&self) -> Result<Option<DateTime<FixedOffset>>> {
        // Get the active game state
        let game_state = self.db.game_state().find_first(vec![]).exec().await.map_err(|e| Box::new(e))?;

        match game_state {
            // If there is no game active, return None
            None => Ok(None),
            // If there is, return its last_daily_refresh value
            Some(game_state) => Ok(Some(game_state.last_daily_refresh))
        }
        
    }
    async fn reset_user_stats<'a>(&self, base_stats: &'a BaseStats) -> Result<Vec<i32>> {
        // Get all user stats ids
        let stats_ids: Vec<i32> = self.db.user().find_many(vec![]).with(user::state::fetch())
            .exec().await.map_err(|e| Box::new(e))?
            // Unwrap outer Option, as we've called fetch for user state
            .iter().map(|u| u.state.as_ref().unwrap())
            // Filter by Some, then unwrap, collecting the user's stats IDs
            // This will exclude users that don't have stats entries
            .filter(|u| u.is_some()).map(|u| u.as_ref().unwrap().stats_id).collect();

        // Update all found stats
        self.db.stats().update_many(vec![stats::id::in_vec(stats_ids.clone())], vec![
            stats::health::set(base_stats.health),
            stats::armor::set(base_stats.armor),
            stats::missing_next_turn::set(false),
        ])
            .exec().await.map_err(|e| Box::new(e))?;

        self.db.user().update_many(vec![], vec![user::lvl::set(1), user::exhausted::set(false), user::riddle_quest_completed::set(false)])
            .exec().await.map_err(|e| Box::new(e))?;

        // Update the game_state's last refresh time to now
        self.db.game_state().update_many(vec![], vec![game_state::last_daily_refresh::set(Utc::now().fixed_offset())])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(stats_ids)
    }
}