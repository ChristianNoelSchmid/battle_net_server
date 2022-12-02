CREATE TABLE users (
    id             INTEGER  PRIMARY KEY  AUTOINCREMENT,
    card_id        INTEGER  NOT NULL,
    user_name      TEXT     NOT NULL, 
    passwd         TEXT     NOT NULL,

    user_img_path  TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);
CREATE TABLE categories (
    id        INTEGER  PRIMARY KEY  AUTOINCREMENT, 
    cat_name  TEXT     NOT NULL
);
CREATE TABLE cards (
    id             INTEGER  PRIMARY KEY AUTOINCREMENT,
    cat_id         INT      NOT NULL,
    item_name      TEXT     NOT NULL,
    item_img_path  TEXT,

    FOREIGN KEY (cat_id) REFERENCES categories(id)
);
CREATE TABLE user_found_cards (
    user_id    INT      NOT NULL,
    card_id    INT      NOT NULL,
    confirmed  BOOLEAN,

    PRIMARY KEY (user_id, card_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (card_id) REFERENCES cards(id)
);
CREATE TABLE game_states (
    id                INTEGER  PRIMARY KEY AUTOINCREMENT,
    murdered_user_id  INTEGER  NOT NULL,
    
    FOREIGN KEY (murdered_user_id) REFERENCES users(id)
);

CREATE TABLE game_target_cards (
    game_state_id  INTEGER NOT NULL,
    card_id        INTEGER NOT NULL,

    PRIMARY KEY (game_state_id, card_id),
    FOREIGN KEY (game_state_id) REFERENCES game_states(id),
    FOREIGN KEY (card_id) REFERENCES cards(id)
);
CREATE TABLE riddles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    answers TEXT NOT NULL
);
CREATE TABLE active_riddles (
    user_id INTEGER NOT NULL,
    riddle_id INTEGER NOT NULL,
    answered BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (user_id, riddle_id), 
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (riddle_id) REFERENCES riddles (id)
);