use axum::extract::ws::Message;
use derive_more::Constructor;
use rand::{thread_rng, RngCore};
use serde::Serialize;

use crate::{services::{quest_service::models::{QuestReward, QuestConsequences}, game_service::models::Stats}, resources::game_resources::Monster};


#[derive(Serialize)]
pub enum BattleAction {
    Attack,
    Defend,
    Idle
}

impl<'a> BattleAction {
    pub fn get_action_flv_txt(&self, monst_stats: &Stats, monst_res: &'a Monster) -> &'a str {
        return match self {
            BattleAction::Attack => monst_res.attack_flv_texts[(monst_stats.power - 1) as usize].as_str(),
            BattleAction::Defend => {
                let idx = thread_rng().next_u32() as usize % monst_res.defend_flv_texts.len();
                monst_res.defend_flv_texts[idx].as_str()
            }
            BattleAction::Idle => {
                let idx = thread_rng().next_u32() as usize % monst_res.idle_flv_texts.len();
                monst_res.idle_flv_texts[idx].as_str()
            }
        };
    }
}

#[derive(Constructor)]
pub struct MonsterState {
    ///
    /// The id of the monster in the database
    /// 
    pub db_id: i32,
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
    pub next_act: i32
}

#[derive(Constructor, Serialize)]
pub struct BattleState {
    ///
    /// The player's current Stats
    /// 
    pl_stats: Stats,
    ///
    /// The monster's current Stats
    /// 
    monst_stats: Stats
}

#[derive(Constructor, Serialize)]
pub struct MonsterNextAction {
    ///
    /// The action the monster just completed
    /// 
    action: BattleAction,
    ///
    /// The flavor-text of the monster's next action
    /// 
    next_action_txt: String,
}

#[derive(Serialize)]
pub enum RoundResult {
    ///
    /// Signals that the user won the battle,
    /// providing quest rewards
    /// 
    Victory(QuestReward),
    ///
    /// Signals that the user was defeated this round,
    /// providing the monster's damage dealt and sabatogues from losing
    /// 
    Defeat { monst_dmg: i32, consq: QuestConsequences },
    ///
    /// Signals that the round did not complete the battle,
    /// providing all relevant info for the end of round, and next round
    /// 
    Next(BattleState, MonsterNextAction)
}

impl RoundResult {
    pub fn to_ws_msg(&self) -> Message {
        Message::Text(serde_json::to_string(&self).unwrap())
    }
    pub fn battle_completed(&self) -> bool {
        match self {
            RoundResult::Victory(_) => true,
            RoundResult::Defeat { monst_dmg: _, consq: _ } => true,
            RoundResult::Next(_,_) => false
        }
    }
}