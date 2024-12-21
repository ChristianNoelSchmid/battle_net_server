use axum::async_trait;
use derive_more::Constructor;
use sqlx::SqlitePool;

use crate::data_layer_error::Result;
use crate::services::game_service::models::Stats;

use super::models::{MonsterState, NextAction};

pub const ATTACK_IDX: i64 = 0;
pub const DEFEND_IDX: i64 = 1;
pub const IDLE_IDX: i64 = 2;

#[async_trait]
pub trait BattleDataLayer : Send + Sync {
    async fn get_monst_state(&self, user_id: i64) -> Result<MonsterState>;
    async fn set_monst_next_action(&self, monst_id: i64, next_action: &NextAction) -> Result<()>;
    async fn get_pl_power(&self, user_id: i64) -> Result<i64>;
    async fn dmg_monst(&self, user_id: i64, pl_power: i64, dmg: i64) -> Result<(i64, bool)>;
    async fn expend_pl_pow(&self, pl_id: i64) -> Result<()>;
    async fn expend_monst_pow(&self, monst_id: i64) -> Result<()>;
    async fn dmg_pl(&self, user_id: i64, dmg: i64) -> Result<()>;
    async fn get_pl_and_monst_stats(&self, user_id: i64) -> Result<(Stats, Stats)>;
    async fn increment_pl_pow(&self, user_id: i64, max_pow: i64) -> Result<()>;
    async fn increment_monst_pow(&self, monst_id: i64, max_pow: i64) -> Result<()>;
}

#[derive(Constructor)]
pub struct DataLayer {
    db: SqlitePool
}

#[async_trait]
impl BattleDataLayer for DataLayer {
    async fn get_monst_state(&self, user_id: i64) -> Result<MonsterState> {
        // Gather the monster state associated with the user's active quest
        let monster = sqlx::query!("
            SELECT ms.id, ms.monster_idx, ms.stats_id, ms.next_action, ms.action_flv_text, s.health, s.power, s.armor, s.missing_next_turn
            FROM quests q JOIN monster_states ms ON q.id = ms.quest_id JOIN stats s ON ms.stats_id = s.id
            WHERE q.user_id = ? AND q.completed = FALSE
            ", user_id
        ).fetch_one(&self.db).await?;

        // Generate the NextAction from the monster state, if it's making a next action
        let next_action = monster.next_action.and_then(|act| {
            Some(NextAction::new(act, monster.power, monster.action_flv_text.unwrap()))
        });

        // Create the monster from the provided info
        let monster = MonsterState::new(
            monster.id,
            monster.monster_idx as usize, 
            Stats::new(monster.health, monster.power, monster.armor, monster.missing_next_turn),
            next_action
        );

        Ok(monster)
    }
    
    async fn get_pl_power(&self, user_id: i64) -> Result<i64> {
        let power = sqlx::query!("
            SELECT s.power FROM user_states us JOIN stats s ON us.stats_id = s.id
            WHERE us.user_id = ?
            ", user_id
        ).fetch_one(&self.db).await?.power;

        Ok(power)
    }

    async fn set_monst_next_action(&self, monst_id: i64, next_action: &NextAction) -> Result<()> {
        sqlx::query!("
            UPDATE monster_states SET next_action = ?, action_flv_text = ? WHERE id = ?
            ", next_action.idx, next_action.flv_text, monst_id
        ).execute(&self.db).await?;
        Ok(())
    }

    async fn expend_pl_pow(&self, pl_id: i64) -> Result<()> {
        let stats_id = sqlx::query!("SELECT stats_id FROM user_states WHERE user_id = ?", pl_id)
            .fetch_one(&self.db).await?.stats_id;
        sqlx::query!("UPDATE stats SET power = 0 WHERE id = ?", stats_id)
            .execute(&self.db).await?;

        Ok(())
    }

    async fn expend_monst_pow(&self, monst_id: i64) -> Result<()> {
        let stats_id = sqlx::query!("SELECT stats_id FROM monster_states WHERE id = ?", monst_id)
            .fetch_one(&self.db).await?.stats_id;
        sqlx::query!("UPDATE stats SET power = 0 WHERE id = ?", stats_id)
            .execute(&self.db).await?;

        Ok(())
    }

    async fn dmg_pl(&self, user_id: i64, dmg: i64) -> Result<()> {
        let stats_id = sqlx::query!("SELECT stats_id FROM user_states WHERE user_id = ?", user_id)
            .fetch_one(&self.db).await?.stats_id;
        sqlx::query!("UPDATE stats SET health = health - ? WHERE id = ?", dmg, stats_id)
            .execute(&self.db).await?;

        Ok(())
    }

    async fn dmg_monst(&self, user_id: i64, pl_power: i64, dmg: i64) -> Result<(i64, bool)> {
        // Get the monster health from the quest the user is currently on
        let quest_monster = sqlx::query!("
            SELECT ms.stats_id, s.health, ms.next_action FROM quests q JOIN monster_states ms ON q.id = ms.quest_id JOIN stats s ON ms.stats_id = s.id
            WHERE q.user_id = ? AND q.completed = FALSE
            ", user_id
        ).fetch_one(&self.db).await?;

        // Get the monster's stats, and divide damage by 2 if it is defending
        let dmg = if quest_monster.next_action.unwrap() == DEFEND_IDX { (dmg as f32 / 2.0) as i64 } else { dmg };

        // If health drops to 0, monster is defeated - return true
        if dmg >= quest_monster.health { 
            return Ok((dmg, true));
        }

        // Update monster health and player power
        sqlx::query!("UPDATE stats SET health = health - ? WHERE id = ?", dmg, quest_monster.stats_id)
            .execute(&self.db).await?;

        let stats_id = sqlx::query!(
            "SELECT stats_id FROM user_states WHERE user_id = ?", user_id
        ).fetch_one(&self.db).await?.stats_id;

        sqlx::query!(
            "UPDATE stats SET power = power - ? WHERE id = ?", pl_power, stats_id
        ).execute(&self.db).await?;

        Ok((dmg, false))
    }

    async fn get_pl_and_monst_stats(&self, user_id: i64) -> Result<(Stats, Stats)> {
        // Get the monster stats and user stats
        let monst_stats = sqlx::query!("
            SELECT s.health, s.power, s.armor, s.missing_next_turn
            FROM quests q JOIN monster_states ms ON q.id = ms.quest_id JOIN stats s ON ms.stats_id = s.id
            WHERE q.user_id = ? AND q.completed = FALSE
            ", user_id
        ).fetch_one(&self.db).await?;
  
        let pl_stats = sqlx::query!("
            SELECT s.health, s.power, s.armor, s.missing_next_turn
            FROM user_states us JOIN stats s ON us.stats_id = s.id
            WHERE us.user_id = ?
            ", user_id
        ).fetch_one(&self.db).await?;
        
        Ok((
            Stats::new(pl_stats.health, pl_stats.power, pl_stats.armor, false),
            Stats::new(monst_stats.health, monst_stats.power, monst_stats.armor, false)
        ))
    }

    async fn increment_pl_pow(&self, user_id: i64, max_pow: i64) -> Result<()> {
        let stats = sqlx::query!("
            SELECT s.id, s.power FROM stats s JOIN user_states us ON us.stats_id = s.id WHERE us.user_id = ?
            ", user_id
        ).fetch_one(&self.db).await?;

        if stats.power < max_pow {
            sqlx::query!("UPDATE stats SET power = power + 1 WHERE id = ?", stats.id)
                .execute(&self.db).await?;
        }

        Ok(())
    }

    async fn increment_monst_pow(&self, monst_id: i64, max_pow: i64) -> Result<()> {
        let stats_id = sqlx::query!(
            "SELECT stats_id FROM monster_states WHERE id = ?", monst_id
        ).fetch_one(&self.db).await?.stats_id;

        let pow = sqlx::query!(
            "SELECT power FROM stats WHERE id = ?", stats_id
        ).fetch_one(&self.db).await?.power;

        if pow < max_pow {
            sqlx::query!("UPDATE stats SET power = power + 1 WHERE id = ?", stats_id)
                .execute(&self.db).await?;
        }

        Ok(())
    }
}