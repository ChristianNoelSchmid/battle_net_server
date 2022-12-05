use sqlite::{Connection, Value};

use crate::{auth::AuthUser, query};

pub fn user_riddle(db: &Connection, user: AuthUser) -> Option<(i64, String)> {
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
