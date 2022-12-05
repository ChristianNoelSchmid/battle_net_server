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
    db_services::game_db_service::{all_evidence_cards, game_state, setup_game},
    execute,
    models::{EvidenceCard, GameInitialState, GameState, User, UserState},
    query,
    sqlite::db,
};

const DEFAULT_GAME_INDEX: i64 = 1;

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/game",
        routes![
            get_game_state,
            get_all_evidence_cards,
            post_setup_game,
            post_guess_target_cards
        ],
    );
    rocket
}

#[post("/setup")]
fn post_setup_game(_user: AuthUser) -> Result<Json<GameInitialState>, status::BadRequest<String>> {
    let db = db();
    match setup_game(&db) {
        Ok(game_init_state) => Ok(Json(game_init_state)),
        Err(text) => Err(BadRequest(Some(text))),
    }
}

#[get("/all-cards")]
fn get_all_evidence_cards(_user: AuthUser) -> Json<Vec<EvidenceCard>> {
    let db = db();
    Json(all_evidence_cards(&db))
}

#[get("/state")]
fn get_game_state(user: AuthUser) -> Result<Json<GameState>, status::BadRequest<String>> {
    let db = db();
    let game_state = game_state(&db, user);

    return match game_state {
        Ok(game_state) => Ok(Json(game_state)),
        Err(text) => Err(status::BadRequest(Some(text))),
    };
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
