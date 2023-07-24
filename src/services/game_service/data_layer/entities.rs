use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize)]
pub struct CardEntity {
    pub cat_idx: i32,
    pub card_idx: i32
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserCardEntity {
    pub cat_idx: i64,
    pub card_idx: i64,
    pub confirmed: i64
}
