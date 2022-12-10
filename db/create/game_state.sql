CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    passwd TEXT NOT NULL,
    card_idx INTEGER NOT NULL
);
CREATE TABLE stats (
    health INTEGER NOT NULL,
    magicka INTEGER NOT NULL,
    flee_from_changes INTEGER
);
/* All users in the given game */
CREATE TABLE users_state (
    last_login     DATETIME DEFAULT NOW,

    base_stats_id  INTEGER  NOT NULL, 
    stats          INTEGER NOT NULL
);
/* All evidence cards individual users have discovered,
   or are marking as potential */
CREATE TABLE user_evidence_cards (
    user_id    INT      NOT NULL,
    cat_idx    TEXT     NOT NULL,
    card_idx   INT      NOT NULL,
    confirmed  BOOLEAN,

    PRIMARY KEY (user_id, cat_idx, card_idx),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
/* The game state - a singleton table */
CREATE TABLE game_state (
    murdered_user_id   INTEGER  NOT NULL,
    target_card_idxs   TEXT     NOT NULL,
    winners            TEXT     NOT NULL  
);
/* All users who have won the game */
CREATE TABLE winners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_idx INTEGER NOT NULL
);