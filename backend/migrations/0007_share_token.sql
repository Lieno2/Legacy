-- Add share_token to Events for public share links
ALTER TABLE "Events"
    ADD COLUMN IF NOT EXISTS share_token TEXT UNIQUE;

-- Backfill existing events with a random token
UPDATE "Events"
SET share_token = gen_random_uuid()::text
WHERE share_token IS NULL;