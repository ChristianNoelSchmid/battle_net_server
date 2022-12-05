/* All users in the given game */
CREATE TABLE users (
    id             INTEGER  PRIMARY KEY  AUTOINCREMENT,
    card_id        INTEGER  NOT NULL,
    user_name      TEXT     NOT NULL, 
    passwd         TEXT     NOT NULL,
    user_img_path  TEXT,
    last_login     DATETIME DEFAULT NOW(),

    FOREIGN KEY (card_id) REFERENCES cards(id)
);
/* All card categories for the game running in the database */
CREATE TABLE categories (
    id        INTEGER  PRIMARY KEY  AUTOINCREMENT, 
    cat_name  TEXT     NOT NULL
);
/* All evidence cards for the game running in the database */
CREATE TABLE evidence_cards (
    id             INTEGER  PRIMARY KEY AUTOINCREMENT,
    cat_id         INT      NOT NULL,
    item_name      TEXT     NOT NULL,
    item_img_path  TEXT,

    FOREIGN KEY (cat_id) REFERENCES categories(id)
);
/* All evidence cards individual users have discovered,
   or are marking as potential */
CREATE TABLE user_evidence_cards (
    user_id    INT      NOT NULL,
    card_id    INT      NOT NULL,
    confirmed  BOOLEAN,

    PRIMARY KEY (user_id, card_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (card_id) REFERENCES cards(id)
);
/* The game state - a singleton table */
CREATE TABLE game_state (
    murdered_user_id  INTEGER  NOT NULL,
    game_target_cards TEXT NOT NULL,
    
    FOREIGN KEY (murdered_user_id) REFERENCES users(id)
);
/* All users who have won the game */
CREATE TABLE winners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_state_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,

    FOREIGN KEY (game_state_id) REFERENCES game_states (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);