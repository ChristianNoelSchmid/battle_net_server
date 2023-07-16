use super::game::Stats;
use serde::{self, Deserialize, Serialize};

#[derive(Serialize)]
pub enum RiddleStatus {
    Correct(Option<QuestReward>),
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
    pub cat_and_card_idxs: Vec<(i64, i64)>,
}

#[derive(Deserialize)]
pub enum EventType {
    Monster,
    Riddle,
}

#[derive(Serialize)]
pub struct QuestEvent<'a> {
    pub monster_event: Option<QuestEventMonster>,
    pub riddle_event: Option<QuestEventRiddle<'a>>,
}

#[derive(Serialize)]
pub struct QuestEventMonster {
    pub monster_idx: i64,
    pub stats: Stats,
}

#[derive(Serialize)]
pub struct QuestEventRiddle<'a> {
    pub idx: i64,
    pub text: &'a String,
}
