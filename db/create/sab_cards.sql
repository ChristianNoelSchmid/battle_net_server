/* A sabotage card, which players can play on other players 
   or that players can have applied to them on quest fail */
CREATE TABLE sabotage_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT
);

/* All applied sabotage cards: who applied it,
   who it's applied to, and the card that's applied */
CREATE TABLE applied_sabotage_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    from_user_id INTEGER NOT NULL,
    to_user_id INTEGER NOT NULL,
    card_applied_id INTEGER NOT NULL,

    FOREIGN KEY (from_user_id) REFERENCES users(id),
    FOREIGN KEY (to_user_id) REFERENCES users(id),
    FOREIGN KEY (card_applied_id) REFERENCES sabotage_cards(id)
);