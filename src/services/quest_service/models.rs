
use crate::services::game_service::models::{Stats, CardModel};

use serde::{self, Serialize};

#[derive(Serialize)]
pub struct QuestStateModel {
    pub quest_type: i64,
    pub monster_state: Option<QuestMonsterModel>,
    pub riddle_state: Option<QuestRiddleModel>,
}


#[derive(Serialize)]
pub struct QuestMonsterModel {
    pub res_idx: i64,
    pub stats: Stats,
}

#[derive(Serialize)]
pub struct QuestRiddleModel {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct QuestReward {
    pub item_idxs: Vec<i64>,
    pub card: Option<CardModel>,
}

#[derive(Debug, Serialize)]
pub struct QuestConsequences {
    pub sab_idxs: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub enum RiddleStatus {
    #[serde(rename="correct")]
    Correct(QuestReward),
    #[serde(rename="incorrect")]
    Incorrect,
}

#[derive(Serialize)]
pub enum MonsterStatus {
    Alive(Stats),
    Defeated(QuestReward),
}