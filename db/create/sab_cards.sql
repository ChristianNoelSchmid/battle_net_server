/* All applied sabotage cards: who applied it,
 who it's applied to, and the card that's applied */
CREATE TABLE applied_sabotage_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    from_user_id INTEGER NOT NULL,
    to_user_id INTEGER NOT NULL,
    card_applied_idx INTEGER NOT NULL,
    FOREIGN KEY (from_user_id) REFERENCES users(id),
    FOREIGN KEY (to_user_id) REFERENCES users(id)
);