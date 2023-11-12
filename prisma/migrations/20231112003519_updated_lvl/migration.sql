-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_monster_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "quest_id" INTEGER NOT NULL,
    "stats_id" INTEGER NOT NULL,
    "monster_idx" INTEGER NOT NULL,
    "next_action" INTEGER,
    "action_flv_text" TEXT,
    CONSTRAINT "monster_states_quest_id_fkey" FOREIGN KEY ("quest_id") REFERENCES "quests" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "monster_states_stats_id_fkey" FOREIGN KEY ("stats_id") REFERENCES "stats" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_monster_states" ("id", "monster_idx", "next_action", "quest_id", "stats_id") SELECT "id", "monster_idx", "next_action", "quest_id", "stats_id" FROM "monster_states";
DROP TABLE "monster_states";
ALTER TABLE "new_monster_states" RENAME TO "monster_states";
CREATE UNIQUE INDEX "monster_states_quest_id_key" ON "monster_states"("quest_id");
CREATE UNIQUE INDEX "monster_states_stats_id_key" ON "monster_states"("stats_id");
CREATE TABLE "new_users" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "email" TEXT NOT NULL,
    "pwd_hash" TEXT NOT NULL,
    "card_idx" INTEGER NOT NULL,
    "lvl" INTEGER NOT NULL DEFAULT 1
);
INSERT INTO "new_users" ("card_idx", "email", "id", "pwd_hash") SELECT "card_idx", "email", "id", "pwd_hash" FROM "users";
DROP TABLE "users";
ALTER TABLE "new_users" RENAME TO "users";
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
