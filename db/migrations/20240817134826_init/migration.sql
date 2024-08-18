-- CreateTable
CREATE TABLE "users" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "email" TEXT NOT NULL,
    "pwd_hash" TEXT NOT NULL,
    "card_idx" INTEGER NOT NULL,
    "lvl" INTEGER NOT NULL DEFAULT 1,
    "riddle_quest_completed" BOOLEAN NOT NULL DEFAULT false,
    "exhausted" BOOLEAN NOT NULL DEFAULT false
);

-- CreateTable
CREATE TABLE "refresh_tokens" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    "replacement_id" INTEGER,
    "created_on" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expires" DATETIME,
    "token" TEXT NOT NULL,
    "revoked_on" DATETIME,
    "revoked_by" TEXT,
    CONSTRAINT "refresh_tokens_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "refresh_tokens_replacement_id_fkey" FOREIGN KEY ("replacement_id") REFERENCES "refresh_tokens" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "stats" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "health" INTEGER NOT NULL,
    "armor" INTEGER NOT NULL,
    "power" INTEGER NOT NULL DEFAULT 1,
    "missing_next_turn" BOOLEAN NOT NULL
);

-- CreateTable
CREATE TABLE "user_states" (
    "user_id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "last_login" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "stats_id" INTEGER NOT NULL,
    CONSTRAINT "user_states_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "user_states_stats_id_fkey" FOREIGN KEY ("stats_id") REFERENCES "stats" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "user_cards" (
    "user_id" INTEGER NOT NULL,
    "cat_idx" INTEGER NOT NULL,
    "card_idx" INTEGER NOT NULL,
    "confirmed" BOOLEAN NOT NULL DEFAULT false,

    PRIMARY KEY ("user_id", "cat_idx", "card_idx"),
    CONSTRAINT "user_cards_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "game_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "murdered_user_id" INTEGER NOT NULL,
    "last_daily_refresh" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "game_states_murdered_user_id_fkey" FOREIGN KEY ("murdered_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "quests" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    "completed" BOOLEAN NOT NULL DEFAULT false,
    "created_on" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "quest_type" INTEGER NOT NULL,
    CONSTRAINT "quests_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "monster_states" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "quest_id" INTEGER NOT NULL,
    "stats_id" INTEGER NOT NULL,
    "monster_idx" INTEGER NOT NULL,
    "next_action" INTEGER,
    "action_flv_text" TEXT,
    CONSTRAINT "monster_states_quest_id_fkey" FOREIGN KEY ("quest_id") REFERENCES "quests" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "monster_states_stats_id_fkey" FOREIGN KEY ("stats_id") REFERENCES "stats" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "quest_riddles" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "quest_id" INTEGER NOT NULL,
    "riddle_idx" INTEGER NOT NULL,
    CONSTRAINT "quest_riddles_quest_id_fkey" FOREIGN KEY ("quest_id") REFERENCES "quests" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "game_target_cards" (
    "cat_idx" INTEGER NOT NULL,
    "card_idx" INTEGER NOT NULL,

    PRIMARY KEY ("cat_idx", "card_idx")
);

-- CreateTable
CREATE TABLE "game_winners" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    CONSTRAINT "game_winners_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

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

-- CreateIndex
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");

-- CreateIndex
CREATE UNIQUE INDEX "refresh_tokens_replacement_id_key" ON "refresh_tokens"("replacement_id");

-- CreateIndex
CREATE INDEX "refresh_tokens_token_idx" ON "refresh_tokens"("token");

-- CreateIndex
CREATE UNIQUE INDEX "user_states_user_id_key" ON "user_states"("user_id");

-- CreateIndex
CREATE UNIQUE INDEX "user_states_stats_id_key" ON "user_states"("stats_id");

-- CreateIndex
CREATE UNIQUE INDEX "monster_states_quest_id_key" ON "monster_states"("quest_id");

-- CreateIndex
CREATE UNIQUE INDEX "monster_states_stats_id_key" ON "monster_states"("stats_id");

-- CreateIndex
CREATE UNIQUE INDEX "quest_riddles_quest_id_key" ON "quest_riddles"("quest_id");

-- CreateIndex
CREATE UNIQUE INDEX "game_winners_user_id_key" ON "game_winners"("user_id");

-- CreateIndex
CREATE UNIQUE INDEX "user_equipped_items_item_id_key" ON "user_equipped_items"("item_id");
