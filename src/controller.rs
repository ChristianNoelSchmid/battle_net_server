use rocket::{get, http::Status, post, routes, serde::json::Json, Build, Rocket};
use sqlite::Value;

use crate::{
    auth::AuthUser,
    execute,
    models::{Card, GetGameState, UserCard, UserCards, Model, User, PostRiddle},
    query,
    sqlite::db,
};

const DEFAULT_GAME_INDEX: i64 = 1;

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/",
        routes![
            get_users,
            get_user_cards,
            get_all_cards,
            post_random_card,
            post_setup_game,
            post_card_guess,
            post_new_riddle
        ],
    );
    rocket
}

#[get("/users")]
fn get_users(_user: AuthUser) -> Json<Vec<(i64, String)>> {
    let db = db();
    return Json(
        query!(db, "SELECT * from users")
            .map(|row| (
                row.get("id"),
                row.get("user_name"),
            ))
            .collect(),
    );
}

#[get("/cards")]
fn get_user_cards(user: AuthUser) -> Json<UserCards> {
    let db = db();
    let card_rows = query!(
        db,
        r"
        SELECT card.id, cat.cat_name, card.item_name, 
               card.item_img_path, user_card.confirmed

        FROM user_found_cards user_card
            JOIN cards card ON card.id = user_card.card_id 
            JOIN categories cat ON cat.id = card.cat_id 

        WHERE user_card.user_id = ?
        ORDER BY user_card.confirmed
    ",
        Value::Integer(user.0)
    );
    let cards: Vec<UserCard> = card_rows.map(|row| UserCard::from_row(row)).collect();

    Json(UserCards {
        confirmed: cards
            .clone()
            .into_iter()
            .filter(|card| card.confirmed)
            .collect(),
        unconfirmed: cards.into_iter().filter(|card| !card.confirmed).collect(),
    })
}

#[get("/all-cards")]
fn get_all_cards(_user: AuthUser) -> Json<Vec<Card>> {
    let db = db();
    let card_rows = query!(
        db,
        r"
            SELECT cat.cat_name, card.id, card.item_name, card.item_img_path
            FROM cards card JOIN categories cat ON cat.id = card.cat_id
        "
    );

    Json(card_rows.map(|row| Card::from_row(row)).collect())
}

#[post("/new-card")]
fn post_random_card(user: AuthUser) -> Json<Option<UserCard>> {
    let db = db();
    let mut rows = query!(
        db,
        // Select all cards that the user has not yet confirmed
        // and that are not in the target collection
        r"
            SELECT card.id, cat_name, item_name, item_img_path FROM cards card
            JOIN categories cat ON card.cat_id = cat.id

            WHERE card.id NOT IN (
                SELECT target_card.card_id FROM game_target_cards target_card 
                WHERE target_card.game_state_id = ?
            )

            AND card.id NOT IN (
                SELECT user_card.card_id FROM user_found_cards user_card
                WHERE user_card.user_id = ? AND user_card.confirmed
            )

            ORDER BY RANDOM()
            LIMIT 1
        ",
        // Insert default game index, since project will only
        // ever involve one game
        Value::Integer(DEFAULT_GAME_INDEX),
        Value::Integer(user.0)
    );

    if let Some(row) = rows.next() {
        let user_card = UserCard {
            card_id: row.get("id"),
            cat_name: row.get("cat_name"),
            item_name: row.get("item_name"),
            item_img_path: row.get("item_img_path"),
            confirmed: true,
        };

        let mut confirmed_rows = query!(
            db,
            r"SELECT user_id FROM user_found_cards WHERE user_id = ? AND card_id = ?",
            Value::Integer(user.0),
            Value::Integer(user_card.card_id)
        );

        if let Some(_) = confirmed_rows.next() {
            execute!(
                db,
                r"
                    UPDATE user_found_cards SET confirmed = true
                    WHERE user_id = ? AND card_id = ?
                ",
                Value::Integer(user.0),
                Value::Integer(user_card.card_id)
            );
        } else {
            execute!(
                db,
                r"
                    INSERT INTO user_found_cards (user_id, card_id, confirmed)
                    VALUES (?, ?, 1)
                ",
                Value::Integer(user.0),
                Value::Integer(user_card.card_id)
            );
        }

        return Json(Some(user_card));
    }
    Json(None)
}

#[post("/setup")]
fn post_setup_game(_user: AuthUser) -> Result<Json<GetGameState>, String> {
    let db = db();

    // Ensure there is no currently running game
    let mut game_states = query!(db, "SELECT id FROM game_states LIMIT 1");
    if let Some(_) = game_states.next() {
        return Result::Err(String::from("Game already running"));
    }

    // Create a murdered user
    let mut murdered_user = query!(
        db, r"SELECT * FROM users ORDER BY RANDOM() LIMIT 1"
    ).map(|row| User::from_row(row));

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
                r"
                    SELECT id, item_name FROM cards 
                    WHERE cat_id = ?  AND id != ?
                    ORDER BY RANDOM() 
                    LIMIT 1
                ",
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
            r"
                INSERT INTO game_states (id, murdered_user_id)
                VALUES (?, ?)
            ",
            Value::Integer(DEFAULT_GAME_INDEX),
            Value::Integer(murdered_user.id)
        );

        for user in query!(db, "SELECT * FROM users").map(|row| User::from_row(row)) {
            // Add the murdered user's corresponding card to everyone's hand
            execute!(
                db, 
                r"
                    INSERT INTO user_found_cards (user_id, card_id, confirmed)
                    VALUES (?, ?, true)
                ", 
                Value::Integer(user.id), 
                Value::Integer(murdered_user.card_id));

            // Add an initial riddle to each user
            execute!(
                db,
                r"
                    INSERT INTO active_riddles (user_id, riddle_id)
                    VALUES (?, (SELECT id FROM riddles ORDER BY RANDOM() LIMIT 1))
                ",
                Value::Integer(user.id)
            );
        }

        for card in target_cards.clone() {
            execute!(
                db,
                r"
                    INSERT INTO game_target_cards (game_state_id, card_id)
                    VALUES (?, ?)
                ",
                Value::Integer(DEFAULT_GAME_INDEX),
                Value::Integer(card.0)
            );
        }

        return Ok(Json(GetGameState {
            target_cards,
            murdered_user: (murdered_user.id, murdered_user.user_name),
        }));
    }
    Err(String::from("Could not setup game. Ensure there are users in the database"))
}

#[post("/update-guessed-card/<card_id>")]
fn post_card_guess(card_id: i64, user: AuthUser) -> Status {
    let db = db();
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
            r"
            INSERT INTO user_found_cards (user_id, card_id, confirmed)
            VALUES (?, ?, false)
        ",
            Value::Integer(user.0),
            Value::Integer(card_id)
        );
    }
    Status::Ok
}

#[post("/add-riddle", format = "json", data = "<riddle>")]
fn post_new_riddle(riddle: Json<PostRiddle>) -> Status {
    let db = db();
    let answers = riddle.answers.join(",");
    execute!(
        db, 
        "INSERT INTO riddles (text, answers) VALUES (?, ?)", 
        Value::String(riddle.text.clone()),
        Value::String(answers)
    );

    Status::Ok
}
