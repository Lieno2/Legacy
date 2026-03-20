-- Add poll_type to EventPolls
-- Types: 'choice' (default, existing) | 'text' | 'rating' | 'yesno' | 'date'
ALTER TABLE "EventPolls"
    ADD COLUMN IF NOT EXISTS poll_type TEXT NOT NULL DEFAULT 'choice';

-- Text answers (for 'text' type polls)
CREATE TABLE IF NOT EXISTS "EventPollTextAnswers" (
    "pollId"     BIGINT NOT NULL REFERENCES "EventPolls"(id) ON DELETE CASCADE,
    "userId"     TEXT   NOT NULL REFERENCES "Users"(id)      ON DELETE CASCADE,
    answer       TEXT   NOT NULL,
    "answeredAt" TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY ("pollId", "userId")
);

-- Rating answers (for 'rating' type polls, value 1-5)
CREATE TABLE IF NOT EXISTS "EventPollRatings" (
    "pollId"     BIGINT NOT NULL REFERENCES "EventPolls"(id) ON DELETE CASCADE,
    "userId"     TEXT   NOT NULL REFERENCES "Users"(id)      ON DELETE CASCADE,
    rating       SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    "answeredAt" TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY ("pollId", "userId")
);

-- Poll templates
CREATE TABLE IF NOT EXISTS "PollTemplates" (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT    NOT NULL,
    poll_type    TEXT    NOT NULL DEFAULT 'choice',
    question     TEXT,
    choices      JSONB,           -- [{label: string}] for choice/yesno/date types
    allow_multiple BOOLEAN DEFAULT false,
    global       BOOLEAN DEFAULT false,  -- true = visible to all users (admin only)
    "createdBy"  TEXT REFERENCES "Users"(id) ON DELETE CASCADE,
    "createdAt"  TIMESTAMPTZ DEFAULT NOW()
);
