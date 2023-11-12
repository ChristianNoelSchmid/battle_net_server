use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;

use crate::prisma::{PrismaClient, stats, quest_monster, quest, user_state, user};
use crate::data_layer_error::Result;
use crate::services::game_service::models::Stats;

use super::models::{MonsterState, NextAction};

pub const ATTACK_IDX: i32 = 0;
pub const DEFEND_IDX: i32 = 1;
pub const IDLE_IDX: i32 = 2;

#[async_trait]
pub trait BattleDataLayer : Send + Sync {
    async fn get_monst_state(&self, user_id: i32) -> Result<MonsterState>;
    async fn set_monst_next_action(&self, monst_id: i32, next_action: &NextAction) -> Result<()>;
    async fn get_pl_power(&self, user_id: i32) -> Result<i32>;
    async fn dmg_monst(&self, user_id: i32, pl_power: i32, dmg: i32) -> Result<(i32, bool)>;
    async fn expend_pl_pow(&self, pl_id: i32) -> Result<()>;
    async fn expend_monst_pow(&self, monst_id: i32) -> Result<()>;
    async fn dmg_pl(&self, user_id: i32, dmg: i32) -> Result<()>;
    async fn get_pl_and_monst_stats(&self, user_id: i32) -> Result<(Stats, Stats)>;
    async fn increment_pl_pow(&self, user_id: i32, max_pow: i32) -> Result<()>;
    async fn increment_monst_pow(&self, monst_id: i32, max_pow: i32) -> Result<()>;
}

#[derive(Constructor)]
pub struct DbBattleDataLayer {
    db: Arc<PrismaClient>
}

#[async_trait]
impl BattleDataLayer for DbBattleDataLayer {
    async fn get_monst_state(&self, user_id: i32) -> Result<MonsterState> {
        let monster = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .with(quest::monster::fetch().with(quest_monster::stats::fetch()))
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .monster.unwrap().unwrap();

        let stats = monster.stats.unwrap();
        let next_action = if monster.next_action.is_none() { 
            None 
        } else { 
            Some(NextAction::new(monster.next_action.unwrap(), monster.action_flv_text.unwrap() ))
        };

        let monster = MonsterState::new(
            monster.id,
            monster.monster_idx as usize, 
            Stats::new(stats.health, stats.power, stats.armor, stats.missing_next_turn),
            next_action
        );

        Ok(monster)
    }
    
    async fn get_pl_power(&self, user_id: i32) -> Result<i32> {
        let user_stats = self.db.user_state().find_unique(user_state::UniqueWhereParam::UserIdEquals(user_id))
            .with(user_state::stats::fetch())
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .stats.unwrap();

        Ok(user_stats.power)
    }

    async fn set_monst_next_action(&self, monst_id: i32, next_action: &NextAction) -> Result<()> {
        self.db.quest_monster().update(
            quest_monster::UniqueWhereParam::IdEquals(monst_id),
            vec![
                quest_monster::next_action::set(Some(next_action.idx)), 
                quest_monster::action_flv_text::set(Some(next_action.flv_text.clone()))
            ]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn expend_pl_pow(&self, pl_id: i32) -> Result<()> {
        let stats_id = self.db.user_state().find_unique(user_state::UniqueWhereParam::UserIdEquals(pl_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().stats_id;

        self.db.stats().update(stats::UniqueWhereParam::IdEquals(stats_id), vec![stats::power::set(0)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn expend_monst_pow(&self, monst_id: i32) -> Result<()> {
        let stats_id = self.db.quest_monster().find_unique(quest_monster::UniqueWhereParam::IdEquals(monst_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().stats_id;

        self.db.stats().update(stats::UniqueWhereParam::IdEquals(stats_id), vec![stats::power::set(0)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn dmg_pl(&self, user_id: i32, dmg: i32) -> Result<()> {
        let stats_id = self.db.user().find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .with(user::state::fetch())
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .state.unwrap().unwrap().stats_id;

        self.db.stats().update(stats::UniqueWhereParam::IdEquals(stats_id), vec![stats::health::decrement(dmg)])
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(())
    }

    async fn dmg_monst(&self, user_id: i32, pl_power: i32, dmg: i32) -> Result<(i32, bool)> {
        // Get the monster stats from the quest the user is currently on
        let quest = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .with(quest::monster::fetch().with(quest_monster::stats::fetch()))
            .exec().await.map_err(|e| Box::new(e))?.unwrap();
 
        // Get the monster's stats, and divide damage by 2 if it is defending
        let monster = quest.monster.unwrap().unwrap();
        let sts = monster.stats.unwrap();
        let dmg = if monster.next_action.unwrap() == DEFEND_IDX { (dmg as f32 / 2.0) as i32 } else { dmg };

        // If health drops to 0, monster is defeated - return true
        if dmg >= sts.health { 
            return Ok((dmg, true));
        }

        // Update monster health and player power
        self.db.stats().update(
            stats::UniqueWhereParam::IdEquals(sts.id), 
            vec![stats::health::decrement(dmg)]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        let pl_stats = self.db.user_state().find_unique(user_state::UniqueWhereParam::UserIdEquals(user_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().stats_id;

        self.db.stats().update(
            stats::UniqueWhereParam::IdEquals(pl_stats), 
            vec![stats::power::decrement(pl_power)]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        Ok((dmg, false))
    }

    async fn get_pl_and_monst_stats(&self, user_id: i32) -> Result<(Stats, Stats)> {
        // Get the monster stats and user stats
        let monst_stats = self.db.quest().find_first(vec![quest::user_id::equals(user_id), quest::completed::equals(false)])
            .with(quest::monster::fetch().with(quest_monster::stats::fetch()))
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .monster.unwrap().unwrap().stats.unwrap();

        let pl_stats = self.db.user_state().find_unique(user_state::UniqueWhereParam::UserIdEquals(user_id))
            .with(user_state::stats::fetch())
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .stats.unwrap();

        Ok((
            Stats::new(pl_stats.health, pl_stats.power, pl_stats.armor, false),
            Stats::new(monst_stats.health, monst_stats.power, monst_stats.armor, false)
        ))
    }

    async fn increment_pl_pow(&self, user_id: i32, max_pow: i32) -> Result<()> {
        let stats_id = self.db.user_state().find_unique(user_state::UniqueWhereParam::UserIdEquals(user_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().stats_id;

        let pow = self.db.stats().find_unique(stats::UniqueWhereParam::IdEquals(stats_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().power;

        if pow < max_pow {
            self.db.stats().update(stats::UniqueWhereParam::IdEquals(stats_id), vec![stats::power::increment(1)])
                .exec().await.map_err(|e| Box::new(e))?;
        }

        Ok(())
    }

    async fn increment_monst_pow(&self, monst_id: i32, max_pow: i32) -> Result<()> {
        let stats_id = self.db.quest_monster().find_unique(quest_monster::UniqueWhereParam::IdEquals(monst_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().stats_id;

        let pow = self.db.stats().find_unique(stats::UniqueWhereParam::IdEquals(stats_id))
            .exec().await.map_err(|e| Box::new(e))?.unwrap().power;

        if pow < max_pow {
            self.db.stats().update(stats::UniqueWhereParam::IdEquals(stats_id), vec![stats::power::increment(1)])
                .exec().await.map_err(|e| Box::new(e))?;
        }

        Ok(())
    }
}