use crate::services::game_service::data_layer::entities::CardEntity;

use super::game_models::Stats;
use chrono::{DateTime, FixedOffset};
use serde::{self, Serialize};


#[derive(Serialize)]
pub enum RiddleStatus {
    Correct(QuestReward),
    Incorrect,
}

#[derive(Serialize)]
pub enum MonsterStatus {
    Alive(Stats),
    Defeated(Option<QuestReward>),
}

#[derive(Serialize)]
pub struct QuestReward {
    pub item_idxs: Vec<i64>,
    pub card: Option<CardEntity>,
}


#[derive(Serialize)]
pub struct QuestEvent<'a> {
    pub monster_event: Option<QuestEventMonster>,
    pub riddle_event: Option<RiddleModel<'a>>,
}

#[derive(Serialize)]
pub struct QuestEventMonster {
    pub monster_idx: i64,
    pub stats: Stats,
}

#[derive(Serialize)]
pub struct RiddleModel<'a> {
    pub idx: i32,
    pub text: &'a str,
}

pub struct QuestModel {
    pub id: i32,
    pub created_on: DateTime<FixedOffset>,
    pub user_id: i32,
    pub lvl: i32,
    pub quest_type: i32,
    pub completed: bool
}