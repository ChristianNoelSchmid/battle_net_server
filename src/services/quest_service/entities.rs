use serde::Serialize;

use crate::services::game_service::models::Stats;

#[derive(Serialize)]
pub struct QuestStateEntity {
    pub id: i64,
    pub quest_type: i64,
    pub monster_state: Option<QuestMonsterEntity>,
    pub riddle_idx: Option<i64>,
    pub completed: bool
}

#[derive(Serialize)]
pub struct QuestMonsterEntity {
    pub monster_idx: i64,
    pub stats: Stats,
}