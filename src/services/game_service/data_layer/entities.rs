use serde::Serialize;

pub struct MurderedUserModel {
    pub id: i64,
    pub card_idx: i64
}

#[derive(Debug, Serialize)]
pub struct CardModel {
    pub cat_idx: i64,
    pub card_idx: i64
}
pub struct GameStateModel {
    pub murdered_user_idx: i64,
    pub target_cards: Option<Vec<CardModel>>,
    pub winner_idxs: Option<Vec<i64>>
}