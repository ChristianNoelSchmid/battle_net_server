use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;

use crate::prisma::{PrismaClient, stats, quest_monster, quest, user_state, user};
use crate::data_layer_error::Result;
use crate::services::game_service::models::Stats;

use super::models::MonsterState;

pub const ATTACK_IDX: i32 = 0;
pub const DEFEND_IDX: i32 = 1;
pub const IDLE_IDX: i32 = 2;

#[async_trait]
pub trait BattleDataLayer : Send + Sync {
    async fn get_monst_state(&self, user_id: i32) -> Result<MonsterState>;
    async fn dmg_monst(&self, user_id: i32, dmg: i32) -> Result<bool>;
    async fn expend_monst_pow(&self, monst_id: i32) -> Result<()>;
    async fn dmg_pl(&self, user_id: i32, dmg: i32) -> Result<()>;
    async fn get_pl_and_monst_stats(&self, user_id: i32) -> Result<(Stats, Stats)>;
}

#[derive(Constructor)]
pub struct DbBattleDataLayer {
    db: Arc<PrismaClient>
}

#[async_trait]
impl BattleDataLayer for DbBattleDataLayer {
    async fn get_monst_state(&self, user_id: i32) -> Result<MonsterState> {
        let monster = self.db.quest().find_first(vec![quest::user_id::equals(user_id)])
            .with(quest::monster::fetch())
            .exec().await.map_err(|e| Box::new(e))?.unwrap()
            .monster.unwrap().unwrap();
        let stats = monster.stats.unwrap();

        let monster = MonsterState::new(
            monster.id,
            monster.monster_idx as usize, 
            Stats::new(stats.health, stats.power, stats.armor, stats.missing_next_turn),
            monster.next_action
        );

        Ok(monster)
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

    async fn dmg_monst(&self, user_id: i32, power: i32) -> Result<bool> {
        // Get the monster stats from the quest the user is currently on
        let quest = self.db.quest().find_first(vec![quest::user_id::equals(user_id)])
            .with(quest::monster::fetch().with(quest_monster::stats::fetch()))
            .exec().await.map_err(|e| Box::new(e))?.unwrap();

        // Get the monster's stats, and divide damage by 2 if it is defending
        let monster = quest.monster.unwrap().unwrap();
        let sts = monster.stats.unwrap();
        let dmg = if monster.next_action == DEFEND_IDX { (power as f32 / 2.0) as i32 } else { power };

        // If health drops to 0, monster is defeated - return true
        if dmg >= sts.health { 
            return Ok(true);
        }

        // Update stats to 
        self.db.stats().update(
            stats::UniqueWhereParam::IdEquals(sts.id), 
            vec![stats::health::decrement(dmg)]
        )
            .exec().await.map_err(|e| Box::new(e))?;

        Ok(false)
    }

    async fn get_pl_and_monst_stats(&self, user_id: i32) -> Result<(Stats, Stats)> {
        // Get the monster stats and user stats
        let monst_stats = self.db.quest().find_first(vec![quest::user_id::equals(user_id)])
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
}