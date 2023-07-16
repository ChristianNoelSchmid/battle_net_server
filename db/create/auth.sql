CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    email TEXT NOT NULL,
    pwd_hash TEXT NOT NULL,
    card_idx INTEGER NOT NULL
);

CREATE TABLE refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    replacement_id INTEGER,
    created_on DATETIME NOT NULL,
    expires DATETIME,
    token TEXT NOT NULL,
    revoked_on DATETIME,
    revoked_by TEXT,

    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (replacement_id) REFERENCES refresh_tokens(id)
);

CREATE INDEX token_idx ON refresh_tokens(token);