use rocket::{Build, Rocket, post, serde::json::Json, http::Status, routes};
use sqlite::Value;

use crate::{execute, sqlite::db, models::{PostRiddle, RiddleProgress, EvidenceCard, Riddle}, auth::AuthUser, query};

pub fn routes(rocket: Rocket<Build>) -> Rocket<Build> {
    let rocket = rocket.mount(
        "/quests",
        routes![post_new_riddle, post_guess_riddle]
    );
    rocket
}

#[post("/add-riddle", format = "json", data = "<riddle>")]
fn post_new_riddle(riddle: Json<PostRiddle>) -> Status {
    let db = db();
    let answers = riddle.answers.join(",");
    execute!(
        db,
        "INSERT INTO riddles (text, answers) VALUES (?, ?)",
        Value::String(riddle.text.clone()),
        Value::String(answers)
    );

    Status::Ok
}

#[post("/guess-riddle/<riddle_id>/<answer>")]
fn post_guess_riddle(riddle_id: i64, answer: String, user: AuthUser)
    -> Result<Json<RiddleProgress>, Status> {
    let db = db();
    // Retrieve the riddle
    let mut riddle = query!(
        db,
        "SELECT answers FROM riddles WHERE id = ?",
        Value::Integer(riddle_id)
    );

    if let Some(riddle) = riddle.next() {
        let answers = riddle.get::<String, &str>("answers");
        for riddle_answer in answers.split(',') {
            if answer == riddle_answer {
                let new_card = retrieve_random_card(user);
                execute!(
                    db,
                    r"UPDATE active_riddles SET answered = 1
                      WHERE user_id = ? AND riddle_id = ?",
                    Value::Integer(user.0),
                    Value::Integer(riddle_id)
                );
                let mut riddle = query!(
                    db,
                    r"SELECT id, text FROM riddles WHERE id NOT IN (
                        SELECT riddle_id FROM active_riddles WHERE user_id = ?
                    )",
                    Value::Integer(user.0)
                )
                .map(|row| Riddle {
                    id: row.get("id"),
                    text: row.get("text"),
                });

                let riddle = riddle.next();

                if let Some(riddle) = riddle.clone() {
                    execute!(
                        db,
                        r"INSERT INTO active_riddles (user_id, riddle_id) VALUES (?, ?)",
                        Value::Integer(user.0),
                        Value::Integer(riddle.id)
                    );
                }
                return Ok(Json(RiddleProgress::Correct((riddle, new_card))));
            }
        }
        // Return incorrect if riddle answer was not correct
        return Ok(Json(RiddleProgress::Incorrect));
    } else {
        // Riddle was not found
        return Err(Status::NotFound);
    }
}

fn retrieve_random_card(user: AuthUser) -> Option<EvidenceCard> {
    let db = db();
    let mut rows = query!(
        db,
        // Select all cards that the user has not yet confirmed
        // and that are not in the target collection
        r"SELECT card.id, cat_name, item_name, item_img_path FROM cards card
          JOIN categories cat ON card.cat_id = cat.id

          WHERE card.id NOT IN (
            SELECT target_card.card_id FROM game_target_cards target_card 
          )

          AND card.id NOT IN (
            SELECT user_card.card_id FROM user_found_cards user_card
            WHERE user_card.user_id = ? AND user_card.confirmed
          )

          ORDER BY RANDOM()
          LIMIT 1",
        // Insert default game index, since project will only
        // ever involve one game
        Value::Integer(user.0)
    );

    if let Some(row) = rows.next() {
        let user_card = EvidenceCard {
            card_id: row.get("id"),
            item_name: row.get("item_name"),
            item_img_path: row.get("item_img_path"),
        };

        let mut confirmed_rows = query!(
            db,
            r"SELECT user_id FROM user_found_cards WHERE user_id = ? AND card_id = ?",
            Value::Integer(user.0),
            Value::Integer(user_card.card_id)
        );

        if let Some(_) = confirmed_rows.next() {
            execute!(
                db,
                r"UPDATE user_found_cards SET confirmed = true
                  WHERE user_id = ? AND card_id = ?",
                Value::Integer(user.0),
                Value::Integer(user_card.card_id)
            );
        } else {
            execute!(
                db,
                r"INSERT INTO user_found_cards (user_id, card_id, confirmed)
                  VALUES (?, ?, 1)",
                Value::Integer(user.0),
                Value::Integer(user_card.card_id)
            );
        }

        return Some(user_card);
    }
    None
}