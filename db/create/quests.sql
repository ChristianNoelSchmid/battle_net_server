/* All users' spell inventories */
CREATE TABLE user_spells(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    spell_tag TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

/* All users' item inventories */
CREATE TABLE user_items(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    item_idx INTEGER NOT NULL,
    equip_slot INTEGER DEFAULT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE user_answered_riddles(
    user_id INTEGER NOT NULL,
    riddle_idx INTEGER NOT NULL,
    PRIMARY KEY (user_id, riddle_idx),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

/* All quests, run by individual users */
CREATE TABLE quests (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    lvl INTEGER DEFAULT 0 NOT NULL,
    completed BOOLEAN DEFAULT FALSE NOT NULL,
    created_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    quest_type INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

/* All instantiated monsters associated with individual quests */
CREATE TABLE quest_monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    quest_id INTEGER NOT NULL,
    monster_idx TEXT NOT NULL,
    stats_id INTEGER NOT NULL,
    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (stats_id) REFERENCES stats(id)
);

/* All instantiated riddles associated with individual quests */
CREATE TABLE quest_riddles (
    quest_id INTEGER NOT NULL,
    riddle_idx INTEGER NOT NULL,
    PRIMARY KEY (quest_id, riddle_idx),
    FOREIGN KEY (quest_id) REFERENCES quests (id)
);