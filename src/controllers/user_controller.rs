use rocket::{get, post, routes, serde::json::Json, Build, Rocket};

use crate::{
    auth::AuthUser,
    db_services::user_db_service::{all_users, update_evidence_card, user_evidence_cards},
    models::{User, UserState},
    sqlite::db,
};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/users",
        routes![
            get_all_users,
            post_login,
            get_user_state,
            post_update_evidence_card
        ],
    );
    rocket
}

#[get("/all")]
fn get_all_users(_user: AuthUser) -> Json<Vec<User>> {
    let db = db();
    Json(all_users(&db))
}

#[post("/login")]
fn post_login(_user: AuthUser) {}

#[get("/state")]
fn get_user_state(user: AuthUser) -> Json<UserState> {
    let db = db();

    // Get all user's evidence card ids from db
    let evidence_cards = user_evidence_cards(&db, user);

    // Filter out confirmed cards
    let confirmed_card_ids = evidence_cards
        .clone()
        .iter()
        .filter(|(_id, confirmed)| *confirmed)
        .map(|(id, _confirmed)| *id)
        .collect();

    // Filter out unconfirmed cards
    let unconfirmed_card_ids = evidence_cards
        .iter()
        .filter(|(_id, confirmed)| !*confirmed)
        .map(|(id, _confirmed)| *id)
        .collect();

    Json(UserState {
        confirmed_card_ids,
        unconfirmed_card_ids,
    })
}

#[post("/update-evidence-card/<card_id>")]
fn post_update_evidence_card(card_id: i64, user: AuthUser) {
    let db = db();
    update_evidence_card(&db, user, card_id);
}
