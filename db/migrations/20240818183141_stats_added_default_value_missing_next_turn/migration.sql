-- RedefineTables
PRAGMA defer_foreign_keys=ON;
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_stats" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "health" INTEGER NOT NULL,
    "armor" INTEGER NOT NULL,
    "power" INTEGER NOT NULL DEFAULT 1,
    "missing_next_turn" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_stats" ("armor", "health", "id", "missing_next_turn", "power") SELECT "armor", "health", "id", "missing_next_turn", "power" FROM "stats";
DROP TABLE "stats";
ALTER TABLE "new_stats" RENAME TO "stats";
PRAGMA foreign_keys=ON;
PRAGMA defer_foreign_keys=OFF;
