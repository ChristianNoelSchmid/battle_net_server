CREATE TABLE stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    health INTEGER NOT NULL,
    magicka INTEGER NOT NULL,
    armor INTEGER NOT NULL,
    wisdom INTEGER NOT NULL,
    reflex INTEGER NOT NULL,
    missing_next_turn BOOLEAN DEFAULT FALSE
);

/* All users in the given game */
CREATE TABLE user_state (
    user_id INTEGER NOT NULL,
    last_login_dt DATETIME DEFAULT NOW,
    cur_stats_id INTEGER NOT NULL,

    FOREIGN KEY (cur_stats_id) REFERENCES stats(id)
);

CREATE TABLE monster_state (
    quest_id INTEGER NOT NULL,
    cur_stats_id INTEGER NOT NULL,

    PRIMARY KEY (quest_id, cur_stats_id),
    FOREIGN KEY (quest_id) REFERENCES quests(id),
    FOREIGN KEY (cur_stats_id) REFERENCES stats(id)
);

/* All evidence cards individual users have discovered,
 or are marking as potential */
CREATE TABLE user_evidence_cards (
    user_id INT NOT NULL,
    cat_idx INT NOT NULL,
    card_idx INT NOT NULL,
    confirmed BOOLEAN NOT NULL,

    PRIMARY KEY (user_id, cat_idx, card_idx),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

/* The game state - a singleton table */
CREATE TABLE game_state (murdered_user_id INTEGER NOT NULL);

CREATE TABLE game_target_cards (
    cat_idx INTEGER NOT NULL,
    card_idx INTEGER NOT NULL,
    PRIMARY KEY (cat_idx, card_idx)
);

CREATE TABLE game_winners (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id)
);