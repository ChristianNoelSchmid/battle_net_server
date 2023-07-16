use rocket::State;
use sqlite::{Connection, Value};

use crate::{
    execute,
    middleware::auth::AuthUser,
    models::{
        game::Stats,
        model::Model,
        quests::{QuestEvent, QuestEventMonster, QuestEventRiddle},
        users::{UserEvidenceCard, UserState},
    },
    query,
    resources::game_resources::Resources,
};

///
/// Returns a list of the user's currently active evidence
/// cards, mapped to whether each is a confirmed card.
///
pub fn user_state<'a>(db: &Connection, user: AuthUser, res: &'a State<Resources>) -> UserState<'a> {
    let evidence_cards = query!(
        db,
        r"
        SELECT cat_idx, card_idx, confirmed FROM user_evidence_cards
        WHERE user_id = ?
        ",
        Value::Integer(user.0)
    )
    .map(|row| UserEvidenceCard {
        cat_idx: row.get("cat_idx"),
        card_idx: row.get("card_idx"),
        confirmed: row.get::<i64, &str>("confirmed") == 1,
    })
    .collect();

    // Get the player's current stats
    let stats = query!(
        db,
        r"
        SELECT * FROM stats
            JOIN user_state ON user_state.cur_stats_id = stats.id
            JOIN users ON users.id = user_state.user_id
        WHERE users.id = ?
        ",
        Value::Integer(user.0)
    )
    .map(|row| Stats::from_row(row))
    .next()
    .unwrap();

    // Attempt to retrieve the user's active quest, if it exists
    let pair = query!(
        db,
        r"SELECT id, active FROM quests WHERE user_id = ?",
        Value::Integer(user.0)
    )
    .map(|row| {
        (
            row.get::<i64, &str>("id"),
            row.get::<i64, &str>("active") == 1,
        )
    })
    .next();

    let mut quest_event = None;

    if let Some((id, active)) = pair {
        // If the user is currently in either a riddle or monster quest
        if active {
            // First attempt to retrieve any relevant monster quest
            let quest_monster = query!(
                db,
                "SELECT * FROM quest_monsters WHERE quest_id = ?",
                Value::Integer(id)
            )
            .next();
            if let Some(quest_monster) = quest_monster {
                // Get the monsters stats as well
                let monster_stats = query!(
                    db,
                    "SELECT * FROM stats JOIN quest_monsters ON stats.id = ?",
                    Value::Integer(quest_monster.get("stats_id"))
                )
                .next()
                .unwrap();

                quest_event = Some(QuestEvent {
                    monster_event: Some(QuestEventMonster {
                        monster_idx: quest_monster.get("monster_idx"),
                        stats: Stats::from_row(monster_stats),
                    }),
                    riddle_event: None,
                });
            } else if let Some(riddle_row) = query!(
                db,
                "SELECT * FROM quest_riddles WHERE quest_id = ?",
                Value::Integer(id)
            )
            .next()
            {
                let riddle_idx: i64 = riddle_row.get("riddle_idx");
                let riddle = &res.riddles[riddle_idx as usize];
                quest_event = Some(QuestEvent {
                    monster_event: None,
                    riddle_event: Some(QuestEventRiddle {
                        idx: riddle_idx,
                        text: &riddle.text,
                    }),
                });
            }
        }
    }

    // Get the user's current items
    let user_items = query!(
        db,
        "SELECT item_idx, equip_slot FROM user_items WHERE user_id = ?",
        Value::Integer(user.0)
    );
    let user_items: Vec<(i64, Option<i64>)> = user_items
        .map(|row| (row.get("item_idx"), row.try_get("equip_slot").ok()))
        .collect();

    UserState {
        evidence_cards,
        stats,
        quest_event,
        item_idxs_and_slots: user_items,
    }
}

pub fn update_evidence_card(db: &Connection, user: AuthUser, cat_idx: i64, card_idx: i64) {
    let mut card = query!(
        db,
        r"SELECT confirmed FROM user_evidence_cards WHERE user_id = ? AND cat_idx = ? AND card_idx = ?",
        Value::Integer(user.0),
        Value::Integer(cat_idx),
        Value::Integer(card_idx)
    );
    if let Some(card) = card.next() {
        if card.get::<i64, &str>("confirmed") == 0 {
            execute!(
                db,
                "DELETE FROM user_evidence_cards WHERE user_id = ? AND cat_idx = ? AND card_idx = ?",
                Value::Integer(user.0),
                Value::Integer(cat_idx),
                Value::Integer(card_idx)
            );
        }
    } else {
        execute!(
            db,
            r"INSERT INTO user_evidence_cards (user_id, cat_idx, card_idx, confirmed)
              VALUES (?, ?, ?, 0)",
            Value::Integer(user.0),
            Value::Integer(cat_idx),
            Value::Integer(card_idx)
        );
    }
}
