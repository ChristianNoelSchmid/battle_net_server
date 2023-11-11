-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_game_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "murdered_user_id" INTEGER NOT NULL,
    "last_daily_refresh" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "game_states_murdered_user_id_fkey" FOREIGN KEY ("murdered_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
INSERT INTO "new_game_states" ("id", "murdered_user_id") SELECT "id", "murdered_user_id" FROM "game_states";
DROP TABLE "game_states";
ALTER TABLE "new_game_states" RENAME TO "game_states";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
