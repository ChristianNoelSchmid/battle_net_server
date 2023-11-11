
use crate::{resources::game_resources::Monster, services::game_service::models::{Stats, CardModel}};

use serde::{self, Serialize};

#[derive(Serialize)]
pub struct QuestStateModel {
    pub lvl: i32,
    pub quest_type: i32,
    pub monster_state: Option<QuestMonsterModel>,
    pub riddle_state: Option<QuestRiddleModel>,
}


#[derive(Serialize)]
pub struct QuestMonsterModel {
    pub monster: Monster,
    pub stats: Stats
}

#[derive(Serialize)]
pub struct QuestRiddleModel {
    pub text: String,
    pub answer_len: i32
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
    Correct(QuestReward),
    Incorrect,
}

#[derive(Serialize)]
pub enum MonsterStatus {
    Alive(Stats),
    Defeated(QuestReward),
}