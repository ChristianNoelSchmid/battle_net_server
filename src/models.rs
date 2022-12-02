use serde::{Deserialize, Serialize};
use sqlite::Row;

pub trait Model {
    fn from_row(row: Row) -> Self;
}

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub card_id: i64,
    pub user_name: String,
}
impl Model for User {
    fn from_row(row: Row) -> Self {
        User {
            id: row.get("id"),
            card_id: row.get("card_id"),
            user_name: row.get("user_name")
        }
    }
}

#[derive(Serialize)]
pub struct Card {
    pub card_id: i64,
    pub cat_name: String,
    pub item_name: String,
    pub item_img_path: Option<String>,
}
impl Model for Card {
    fn from_row(row: Row) -> Self {
        Card {
            card_id: row.get("id"),
            cat_name: row.get("cat_name"),
            item_name: row.get("item_name"),
            item_img_path: row.get("item_img_path"),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct UserCard {
    pub card_id: i64,
    pub cat_name: String,
    pub item_name: String,
    pub item_img_path: Option<String>,
    pub confirmed: bool,
}

impl Model for UserCard {
    fn from_row(row: Row) -> Self {
        UserCard {
            card_id: row.get("id"),
            cat_name: row.get("cat_name"),
            item_name: row.get("item_name"),
            item_img_path: row.get("item_img_path"),
            confirmed: row.get::<i64, &str>("confirmed") == 1,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct UserCards {
    pub confirmed: Vec<UserCard>,
    pub unconfirmed: Vec<UserCard>,
}

#[derive(Serialize)]
pub struct GetGameState {
    pub target_cards: Vec<(i64, String)>,
    pub murdered_user: (i64, String),
}

#[derive(Deserialize)]
pub struct PostRiddle {
    pub text: String,
    pub answers: Vec<String>
}
