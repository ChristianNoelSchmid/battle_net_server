use rand::{thread_rng, RngCore};
use sqlite::{Connection, Value};

use crate::{
    auth::AuthUser,
    execute,
    query, models::{quests::{QuestEvent, RiddleProgress, Riddle}, game::EvidenceCard, model::Model}
};

pub fn generate_quest_event(db: &Connection, user: AuthUser, event_id: i64) -> QuestEvent {
    match thread_rng().next_u32() % 2 {
        0 => generate_monster_event(db, user, event_id),
        _ => generate_riddle_event(db, user, event_id),
    }

    todo!();
}

pub fn generate_monster_event(db: &Connection, user: AuthUser, event_id: i64) {}

pub fn generate_riddle_event(db: &Connection, user: AuthUser, event_id: i64) {}

pub fn retr_user_riddle(db: &Connection, user: AuthUser) -> Option<(i64, String)> {
    let riddle = query!(
        db,
        r"SELECT riddle.id, riddle.text FROM quest_riddles quest_riddle
          JOIN riddles riddle ON riddle.id = quest_riddle.riddle_id
          WHERE quest_riddle.user_id = ?
          AND quest_riddle.answered = false",
        Value::Integer(user.0)
    );

    let mut riddle = riddle.map(|row| (row.get("id"), row.get("text")));

    riddle.next()
}

pub fn guess_riddle(
    db: &Connection,
    user: AuthUser,
    riddle_id: i64,
    answer: String,
) -> Option<RiddleProgress> {
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
                let new_card = retr_evidence_card(&db, user);
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
                return Some(RiddleProgress::Correct((riddle, new_card)));
            }
        }
        // Return incorrect if riddle answer was not correct
        return Some(RiddleProgress::Incorrect);
    } else {
        // Riddle was not found
        return None;
    }
}

pub fn retr_evidence_card(db: &Connection, user: AuthUser) -> Option<EvidenceCard> {
    let mut new_card = query!(
        db,
        // Select all cards that the user has not yet confirmed
        // and that are not in the target collection
        r"SELECT * FROM evidence_cards evidence_card

          WHERE evidence_card.id NOT IN (
            SELECT target_card.card_id FROM game_target_cards target_card
          )
          AND evidence_card.id NOT IN (
            SELECT user_card.card_id FROM user_evidence_cards user_card
            WHERE user_card.user_id = ? AND user_card.confirmed
          )

          ORDER BY RANDOM()
          LIMIT 1",
        Value::Integer(user.0)
    ).map(|row| EvidenceCard::from_row(row));

    if let Some(new_card) = new_card.next() {
        let mut picked_card = query!(
            db,
            r"SELECT user_id FROM user_evidence_cards WHERE user_id = ? AND card_id = ?",
            Value::Integer(user.0),
            Value::Integer(new_card.card_id)
        );

        if let Some(_) = picked_card.next() {
            execute!(
                db,
                r"UPDATE user_evidence_cards SET confirmed = true
                    WHERE user_id = ? AND card_id = ?",
                Value::Integer(user.0),
                Value::Integer(new_card.card_id)
            );
        } else {
            execute!(
                db,
                r"INSERT INTO user_evidence_cards (user_id, card_id, confirmed)
                    VALUES (?, ?, TRUE)",
                Value::Integer(user.0),
                Value::Integer(new_card.card_id)
            );
        }

        return Some(new_card);
    }
    None
}
