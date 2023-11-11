/*
  Warnings:

  - You are about to drop the column `magicka` on the `stats` table. All the data in the column will be lost.
  - You are about to drop the column `reflex` on the `stats` table. All the data in the column will be lost.
  - You are about to drop the column `wisdom` on the `stats` table. All the data in the column will be lost.

*/
-- CreateTable
CREATE TABLE "user_items" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    "item_idx" INTEGER NOT NULL,
    CONSTRAINT "user_items_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "user_equipped_items" (
    "item_id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    CONSTRAINT "user_equipped_items_item_id_fkey" FOREIGN KEY ("item_id") REFERENCES "user_items" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_monster_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "quest_id" INTEGER NOT NULL,
    "stats_id" INTEGER NOT NULL,
    "monster_idx" INTEGER NOT NULL,
    "next_action" INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT "monster_states_quest_id_fkey" FOREIGN KEY ("quest_id") REFERENCES "quests" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "monster_states_stats_id_fkey" FOREIGN KEY ("stats_id") REFERENCES "stats" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_monster_states" ("id", "monster_idx", "quest_id", "stats_id") SELECT "id", "monster_idx", "quest_id", "stats_id" FROM "monster_states";
DROP TABLE "monster_states";
ALTER TABLE "new_monster_states" RENAME TO "monster_states";
CREATE UNIQUE INDEX "monster_states_quest_id_key" ON "monster_states"("quest_id");
CREATE UNIQUE INDEX "monster_states_stats_id_key" ON "monster_states"("stats_id");
CREATE TABLE "new_stats" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "health" INTEGER NOT NULL,
    "armor" INTEGER NOT NULL,
    "missing_next_turn" BOOLEAN NOT NULL
);
INSERT INTO "new_stats" ("armor", "health", "id", "missing_next_turn") SELECT "armor", "health", "id", "missing_next_turn" FROM "stats";
DROP TABLE "stats";
ALTER TABLE "new_stats" RENAME TO "stats";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;

-- CreateIndex
CREATE UNIQUE INDEX "user_equipped_items_item_id_key" ON "user_equipped_items"("item_id");
