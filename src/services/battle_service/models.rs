use axum::extract::ws::Message;
use derive_more::Constructor;

use serde::Serialize;

use crate::services::{quest_service::models::{QuestReward, QuestConsequences}, game_service::models::Stats};

#[derive(Constructor, Serialize)]
pub struct MonsterState {
    ///
    /// The id of the monster in the database
    /// 
    pub db_id: i64,
    ///
    /// The index of the monster in the Resources collection
    /// 
    pub res_idx: usize,
    ///
    /// The monster's stats
    ///
    pub stats: Stats,    
    ///
    /// The monster's next action
    /// 
    pub next_action: Option<NextAction>
}

#[derive(Constructor, Serialize)]
pub struct NextAction {
    ///
    /// The index of the type of action being performed next
    /// 
    pub idx: i64,
    ///
    /// The flavor text of the action being performed
    /// 
    pub flv_text: String
}

#[derive(Serialize)]
pub enum RoundResult {
    ///
    /// Signals that the user won the battle,
    /// providing quest rewards
    /// 
    #[serde(rename="victory")]
    Victory { reward: QuestReward, pl_dmg_dealt: i64 },
    ///
    /// Signals that the user was defeated this round,
    /// providing the monster's damage dealt and sabatogues from losing
    /// 
    #[serde(rename="defeat")]
    Defeat { monst_dmg: i64, consq: QuestConsequences, pl_dmg_dealt: i64 },
    ///
    /// Signals that the round did not complete the battle,
    /// providing all relevant info for the end of round, and next round
    /// 
    #[serde(rename="next")]
    Next { pl_stats: Stats, monst_stats: Stats, next_action: NextAction, pl_dmg_dealt: i64 }
}

impl RoundResult {
    pub fn to_ws_msg(&self) -> Message {
        Message::Text(serde_json::to_string(&self).unwrap())
    }
    pub fn battle_completed(&self) -> bool {
        match self {
            RoundResult::Victory { reward: _, pl_dmg_dealt: _ } => true,
            RoundResult::Defeat { monst_dmg: _, consq: _, pl_dmg_dealt: _ } => true,
            RoundResult::Next { pl_stats: _, monst_stats: _, next_action: _, pl_dmg_dealt: _ } => false
        }
    }
}