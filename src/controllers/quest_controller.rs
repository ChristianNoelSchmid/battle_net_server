use rocket::{post, response::status, routes, serde::json::Json, Build, Rocket};

use crate::{
    auth::AuthUser,
    db_services::quest_db_services::{guess_riddle, retr_evidence_card},
    sqlite::db, models::{quests::RiddleProgress, game::EvidenceCard},
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount("/quests", routes![post_guess_riddle, post_new_card]);
    rocket
}

#[post("/guess-riddle/<riddle_id>/<answer>")]
fn post_guess_riddle(
    riddle_id: i64,
    answer: String,
    user: AuthUser,
) -> Result<Json<RiddleProgress>, status::NotFound<String>> {
    let db = db();
    return match guess_riddle(&db, user, riddle_id, answer) {
        Some(progress) => Ok(Json(progress)),
        None => Err(status::NotFound(format!(
            "riddle with id {riddle_id} does not exist"
        ))),
    };
}

#[post("/new-card")]
fn post_new_card(user: AuthUser) -> Json<EvidenceCard> {
    let db = db();
    Json(retr_evidence_card(&db, user).unwrap())
}
