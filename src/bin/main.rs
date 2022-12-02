#[macro_use]
extern crate rocket;
use christmas_2022::{controller, cors::Cors};

#[launch]
fn rocket() -> _ {
    let rckt = rocket::build().attach(Cors);
    let rckt = controller::routes(rckt);

    rckt
}
