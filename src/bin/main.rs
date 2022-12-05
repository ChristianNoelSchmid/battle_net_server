#[macro_use]
extern crate rocket;
use christmas_2022::{controllers::{game_state_controller, quest_controller}, cors::Cors};
use rocket::fs::{FileServer, relative};

#[launch]
fn rocket() -> _ {
    let mut rckt = rocket::build()
        // Build the file server
        .mount("/assets", FileServer::from(relative!("static")))
        .attach(Cors);

    // Apply all the routes from controllers
    rckt = game_state_controller::routes(rckt);
    rckt = quest_controller::routes(rckt);

    rckt
}
