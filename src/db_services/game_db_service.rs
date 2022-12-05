use sqlite::{Connection, Value};

use crate::{
    auth::AuthUser,
    execute,
    models::{EvidenceCard, GameInitialState, GameState, Model, User},
    query,
};

pub const GAME_STATE_ID: i64 = 1;

pub fn all_evidence_cards(db: &Connection) -> Vec<EvidenceCard> {
    let card_rows = query!(
        db,
        r"SELECT cat.cat_name, card.id, card.item_name, card.item_img_path
          FROM cards card JOIN categories cat ON cat.id = card.cat_id"
    );

    card_rows.map(|row| EvidenceCard::from_row(row)).collect()
}

pub fn setup_game(db: &Connection) -> Result<GameInitialState, String> {
    // Ensure there is no currently running game
    let mut game_states = query!(db, "SELECT id FROM game_states LIMIT 1");
    if let Some(_) = game_states.next() {
        return Result::Err(String::from("Game already running"));
    }

    // Create a murdered user
    let mut murdered_user =
        query!(db, r"SELECT * FROM users ORDER BY RANDOM() LIMIT 1").map(|row| User::from_row(row));

    // Ensure there are users in the collection currently
    if let Some(murdered_user) = murdered_user.next() {
        // Get all categories
        let cat_ids = query!(db, "SELECT id FROM categories")
            .map(|row| row.get("id"))
            .collect::<Vec<i64>>();
        let mut target_cards = Vec::new();

        // For each, category, select one card as the target card
        for id in cat_ids {
            let mut card = query!(
                db,
                r"SELECT id, item_name FROM cards WHERE cat_id = ? AND id != ?
                  ORDER BY RANDOM() 
                  LIMIT 1",
                Value::Integer(id),
                Value::Integer(murdered_user.card_id)
            )
            .map(|row| (row.get("id"), row.get("item_name")));

            if let Some(card) = card.next() {
                target_cards.push(card);
            }
        }

        // Create a new game state, with the murdered user
        execute!(
            db,
            r"INSERT INTO game_states murdered_user_id) VALUES (?)",
            Value::Integer(murdered_user.id)
        );

        for user in query!(db, "SELECT * FROM users").map(|row| User::from_row(row)) {
            // Add the murdered user's corresponding card to everyone's hand
            execute!(
                db,
                r"INSERT INTO user_found_cards (user_id, card_id, confirmed)
                  VALUES (?, ?, true)",
                Value::Integer(user.id),
                Value::Integer(murdered_user.card_id)
            );
        }

        for card in target_cards.clone() {
            execute!(
                db,
                r"INSERT INTO game_target_cards (game_state_id, card_id) VALUES (?, ?)",
                Value::Integer(GAME_STATE_ID),
                Value::Integer(card.0)
            );
        }

        Ok(GameInitialState {
            target_cards,
            murdered_user,
        })
    } else {
        Err(String::from(
            "Could not setup game. Ensure there are users in the database",
        ))
    }
}

pub fn game_state(db: &Connection, user: AuthUser) -> Result<GameState, String> {
    let mut winner = query!(
        db,
        r"SELECT * FROM winners WHERE user_id = ?",
        Value::Integer(user.0)
    );
    let target_cards;
    let winners;
    if let Some(_) = winner.next() {
        target_cards = Some(
            query!(
                db,
                r"SELECT * FROM cards card
                JOIN game_target_cards target_card ON target_card.card_id = card.id
                WHERE target_card.game_state_id = ?",
                Value::Integer(GAME_STATE_ID)
            )
            .map(|row| EvidenceCard::from_row(row))
            .collect::<Vec<EvidenceCard>>(),
        );

        winners = Some(
            query!(
                db,
                r"SELECT * from winners winner JOIN users user ON winner.user_id = user.id
                WHERE winner.game_state_id = ? ORDER BY winner.id",
                Value::Integer(GAME_STATE_ID)
            )
            .map(|row| User::from_row(row))
            .collect::<Vec<User>>(),
        );
    } else {
        target_cards = None;
        winners = None;
    };
    let cats = query!(db, r"SELECT id, cat_name FROM categories")
        .map(|row| (row.get("id"), row.get("cat_name")));

    let cards = query!(db, r"SELECT * FROM cards card")
        .map(|row| EvidenceCard::from_row(row))
        .collect::<Vec<EvidenceCard>>();

    let mut murdered_user = query!(
        db,
        r"SELECT * FROM game_states game_state
          JOIN users user ON game_state.murdered_user_id = user.id
          WHERE game_state.id = ?",
        Value::Integer(GAME_STATE_ID)
    )
    .map(|row| User::from_row(row));

    return if let Some(murdered_user) = murdered_user.next() {
        Ok(GameState {
            murdered_user: murdered_user,
            categories: cats.collect(),
            cards,
            target_cards,
            winners,
        })
    } else {
        Err(String::from("Game has not yet started."))
    };
}
