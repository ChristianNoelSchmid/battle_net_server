/* All possible riddles in the game */
CREATE TABLE riddles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    answers TEXT NOT NULL
);
/* All possible monsters in the game */
CREATE TABLE monsters (
    id INTEGER PRIMARY KEY AUTOINCREMENT
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
    monster_id INTEGER NOT NULL

    PRIMARY KEY (quest_id, monster_id),
    FOREIGN KEY (quest_id) REFERENCES quests(id)
    FOREIGN KEY (monster_id) REFERENCES monsters(id)
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