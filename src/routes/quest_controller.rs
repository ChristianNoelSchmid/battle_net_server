/*use rocket::{post, routes, serde::json::Json, Build, Rocket, State};

use crate::{
    db_services::quest_db_services::{
        equip_item, generate_quest_event, guess_riddle, retr_evd_card_idxs,
    },
    middleware::auth::AuthUser,
    models::quests::{EventType, QuestEvent, RiddleStatus},
    resources::game_resources::Resources,
    sqlite::db,
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/quests",
        routes![
            post_guess_riddle,
            post_new_card,
            post_new_event,
            post_equip_item
        ],
    );
    rocket
}

#[post("/new-event/<ty>")]
fn post_new_event<'a>(
    user: AuthUser,
    res: &'a State<Resources>,
    ty: i64,
) -> Json<Result<QuestEvent<'a>, &'static str>> {
    let db = db();
    Json(generate_quest_event(
        &db,
        user,
        if ty == 0 {
            EventType::Monster
        } else {
            EventType::Riddle
        },
        res,
    ))
}

#[post("/guess-riddle/<ans>")]
fn post_guess_riddle(
    user: AuthUser,
    ans: String,
    res: &State<Resources>,
) -> Result<Json<RiddleStatus>, &'static str> {
    let db = db();
    guess_riddle(&db, user, ans, res).and_then(|result| Ok(Json(result)))
}

#[post("/new-card")]
fn post_new_card(user: AuthUser, res: &State<Resources>) -> Json<Option<(i64, i64)>> {
    let db = db();
    Json(retr_evd_card_idxs(&db, user, res))
}

#[post("/equip-item/<idx>/<slot>")]
fn post_equip_item(
    user: AuthUser,
    idx: i64,
    slot: i64,
    res: &State<Resources>,
) -> Result<(), &'static str> {
    let db = db();
    equip_item(&db, idx, slot, user, res)
}
*/