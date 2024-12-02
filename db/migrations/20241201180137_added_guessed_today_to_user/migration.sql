-- RedefineTables
PRAGMA defer_foreign_keys=ON;
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_users" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "email" TEXT NOT NULL,
    "pwd_hash" TEXT NOT NULL,
    "card_idx" INTEGER NOT NULL,
    "lvl" INTEGER NOT NULL DEFAULT 1,
    "riddle_quest_completed" BOOLEAN NOT NULL DEFAULT false,
    "exhausted" BOOLEAN NOT NULL DEFAULT false,
    "guessed_today" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_users" ("card_idx", "email", "exhausted", "id", "lvl", "pwd_hash", "riddle_quest_completed") SELECT "card_idx", "email", "exhausted", "id", "lvl", "pwd_hash", "riddle_quest_completed" FROM "users";
DROP TABLE "users";
ALTER TABLE "new_users" RENAME TO "users";
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");
PRAGMA foreign_keys=ON;
PRAGMA defer_foreign_keys=OFF;
