#[macro_use]
extern crate rocket;
use lazy_static::lazy_static;

use christmas_2022::{
    controllers::{game_controller, quest_controller, users_controller},
    middleware::cors::Cors,
    resources::game_resources::{ResourceLoader, Resources},
};
use rocket::fs::{relative, FileServer};
lazy_static! {
    static ref RES_LOADER: ResourceLoader = ResourceLoader::load(String::new());
}

#[launch]
fn rocket() -> _ {
    let mut rckt = rocket::build()
        // Build Cors policy
        .attach(Cors)
        // Create Resources state management
        .manage(Resources::from_loader(&RES_LOADER))
        // Build the file server
        .mount("/assets", FileServer::from(relative!("static")));

    // Apply all the routes from controllers
    rckt = game_controller::routes(rckt);
    rckt = quest_controller::routes(rckt);
    rckt = users_controller::routes(rckt);

    rckt
}
