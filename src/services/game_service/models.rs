use crate::resources::game_resources::BaseStats;
use derive_more::Constructor;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize)]
pub struct CardModel {
    pub cat_idx: i64,
    pub card_idx: i64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCardModel {
    pub cat_idx: i64,
    pub card_idx: i64,
    pub confirmed: bool
}

///
/// Initial state of a game, directly after setup
/// 
#[derive(Serialize)]
pub struct GameInitialStateModel {
    pub target_cards: Vec<CardModel>,
    pub murdered_user_card_idx: i64,
}

#[derive(Serialize)]
pub struct GameStateModel {
    pub murdered_user_id: i64,
    pub user_cards: Vec<UserCardModel>,
    pub target_cards: Option<Vec<CardModel>>,
    pub winner_idxs: Option<Vec<i64>>,
    pub pl_exhausted: bool,
    pub pl_completed_riddle: bool
}

#[derive(Serialize, Clone, Constructor, Copy)]
pub struct Stats {
    pub health: i64,
    pub power: i64,
    pub armor: i64,
    pub miss_turn: bool,
}

pub struct MurderedUserModel {
    pub card_idx: i64
}

impl Stats {
    pub fn from_base_stats(b_stats: BaseStats) -> Self {
        Self::new(b_stats.health, 1, b_stats.armor, false)
    }
}