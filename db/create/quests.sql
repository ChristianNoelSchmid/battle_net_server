/* All users' item inventories */
CREATE TABLE user_items(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    item_tag TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id)
);
/* All users' spell inventories */
CREATE TABLE user_spells(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    spell_tag TEXT NOT NULL,

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
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    quest_status INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY (user_id) REFERENCES users (id)
);
/* All instantiated monsters associated with individual quests */
CREATE TABLE quest_monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quest_id INTEGER NOT NULL,
    monster_tag TEXT NOT NULL,
    stats_id INTEGER NOT NULL,
    defeated BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (stats_id) REFERENCES stats(id)
);
/* All instantiated riddles associated with individual quests */
CREATE TABLE quest_riddles (
    quest_id INTEGER NOT NULL,
    riddle_id INTEGER NOT NULL,
    answered BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (quest_id, riddle_id), 
    FOREIGN KEY (quest_id) REFERENCES quests (id)
);