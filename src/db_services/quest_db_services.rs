use rand::{seq::IteratorRandom, thread_rng, RngCore};
use rocket::State;
use sqlite::{Connection, Value};

use crate::{
    execute,
    middleware::auth::AuthUser,
    models::quests::{GuessRiddleResult, QuestEvent},
    query,
    resources::game_resources::Resources,
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

pub fn guess_riddle(
    db: &Connection,
    user: AuthUser,
    riddle_idx: i64,
    answer: String,
    res: &State<Resources>,
) -> Option<GuessRiddleResult> {
    if let Some(riddle) = res.riddles.get(riddle_idx as usize) {
        if riddle.answers.contains(&answer) {
            return Some(GuessRiddleResult::Correct);
        }
        return Some(GuessRiddleResult::Incorrect);
    }
    None
}

pub fn retr_evd_cat_card_idxs(
    db: &Connection,
    user: AuthUser,
    res: &State<Resources>,
) -> Option<(i64, i64)> {
    let mut rng = thread_rng();

    let conf_card_idxs: Vec<(i64, i64)> = query!(
        &db,
        r"
        SELECT cat_idx, card_idx FROM user_evidence_cards WHERE user_id = ? AND confirmed = TRUE
    ",
        Value::Integer(user.0)
    )
    .map(|row| (row.get("cat_idx"), row.get("card_idx")))
    .collect();

    let mut all_cat_cards = Vec::new();
    for (cat_idx, cat) in res.evd_card_cats.iter().enumerate() {
        all_cat_cards.append(
            &mut (0..cat.cards.len())
                .map(|card_idx| (cat_idx as i64, card_idx as i64))
                .collect(),
        );
    }

    if let Some(sel_cat_and_card_idxs) = all_cat_cards
        .iter()
        .filter(|pair| !conf_card_idxs.contains(*pair))
        .choose(&mut rng)
    {
        return Some(*sel_cat_and_card_idxs);
    }
    None
}
