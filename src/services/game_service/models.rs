use crate::resources::game_resources::BaseStats;
use derive_more::Constructor;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize)]
pub struct CardModel {
    pub cat_idx: i32,
    pub card_idx: i32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCardModel {
    pub cat_idx: i32,
    pub card_idx: i32,
    pub confirmed: bool
}

///
/// Initial state of a game, directly after setup
/// 
#[derive(Serialize)]
pub struct GameInitialStateModel {
    pub target_cards: Vec<CardModel>,
    pub murdered_user_card_idx: i32,
}

#[derive(Serialize)]
pub struct GameStateModel {
    pub murdered_user_idx: i32,
    pub user_cards: Vec<UserCardModel>,
    pub target_cards: Option<Vec<CardModel>>,
    pub winner_idxs: Option<Vec<i32>>
}

#[derive(Serialize, Clone, Constructor, Copy)]
pub struct Stats {
    pub health: i32,
    pub power: i32,
    pub armor: i32,
    pub miss_turn: bool,
}

pub struct MurderedUserModel {
    pub card_idx: i32
}

impl Stats {
    pub fn from_base_stats(b_stats: BaseStats) -> Self {
        Self::new(b_stats.health, 1, b_stats.armor, false)
    }
}