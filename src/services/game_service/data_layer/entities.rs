use serde::{Serialize, Deserialize};

pub struct MurderedUserEntity {
    pub id: i64,
    pub card_idx: i64
}
#[derive(Debug, Serialize)]
pub struct CardEntity {
    pub cat_idx: i64,
    pub card_idx: i64
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserCardEntity {
    pub cat_idx: i64,
    pub card_idx: i64,
    pub confirmed: i64
}
pub struct GameStateEntity {
    pub murdered_user_idx: i64,
    pub user_cards: Vec<UserCardEntity>,
    pub target_cards: Option<Vec<CardEntity>>,
    pub winner_idxs: Option<Vec<i64>>
}