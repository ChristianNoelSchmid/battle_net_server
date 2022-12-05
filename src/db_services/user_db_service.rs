use sqlite::{Connection, Value};

use crate::{
    auth::AuthUser,
    execute,
    models::{Model, User},
    query,
};

pub fn all_users(db: &Connection) -> Vec<User> {
    query!(db, "SELECT * from users")
        .map(|row| User::from_row(row))
        .collect()
}

///
/// Returns a list of the user's currently active evidence
/// cards, mapped to whether each is a confirmed card.
///
pub fn user_evidence_cards(db: &Connection, user: AuthUser) -> Vec<(i64, bool)> {
    let cards = query!(
        db,
        r"SELECT card.id, user_card.confirmed FROM evidence_cards card
        JOIN user_evidence_cards user_card ON card.id = user_card.card_id 
        WHERE user_card.user_id = ?",
        Value::Integer(user.0)
    );
    let cards = cards.map(|row| (row.get("id"), row.get::<i64, &str>("confirmed") == 1));
    cards.collect()
}

pub fn update_evidence_card(db: &Connection, user: AuthUser, card_id: i64) {
    let mut card = query!(
        db,
        r"SELECT confirmed FROM user_found_cards WHERE user_id = ? AND card_id = ?",
        Value::Integer(user.0),
        Value::Integer(card_id)
    );
    if let Some(card) = card.next() {
        if card.get::<i64, &str>("confirmed") == 0 {
            execute!(
                db,
                "DELETE FROM user_found_cards WHERE user_id = ? and card_id = ?",
                Value::Integer(user.0),
                Value::Integer(card_id)
            );
        }
    } else {
        execute!(
            db,
            r"INSERT INTO user_found_cards (user_id, card_id, confirmed)
              VALUES (?, ?, false)",
            Value::Integer(user.0),
            Value::Integer(card_id)
        );
    }
}
