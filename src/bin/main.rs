#[macro_use]
extern crate rocket;
use christmas_2022::{controller, cors::Cors};
use rocket::fs::{FileServer, relative};

#[launch]
fn rocket() -> _ {
    let rckt = rocket::build()
        .mount("/assets", FileServer::from(relative!("static")))
        .attach(Cors);
    let rckt = controller::routes(rckt);

    rckt
}
