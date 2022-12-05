use rocket::{
    get,
    http::Status,
    post,
    response::status::{self, BadRequest},
    routes,
    serde::json::Json,
    Build, Rocket,
};
use sqlite::Value;

use crate::{
    auth::AuthUser,
    execute,
    models::{
        EvidenceCard, GameInitialState, GameState, Model, PostRiddle, Riddle, RiddleProgress, User,
        UserState,
    },
    query,
    sqlite::db,
};

const DEFAULT_GAME_INDEX: i64 = 1;

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/game",
        routes![
            login,
            get_users,
            get_game_state,
            get_user_state,
            get_all_cards,
            post_setup_game,
            post_card_guess,
            post_guess_target_cards
        ],
    );
    rocket
}

#[post("/login")]
fn login(_user: AuthUser) {}

#[get("/users")]
fn get_users(_user: AuthUser) -> Json<Vec<(i64, String)>> {
    let db = db();
    return Json(
        query!(db, "SELECT * from users")
            .map(|row| (row.get("id"), row.get("user_name")))
            .collect(),
    );
}

#[get("/user-state")]
fn get_user_state(user: AuthUser) -> Json<UserState> {
    let db = db();
    let cards = query!(
        db,
        r"SELECT card.id, user_card.confirmed FROM cards card
          JOIN user_found_cards user_card ON card.id = user_card.card_id 
          WHERE user_card.user_id = ?",
        Value::Integer(user.0)
    )
    .map(|row| (row.get("id"), row.get::<i64, &str>("confirmed") == 1))
    .collect::<Vec<(i64, bool)>>();

    let mut riddle = query!(
        db,
        r"SELECT riddle.id, riddle.text FROM active_riddles active_riddle
          JOIN riddles riddle ON riddle.id = active_riddle.riddle_id
          WHERE active_riddle.user_id = ?
          AND active_riddle.answered = false",
        Value::Integer(user.0)
    );

    Json(UserState {
        confirmed_card_ids: cards
            .clone().iter()
            .filter(|(_id, confirmed)| *confirmed)
            .map(|(id, _confirmed)| *id)
            .collect(),
        unconfirmed_card_ids: cards
            .iter().filter(|(_id, confirmed)| !*confirmed)
            .map(|(id, _confirmed)| *id)
            .collect(),
        current_riddle: riddle
            .next()
            .and_then(|r| Some((r.get("id"), r.get("text")))),
    })
}

#[get("/all-cards")]
fn get_all_cards(_user: AuthUser) -> Json<Vec<EvidenceCard>> {
    let db = db();
    let card_rows = query!(
        db,
        r"SELECT cat.cat_name, card.id, card.item_name, card.item_img_path
          FROM cards card JOIN categories cat ON cat.id = card.cat_id"
    );

    Json(card_rows.map(|row| EvidenceCard::from_row(row)).collect())
}

#[post("/setup")]
fn post_setup_game(_user: AuthUser) -> Result<Json<GameInitialState>, status::BadRequest<String>> {
    let db = db();

    // Ensure there is no currently running game
    let mut game_states = query!(db, "SELECT id FROM game_states LIMIT 1");
    if let Some(_) = game_states.next() {
        return Result::Err(BadRequest(Some(String::from("Game already running"))));
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
            r"INSERT INTO game_states (id, murdered_user_id) VALUES (?, ?)",
            Value::Integer(DEFAULT_GAME_INDEX),
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

            // Add an initial riddle to each user
            execute!(
                db,
                r"INSERT INTO active_riddles (user_id, riddle_id)
                VALUES (?, (SELECT id FROM riddles ORDER BY RANDOM() LIMIT 1))",
                Value::Integer(user.id)
            );
        }

        for card in target_cards.clone() {
            execute!(
                db,
                r"INSERT INTO game_target_cards (game_state_id, card_id) VALUES (?, ?)",
                Value::Integer(DEFAULT_GAME_INDEX),
                Value::Integer(card.0)
            );
        }

        return Ok(Json(GameInitialState {
            target_cards,
            murdered_user: (murdered_user.id, murdered_user.user_name),
        }));
    }
    Err(BadRequest(Some(String::from(
        "Could not setup game. Ensure there are users in the database",
    ))))
}

#[get("/game-state")]
fn get_game_state(user: AuthUser) -> Result<Json<GameState>, status::BadRequest<String>> {
    let db = db();
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
                Value::Integer(DEFAULT_GAME_INDEX)
            )
            .map(|row| EvidenceCard::from_row(row)).collect::<Vec<EvidenceCard>>(),
        );

        winners = Some(
            query!(
                db,
                r"SELECT * from winners winner JOIN users user ON winner.user_id = user.id
                WHERE winner.game_state_id = ? ORDER BY winner.id",
                Value::Integer(DEFAULT_GAME_INDEX)
            )
            .map(|row| User::from_row(row)).collect::<Vec<User>>(),
        );
    } else {
        target_cards = None;
        winners = None;
    };
    let cats = query!(db, r"SELECT id, cat_name FROM categories")
        .map(|row| (row.get("id"), row.get("cat_name")));

    let cards = query!(db, r"SELECT * FROM cards card")
        .map(|row| EvidenceCard::from_row(row)).collect::<Vec<EvidenceCard>>();

    let mut murdered_user = query!(
        db,
        r"SELECT * FROM game_states game_state
          JOIN users user ON game_state.murdered_user_id = user.id
          WHERE game_state.id = ?",
        Value::Integer(DEFAULT_GAME_INDEX)
    )
    .map(|row| User::from_row(row));

    if let Some(murdered_user) = murdered_user.next() {
        return Ok(Json(GameState {
            game_state_id: DEFAULT_GAME_INDEX,
            murdered_user: murdered_user,
            categories: cats.collect(),
            cards,
            target_cards,
            winners,
        }));
    } else {
        return Err(status::BadRequest(Some(String::from(
            "Game has not yet started.",
        ))));
    }
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
            r"INSERT INTO user_found_cards (user_id, card_id, confirmed)
              VALUES (?, ?, false)",
            Value::Integer(user.0),
            Value::Integer(card_id)
        );
    }
    Status::Ok
}

#[post("/guess-target-cards", format = "json", data = "<guess>")]
fn post_guess_target_cards(guess: Json<Vec<i64>>, user: AuthUser) -> Status {
    let db = db();
    let target_ids = query!(
        db,
        r"
            SELECT card_id FROM game_target_cards
            WHERE game_state_id = ?
        ",
        Value::Integer(DEFAULT_GAME_INDEX)
    )
    .map(|row| row.get("card_id"))
    .collect::<Vec<i64>>();

    if guess.0.len() != target_ids.len() {
        return Status::BadRequest;
    }
    for guess_id in guess.0 {
        if !target_ids.contains(&guess_id) {
            return Status::BadRequest;
        }
    }
    execute!(
        db,
        r"INSERT INTO winners (game_state_id, user_id) VALUES (?, ?)",
        Value::Integer(DEFAULT_GAME_INDEX),
        Value::Integer(user.0)
    );
    Status::Ok
}
