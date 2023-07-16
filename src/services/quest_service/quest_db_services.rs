use std::collections::HashSet;

use rand::{seq::IteratorRandom, thread_rng};

use crate::{
    execute,
    models::{
        game::Stats,
        quests::{
            EventType, QuestEvent, QuestEventMonster, QuestEventRiddle, QuestReward, RiddleStatus,
        },
    },
    query,
    resources::game_resources::Resources,
};

pub fn generate_quest_event<'a>(
    db: &Connection,
    user: AuthUser,
    event_type: EventType,
    res: &'a State<Resources>,
) -> Result<QuestEvent<'a>, &'static str> {
    let quest_level;
    let mut quest_id: Option<i64> = None;

    // Check if an active quest event already exists
    // for the user
    if let Some(quest_row) = query!(
        db,
        "SELECT id, active FROM quests WHERE user_id = ? AND complete = FALSE",
        Value::Integer(user.0)
    )
    .next()
    {
        // If it's active, do not generate a new event
        // Otherwise, create a level 2 event
        if quest_row.get::<i64, &str>("active") == 1 {
            return Err("Quest already exists and active for user");
        } else {
            quest_level = 2;
            quest_id = quest_row.get("id");
        }
    } else {
        quest_level = 1;
    }

    if let Some(quest_id) = quest_id {
        execute!(
            db,
            "UPDATE quests SET event_lvl=?, active=TRUE WHERE quest_id=?",
            Value::Integer(quest_level),
            Value::Integer(quest_id)
        );
    } else {
        execute!(
            db,
            "INSERT INTO quests (user_id, event_lvl, active, complete) VALUES (?, ?, TRUE, FALSE)",
            Value::Integer(user.0),
            Value::Integer(quest_level)
        );
        quest_id = Some(
            query!(db, "SELECT LAST_INSERT_ROWID()")
                .map(|row| row.get(0))
                .next()
                .unwrap(),
        );
    }

    // First attempt to retrieve any active events
    let mut monster_event = None;
    let mut riddle_event = None;

    match event_type {
        EventType::Monster => {
            monster_event = Some(generate_monster_event(
                db,
                quest_id.unwrap(),
                quest_level,
                res,
            ));
        }
        EventType::Riddle => {
            riddle_event = generate_riddle_event(db, user, quest_id.unwrap(), quest_level, res);
            if riddle_event.is_none() {
                execute!(
                    db,
                    "DELETE FROM quests WHERE id = ?",
                    Value::Integer(quest_id.unwrap())
                );
            }
        }
    };

    Ok(QuestEvent {
        monster_event,
        riddle_event,
    })
}

fn generate_monster_event(
    db: &Connection,
    quest_id: i64,
    quest_level: i64,
    res: &State<Resources>,
) -> QuestEventMonster {
    // Choose a new monster to fight the player
    let (monster_idx, monster) = res
        .monsters
        .iter()
        .enumerate()
        .filter(|(_, monster)| monster.level == quest_level)
        .choose(&mut thread_rng())
        .unwrap();

    execute!(
        db,
        r"
        INSERT INTO stats (health, magicka, armor, wisdom, reflex, missing_next_turn) 
        VALUES (?, ?, ?, ?, ?, FALSE)
    ",
        Value::Integer(monster.stats.health),
        Value::Integer(monster.stats.magicka),
        Value::Integer(monster.stats.armor),
        Value::Integer(monster.stats.wisdom),
        Value::Integer(monster.stats.reflex)
    );
    let stats_id: i64 = query!(db, "SELECT LAST_INSERT_ROWID()")
        .map(|row| row.get(0))
        .next()
        .unwrap();
    execute!(
        db,
        "INSERT INTO quest_monsters (quest_id, monster_idx, stats_id) VALUES (?, ?, ?)",
        Value::Integer(quest_id),
        Value::Integer(monster_idx as i64),
        Value::Integer(stats_id)
    );

    QuestEventMonster {
        monster_idx: monster_idx as i64,
        stats: Stats::from_base_stats(monster.stats),
    }
}

pub fn generate_riddle_event<'a>(
    db: &Connection,
    user: AuthUser,
    quest_id: i64,
    quest_level: i64,
    res: &'a State<Resources>,
) -> Option<QuestEventRiddle<'a>> {
    let ans_riddles: HashSet<i64> = query!(
        db,
        "SELECT riddle_idx FROM user_answered_riddles WHERE user_id = ?",
        Value::Integer(user.0)
    )
    .map(|row| row.get("riddle_idx"))
    .collect();

    // Choose a new riddle to give the player,
    // that the player has not seen
    let riddle_and_idx = res
        .riddles
        .iter()
        .enumerate()
        .filter(|(idx, riddle)| {
            riddle.level == quest_level && !ans_riddles.contains(&(*idx as i64))
        })
        .choose(&mut thread_rng());

    return if let Some((idx, riddle)) = riddle_and_idx {
        execute!(
            db,
            "INSERT INTO quest_riddles (quest_id, riddle_idx, active) VALUES (?, ?, TRUE)",
            Value::Integer(quest_id),
            Value::Integer(idx as i64)
        );

        Some(QuestEventRiddle {
            idx: idx as i64,
            text: &riddle.text,
        })
    } else {
        None
    };
}

pub fn guess_riddle(
    db: &Connection,
    user: AuthUser,
    answer: String,
    res: &State<Resources>,
) -> Result<RiddleStatus, &'static str> {
    let answer = answer.to_lowercase();
    let riddle_idx: i64 = query!(db, "SELECT riddle_idx FROM quest_riddles WHERE user_id = ?")
        .map(|row| row.get("riddle_idx"))
        .next()
        .unwrap();

    if let Some(riddle) = res.riddles.get(riddle_idx as usize) {
        if riddle
            .answers
            .iter()
            .any(|ans| ans.to_lowercase() == answer)
        {
            execute!(
                db,
                "DELETE FROM quest_riddles WHERE user_id = ?",
                Value::Integer(user.0)
            );
            let (quest_id, quest_level): (i64, i64) = query!(
                db,
                "SELECT id, level FROM quests WHERE user_id = ?",
                Value::Integer(user.0)
            )
            .map(|row| (row.get("quest_id"), row.get("level")))
            .next()
            .unwrap();
            if quest_level == 1 {
                execute!(
                    db,
                    "UPDATE quests SET level = 2, active = FALSE WHERE quest_id = ?",
                    Value::Integer(quest_id)
                );
            } else {
                execute!(
                    db,
                    "DELETE FROM quests WHERE quest_id = ?",
                    Value::Integer(quest_id)
                );
                let new_card = retr_evd_card_idxs(db, user, res);
                return Ok(RiddleStatus::Correct(Some(QuestReward {
                    item_idxs: vec![],
                    cat_and_card_idxs: if let Some((cat_idx, card_idx)) = new_card {
                        vec![(cat_idx, card_idx)]
                    } else {
                        vec![]
                    },
                })));
            }
            return Ok(RiddleStatus::Incorrect);
        }
    }
    Err("Could not find riddle for given user")
}

pub fn retr_evd_card_idxs(
    db: &Connection,
    user: AuthUser,
    res: &State<Resources>,
) -> Option<(i64, i64)> {
    let mut rng = thread_rng();

    let conf_card_idxs: Vec<(i64, i64)> = query!(
        &db,
        r"
        SELECT cat_idx, card_idx FROM user_evidence_cards 
        WHERE user_id = ? AND confirmed = TRUE
        ",
        Value::Integer(user.0)
    )
    .map(|row| (row.get("cat_idx"), row.get("card_idx")))
    .collect();

    let target_cards: Vec<(i64, i64)> = query!(
        &db,
        "SELECT cat_idx, card_idx FROM game_target_cards ORDER BY cat_idx"
    )
    .map(|row| (row.get("cat_idx"), row.get("card_idx")))
    .collect();

    let mut all_cat_cards = Vec::new();
    for (cat_idx, cat) in res.evd_cats_and_cards.iter().enumerate() {
        all_cat_cards.append(
            &mut (0..cat.cards.len())
                .map(|card_idx| (cat_idx as i64, card_idx as i64))
                .collect(),
        );
    }

    if let Some(sel_cat_and_card_idxs) = all_cat_cards
        .iter()
        .filter(|pair| !conf_card_idxs.contains(*pair) && !target_cards.contains(*pair))
        .choose(&mut rng)
    {
        if query!(
            &db,
            r"
            SELECT * FROM user_evidence_cards 
            WHERE user_id = ? AND cat_idx = ? AND card_idx = ?
        ",
            Value::Integer(user.0),
            Value::Integer(sel_cat_and_card_idxs.0),
            Value::Integer(sel_cat_and_card_idxs.1)
        )
        .next()
        .is_some()
        {
            execute!(
                &db,
                r"
                UPDATE user_evidence_cards SET confirmed = 1
                WHERE user_id = ? AND cat_idx = ? AND card_idx = ?
            ",
                Value::Integer(user.0),
                Value::Integer(sel_cat_and_card_idxs.0),
                Value::Integer(sel_cat_and_card_idxs.1)
            );
        } else {
            execute!(
                &db,
                r"
                INSERT INTO user_evidence_cards (user_id, cat_idx, card_idx, confirmed)
                VALUES (?, ?, ?, 1)
                ",
                Value::Integer(user.0),
                Value::Integer(sel_cat_and_card_idxs.0),
                Value::Integer(sel_cat_and_card_idxs.1)
            )
        }
        return Some(*sel_cat_and_card_idxs);
    }
    None
}

pub fn equip_item(
    db: &Connection,
    item_idx: i64,
    item_slot: i64,
    user: AuthUser,
    res: &State<Resources>,
) -> Result<(), &'static str> {
    // If an item is being equipped, and the user
    // isn't just requesting unequipping
    if item_idx != -1 {
        // Ensure that the item being equipped exists
        // in the user's inventory, unequipped
        let mut item_in_inv = query!(
            db,
            r"SELECT * FROM user_items WHERE user_id = ? AND item_idx = ? AND equip_slot = NULL",
            Value::Integer(user.0),
            Value::Integer(item_idx)
        );
        if let Some(_) = item_in_inv.next() {
            // Select the item that's being replaced, if
            // on exists in the slot
            let prev_item_idx: Option<i64> = query!(
                db,
                "SELECT item_idx FROM user_items WHERE user_id = ? AND equip_slot = ?",
                Value::Integer(user.0),
                Value::Integer(item_slot)
            )
            .map(|row| row.get("item_idx"))
            .next();

            // If there was an item in that slot, unapply all effects
            if let Some(prev_item) = prev_item_idx.and_then(|idx| Some(&res.items[idx as usize])) {
                if let Some(effects) = prev_item.effects_self.as_ref() {
                    for effect in effects {
                        effect.to_effect().remove_from_user(db, user.0);
                    }
                }
            }

            // Retrieve the new item and apply the effects
            let new_item = &res.items[item_idx as usize];
            if let Some(effects) = new_item.effects_self.as_ref() {
                for effect in effects {
                    effect.to_effect().apply_to_user(db, user.0);
                }
            }
            execute!(
                db,
                r"UPDATE user_items SET equip_slot = ?
                WHERE user_id = ? AND item_idx = ? LIMIT 1",
                Value::Integer(item_slot),
                Value::Integer(user.0),
                Value::Integer(item_idx)
            );
            return Ok(());
        } else {
            return Err("User does not have item");
        }
    // If the slot is only being unequipped, invoke the command
    } else {
        execute!(
            db,
            r"UPDATE user_items SET equip_slot = NULL
            WHERE user_id = ? AND item_idx = ? LIMIT 1",
            Value::Integer(user.0),
            Value::Integer(item_idx)
        );
        return Ok(());
    }
}

pub fn reset_users(db: &Connection, res: &State<Resources>) {
    let ubs = res.user_base_stats;

    // Set all users' statuses to default
    execute!(
        db,
        r"
        UPDATE stats JOIN user_state ON user_id = id SET
            health = ?, magicka = ?, reflex = ?,
            wisdom = ?, armor = ?, missing_next_turn = FALSE
        ",
        Value::Integer(ubs.health),
        Value::Integer(ubs.magicka),
        Value::Integer(ubs.reflex),
        Value::Integer(ubs.wisdom),
        Value::Integer(ubs.armor)
    );

    // Then, re-add any item effects equipped
    let ids = query!(db, "SELECT id in users").map(|row| row.get::<i64, &str>("id"));
    for user_id in ids {
        let item_tags = query!(
            db,
            "SELECT item_tag FROM user_items WHERE user_id = ? AND equiped",
            Value::Integer(user_id)
        )
        .map(|row| row.get::<String, &str>("item_tag"));
        for tag in item_tags {
            let item = res.items.iter().find(|item| item.tag == tag).unwrap();
            if let Some(effects) = item.effects_self.as_ref() {
                for effect in effects.iter().map(|eff| eff.to_effect()) {
                    effect.apply_to_user(db, user_id);
                }
            }
        }
    }
}
