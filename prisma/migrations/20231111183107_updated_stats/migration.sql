-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_stats" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "health" INTEGER NOT NULL,
    "armor" INTEGER NOT NULL,
    "power" INTEGER NOT NULL DEFAULT 1,
    "missing_next_turn" BOOLEAN NOT NULL
);
INSERT INTO "new_stats" ("armor", "health", "id", "missing_next_turn") SELECT "armor", "health", "id", "missing_next_turn" FROM "stats";
DROP TABLE "stats";
ALTER TABLE "new_stats" RENAME TO "stats";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
