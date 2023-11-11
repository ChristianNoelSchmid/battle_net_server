use std::sync::Arc;

use axum::async_trait;
use derive_more::Constructor;
use rand::{thread_rng, RngCore};

use crate::ai::AI;
use crate::resources::game_resources::{Resources, Monster};

use self::data_layer::{BattleDataLayer, ATTACK_IDX};
use self::error::{Result, BattleServiceError};
use self::models::{MonsterNextAction, RoundResult, BattleState};

use super::game_service::models::Stats;
use super::quest_service::QuestService;

pub mod data_layer;
pub mod error;
pub mod models;

#[async_trait]
pub trait BattleService : Send + Sync {
    async fn attack(&self, user_id: i32, power: i32) -> Result<RoundResult>;
    async fn defend(&self, user_id: i32) -> Result<RoundResult>;
    async fn use_item(&self, user_id: i32, item_idx: i32) -> Result<RoundResult>;
}

#[derive(Constructor)]
pub struct CoreBattleService {
    data_layer: Arc<dyn BattleDataLayer>,
    quest_service: Arc<dyn QuestService>,
    res: Arc<Resources>,
}

#[async_trait]
impl BattleService for CoreBattleService {
    async fn attack(&self, user_id: i32, power: i32) -> Result<RoundResult> { 
        // Damage the monster, and test if it's been defeated
        return if self.data_layer.dmg_monst(user_id, power).await.map_err(|e| e.into())? {
            // If it was defeated, complete the quest and return the victory signal, with rewards
            let reward = self.quest_service.complete_quest(user_id).await.map_err(|e| BattleServiceError::QuestServiceError(e))?;
            Ok(RoundResult::Victory(reward))
        } else {
            // Otherwise, perform the monster's action, and return the results
            self.perform_monster_action(user_id, false).await.map_err(|e| e.into())
        }
    }

    async fn defend(&self, user_id: i32) -> Result<RoundResult> {
        self.perform_monster_action(user_id, true).await.map_err(|e| e.into())
    }
    async fn use_item(&self, user_id: i32, item_idx: i32) -> Result<RoundResult> { 
        todo!();
    }
}

impl CoreBattleService {
    ///
    /// Returns the amount of damage the monster does this turn, given the info
    /// 
    fn get_monster_dmg(&self, monst_stats: &Stats, monst_res: &Monster, pl_defd: bool) -> i32 {
        let rng = monst_res.pow_dmg[(monst_stats.power - 1) as usize];
        let dmg = (thread_rng().next_u32() % (rng.1 - rng.0) as u32) as i32;

        return if pl_defd { rng.0 + dmg } else { ((rng.0 + dmg) as f32 / 2.0) as i32 };
    }
    
    ///
    /// Performs the monster's action, damaging the player if attacking,
    /// and generating its next action
    /// 
    async fn perform_monster_action(&self, user_id: i32, pl_defd: bool) -> Result<RoundResult> {
        // Get the current state of the Monster, and current player and Monster Stats
        let (pl_stats, monst_stats) = self.data_layer.get_pl_and_monst_stats(user_id).await.map_err(|e| e.into())?;
        let monst_state = self.data_layer.get_monst_state(user_id).await.map_err(|e| e.into())?;
        let monst_res = &self.res.monsters[monst_state.res_idx];

        // Have the monster attack the player, and determine if the player is defeated
        if monst_state.next_act == ATTACK_IDX {
            let monst_dmg = self.get_monster_dmg(&monst_stats, &self.res.monsters[monst_state.res_idx], pl_defd);
            if monst_dmg >= pl_stats.health {
                let consq = self.quest_service.fail_quest(user_id).await.map_err(|e| BattleServiceError::QuestServiceError(e))?;
                return Ok(RoundResult::Defeat { monst_dmg, consq })
            }
            self.data_layer.dmg_pl(user_id, monst_dmg).await.map_err(|e| e.into())?;
        }

        // Determine the monster's next action, and new Stats
        let (pl_stats, monst_stats) = self.data_layer.get_pl_and_monst_stats(user_id).await.map_err(|e| e.into())?;
        let next_battle_action = AI[monst_res.name.as_str()].next_act(&pl_stats, &monst_stats);
        let next_flv_text = next_battle_action.get_action_flv_txt(&monst_stats, &monst_res).to_string();
        let next_monster_action = MonsterNextAction::new(next_battle_action, next_flv_text);

        return Ok(RoundResult::Next(BattleState::new(pl_stats, monst_stats), next_monster_action));
    }
}