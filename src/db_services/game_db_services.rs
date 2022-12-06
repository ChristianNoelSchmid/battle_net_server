use sqlite::{Connection, Value};

use crate::{
    auth::AuthUser,
    execute,
    query, models::{game::{GameInitialState, GameState, EvidenceCard}, users::User, model::Model},
};

pub const GAME_STATE_ID: i64 = 1;

pub fn setup_game(db: &Connection) -> Result<GameInitialState, String> {
    // Ensure there is no currently running game
    let mut game_states = query!(db, "SELECT murdered_user_id FROM game_state LIMIT 1");
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
        let mut target_card_ids = Vec::new();

        // For each, category, select one card as the target card
        for id in cat_ids {
            let mut card_id = query!(
                db,
                r"SELECT id FROM evidence_cards WHERE cat_id = ? AND id != ?
                  ORDER BY RANDOM() 
                  LIMIT 1",
                Value::Integer(id),
                Value::Integer(murdered_user.card_id)
            )
            .map(|row| row.get::<i64, &str>("id"));

            if let Some(card_id) = card_id.next() {
                target_card_ids.push(card_id);
                execute!(
                    db, 
                    "INSERT INTO game_target_cards (card_id) VALUES (?)",
                    Value::Integer(card_id)
                );
            }
        }

        // Create a new game state, with the murdered user
        execute!(
            db,
            r"INSERT INTO game_state (murdered_user_id) VALUES (?)",
            Value::Integer(murdered_user.id)
        );

        for user in query!(db, "SELECT * FROM users").map(|row| User::from_row(row)) {
            // Add the murdered user's corresponding card to everyone's hand
            execute!(
                db,
                r"INSERT INTO user_evidence_cards (user_id, card_id, confirmed)
                  VALUES (?, ?, true)",
                Value::Integer(user.id),
                Value::Integer(murdered_user.card_id)
            );
        }

        Ok(GameInitialState {
            target_card_ids,
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
                r"SELECT * FROM evidence_cards evidence_card
                JOIN game_target_cards target_card ON target_card.card_id = evidence_card.id"
            )
            .map(|row| EvidenceCard::from_row(row))
            .collect::<Vec<EvidenceCard>>(),
        );

        winners = Some(
            query!(
                db,
                r"SELECT * from winners winner JOIN users user ON winner.user_id = user.id
                ORDER BY winner.id",
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

    let cards = query!(db, r"SELECT * FROM evidence_cards")
        .map(|row| EvidenceCard::from_row(row))
        .collect::<Vec<EvidenceCard>>();

    let mut murdered_user = query!(
        db,
        r"SELECT * FROM users
          JOIN game_state ON murdered_user_id = users.id"
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

pub fn guess_target_cards(db: &Connection, guess: &[i64], user: AuthUser) -> bool {
    let target_ids = query!(db, r"SELECT card_id FROM game_target_cards")
        .map(|row| row.get("card_id"))
        .collect::<Vec<i64>>();

    if guess.len() != target_ids.len() {
        return false;
    }
    for guess_id in guess {
        if !target_ids.contains(&guess_id) {
            return false;
        }
    }
    execute!(
        db,
        r"INSERT INTO winners (user_id) VALUES (?)",
        Value::Integer(user.0)
    );
    true
}