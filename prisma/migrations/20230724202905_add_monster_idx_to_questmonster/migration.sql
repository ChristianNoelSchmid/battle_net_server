/*
  Warnings:

  - Added the required column `monster_idx` to the `monster_states` table without a default value. This is not possible if the table is not empty.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_monster_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "quest_id" INTEGER NOT NULL,
    "stats_id" INTEGER NOT NULL,
    "monster_idx" INTEGER NOT NULL,
    CONSTRAINT "monster_states_quest_id_fkey" FOREIGN KEY ("quest_id") REFERENCES "quests" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "monster_states_stats_id_fkey" FOREIGN KEY ("stats_id") REFERENCES "stats" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_monster_states" ("id", "quest_id", "stats_id") SELECT "id", "quest_id", "stats_id" FROM "monster_states";
DROP TABLE "monster_states";
ALTER TABLE "new_monster_states" RENAME TO "monster_states";
CREATE UNIQUE INDEX "monster_states_quest_id_key" ON "monster_states"("quest_id");
CREATE UNIQUE INDEX "monster_states_stats_id_key" ON "monster_states"("stats_id");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
