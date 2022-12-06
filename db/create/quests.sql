/* All possible riddles in the game */
CREATE TABLE riddles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    answers TEXT NOT NULL
);
/* All possible monsters in the game */
CREATE TABLE monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lvl: INTEGER NOT NULL,
    base_damage INTEGER NOT NULL,
    base_health INTEGER NOT NULL,
    base_flee_chance INTEGER NOT NULL
);
/* All possible items in the game */
CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    health_inc: INTEGER,
    damage_inc: INTEGER,
    flee_inc: INTEGER,
    req_monster_lvl: INTEGER
);
/* All possible spells in the game */
CREATE TABLE spells (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    health_inc: INTEGER,
    damage_inc: INTEGER,
    flee_inc: INTEGER,
    req_monster_lvel: INTEGER
);

/* All quests, run by individual users */
CREATE TABLE quests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    quest_status BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (user_id) REFERENCES users (id)
);
/* All instantiated monsters associated with individual quests */
CREATE TABLE quest_monsters (
    quest_id INTEGER NOT NULL,
    monster_id INTEGER NOT NULL,

    PRIMARY KEY (quest_id, monster_id),
    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (monster_id) REFERENCES monsters(id)
);
/* All instantiated monsters' item inventories */
CREATE TABLE quest_monster_items(
    quest_id INTEGER NOT NULL,
    monster_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,  

    PRIMARY KEY (quest_id, monster_id, item_id),
    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (monster_id) REFERENCES monsters(id),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
/* All instantiated monsters' spell inventories */
CREATE TABLE quest_monster_spells(
    quest_id INTEGER NOT NULL,
    monster_id INTEGER NOT NULL,
    spell_id INTEGER NOT NULL,  

    PRIMARY KEY (quest_id, monster_id, spell_id),
    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (monster_id) REFERENCES monsters(id),
    FOREIGN KEY (spell_id) REFERENCES spells(id)
);
/* All instantiated riddles associated with individual quests */
CREATE TABLE quest_riddles (
    quest_id INTEGER NOT NULL,
    riddle_id INTEGER NOT NULL,
    answered BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (quest_id, riddle_id), 
    FOREIGN KEY (quest_id) REFERENCES quests (id),
    FOREIGN KEY (riddle_id) REFERENCES riddles (id)
);
/* All users' item inventories */
CREATE TABLE user_items(
    user_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,

    PRIMARY KEY (user_id, item_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
/* All users' spell inventories */
CREATE TABLE user_spells(
    user_id INTEGER NOT NULL,
    spell_id INTEGER NOT NULL,

    PRIMARY KEY (user_id, spell_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (spell_id) REFERENCES spells(id)

);