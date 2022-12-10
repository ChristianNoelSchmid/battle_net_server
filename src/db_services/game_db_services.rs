use rand::{seq::IteratorRandom, thread_rng};
use rocket::State;
use sqlite::{Connection, Value};

use crate::{
    execute,
    middleware::auth::AuthUser,
    models::game::{GameInitialState, GameState},
    query,
    resources::game_resources::Resources,
};

pub const GAME_STATE_ID: i64 = 1;

pub fn setup_game(db: &Connection, res: &State<Resources>) -> Result<GameInitialState, String> {
    let mut rng = thread_rng();

    // Ensure there is no currently running game
    let mut game_states = query!(db, "SELECT murdered_user_id FROM game_state LIMIT 1");
    if let Some(_) = game_states.next() {
        return Result::Err(String::from("Game already running"));
    }

    // Create a murdered user
    let (mu_idx, mu_card_idx): (i64, i64) = query!(
        db,
        "SELECT id, card_idx FROM users ORDER BY RANDOM() LIMIT 1"
    )
    .map(|row| (row.get("id"), row.get("card_idx")))
    .next()
    .unwrap();

    // For each category, select one card as the target card
    let mut target_card_idxs = Vec::new();
    for cat in res.evd_card_cats {
        let (card_index, _) = cat
            .cards
            .iter()
            .enumerate()
            .choose(&mut rng)
            .expect("Could not find all category cards.");

        execute!(
            db,
            "INSERT INTO game_target_cards (card_id) VALUES (?)",
            Value::Integer(card_index as i64)
        );
        target_card_idxs.push(card_index as i64);

        // Create a new game state, with the murdered user
        execute!(
            db,
            r"INSERT INTO game_state (murdered_user_id) VALUES (?)",
            Value::Integer(mu_idx as i64)
        );

        // Add the murdered user's corresponding card to everyone's hand
        for id in query!(&db, "SELECT id FROM users").map(|row| row.get("id")) {
            execute!(
                &db,
                r"
                INSERT INTO user_evidence_cards (user_id, card_idx, confirmed)
                VALUES (?, ?, TRUE)
            ",
                Value::Integer(id),
                Value::Integer(mu_card_idx)
            );
        }
    }
    Ok(GameInitialState {
        target_card_idxs,
        murdered_user_id: mu_idx,
    })
}

pub fn game_state<'a>(
    db: &Connection,
    user: AuthUser,
    res: &'a State<Resources>,
) -> Result<GameState<'a>, String> {
    let winner_ids = query!(
        db,
        r"SELECT winners FROM game_state",
        Value::Integer(user.0)
    );
    let winner_ids = winner_ids
        .map(|row| row.get::<String, &str>("winners"))
        .next()
        .unwrap();
    let winner_ids = winner_ids
        .split(",")
        .map(|value| value.parse::<i64>().unwrap())
        .collect::<Vec<i64>>();

    let (target_card_idxs, winner_ids) = if winner_ids.contains(&user.0) {
        // Retrieve the target cards value, which is the indices split by ','
        let values = query!(db, "SELECT target_card_idxs FROM game_state");
        let values = values
            .map(|row| row.get::<String, &str>("target_card_idxs"))
            .next()
            .unwrap();
        let values = values
            .split(",")
            .map(|value| value.parse::<i64>().unwrap())
            .collect::<Vec<i64>>();

        (Some(values), Some(winner_ids))
    } else {
        (None, None)
    };

    let mut murdered_user_idx = query!(
        db,
        r"SELECT id FROM users JOIN game_state ON murdered_user_id = users.id"
    )
    .map(|row| row.get::<i64, &str>("id"));

    return if let Some(murdered_user_idx) = murdered_user_idx.next() {
        Ok(GameState {
            murdered_user_idx,
            evd_card_cats: &res.evd_card_cats,
            target_card_idxs,
            winner_ids,
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
