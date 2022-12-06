#[macro_use]
extern crate rocket;
use christmas_2022::{
    controllers::{game_controller, quest_controller, users_controller},
    cors::Cors,
};
use rocket::fs::{relative, FileServer};

#[launch]
fn rocket() -> _ {
    let mut rckt = rocket::build()
        // Build the file server
        .mount("/assets", FileServer::from(relative!("static")))
        .attach(Cors);

    // Apply all the routes from controllers
    rckt = game_controller::routes(rckt);
    rckt = quest_controller::routes(rckt);
    rckt = users_controller::routes(rckt);

    rckt
}
