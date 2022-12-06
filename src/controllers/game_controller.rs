use rocket::{
    get,
    post,
    response::status::{self, BadRequest},
    routes,
    serde::json::Json,
    Build, Rocket,
};

use crate::{
    auth::AuthUser,
    db_services::game_db_services::{game_state, setup_game, guess_target_cards},
    sqlite::db, models::game::{GameInitialState, GameState},
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/game",
        routes![
            get_game_state,
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
fn post_guess_target_cards(guess: Json<Vec<i64>>, user: AuthUser) -> Json<bool> {
    let db = db();
    Json(guess_target_cards(&db, &guess, user))
}
