use serde::Serialize;

use crate::services::game_service::models::Stats;

#[derive(Serialize)]
pub struct QuestStateEntity {
    pub id: i32,
    pub lvl: i32,
    pub quest_type: i32,
    pub monster_state: Option<QuestMonsterEntity>,
    pub riddle_idx: Option<i32>,
    pub completed: bool
}

#[derive(Serialize)]
pub struct QuestMonsterEntity {
    pub monster_idx: i32,
    pub stats: Stats,
}