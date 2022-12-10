use rocket::{post, response::status, routes, serde::json::Json, Build, Rocket, State};

use crate::{
    db_services::quest_db_services::{guess_riddle, retr_evd_cat_card_idxs},
    middleware::auth::AuthUser,
    models::quests::GuessRiddleResult,
    resources::game_resources::Resources,
    sqlite::db,
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount("/quests", routes![post_guess_riddle, post_new_card]);
    rocket
}

#[post("/guess-riddle/<riddle_idx>/<answer>")]
fn post_guess_riddle(
    riddle_idx: i64,
    answer: String,
    user: AuthUser,
    res: &State<Resources>,
) -> Result<Json<GuessRiddleResult>, status::NotFound<String>> {
    let db = db();
    return match guess_riddle(&db, user, riddle_idx, answer, res) {
        Some(progress) => Ok(Json(progress)),
        None => Err(status::NotFound(format!(
            "riddle with id {riddle_idx} does not exist"
        ))),
    };
}

#[post("/new-card")]
fn post_new_card(user: AuthUser, res: &State<Resources>) -> Json<Option<(i64, i64)>> {
    let db = db();
    Json(retr_evd_cat_card_idxs(&db, user, res))
}
