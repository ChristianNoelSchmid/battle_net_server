/*
  Warnings:

  - You are about to drop the column `replacement_id` on the `refresh_tokens` table. All the data in the column will be lost.

*/
-- RedefineTables
PRAGMA defer_foreign_keys=ON;
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_refresh_tokens" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    "repl_id" INTEGER,
    "created_on" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expires" DATETIME,
    "token" TEXT NOT NULL,
    "revoked_on" DATETIME,
    "revoked_by" TEXT,
    CONSTRAINT "refresh_tokens_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "refresh_tokens_repl_id_fkey" FOREIGN KEY ("repl_id") REFERENCES "refresh_tokens" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_refresh_tokens" ("created_on", "expires", "id", "revoked_by", "revoked_on", "token", "user_id") SELECT "created_on", "expires", "id", "revoked_by", "revoked_on", "token", "user_id" FROM "refresh_tokens";
DROP TABLE "refresh_tokens";
ALTER TABLE "new_refresh_tokens" RENAME TO "refresh_tokens";
CREATE UNIQUE INDEX "refresh_tokens_repl_id_key" ON "refresh_tokens"("repl_id");
CREATE INDEX "refresh_tokens_token_idx" ON "refresh_tokens"("token");
PRAGMA foreign_keys=ON;
PRAGMA defer_foreign_keys=OFF;
