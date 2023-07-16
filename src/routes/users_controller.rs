/*use rocket::{get, post, routes, serde::json::Json, Build, Rocket, State};

use crate::{
    db_services::user_db_services::{update_evidence_card, user_state},
    middleware::auth::AuthUser,
    models::users::UserState,
    resources::game_resources::Resources,
    sqlite::db,
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/users",
        routes![post_login, get_user_state, post_update_evidence_card],
    );
    rocket
}

#[post("/login")]
fn post_login(_user: AuthUser) {}

#[get("/state")]
fn get_user_state<'a>(user: AuthUser, res: &'a State<Resources>) -> Json<UserState<'a>> {
    let db = db();

    // Get all user's evidence card ids from db
    Json(user_state(&db, user, res))
}

#[post("/update-evidence-card/<cat_idx>/<card_idx>")]
fn post_update_evidence_card(cat_idx: i64, card_idx: i64, user: AuthUser) {
    let db = db();
    update_evidence_card(&db, user, cat_idx, card_idx);
}
*/