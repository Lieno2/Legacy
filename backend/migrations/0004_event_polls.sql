-- Poll definition (one per event)
CREATE TABLE IF NOT EXISTS "EventPolls" (
  id          BIGSERIAL PRIMARY KEY,
  "eventId"   BIGINT NOT NULL REFERENCES "Events"(id) ON DELETE CASCADE,
  question    TEXT NOT NULL,
  "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE ("eventId")
);

-- Choices for a poll
CREATE TABLE IF NOT EXISTS "EventPollChoices" (
  id       BIGSERIAL PRIMARY KEY,
  "pollId" BIGINT NOT NULL REFERENCES "EventPolls"(id) ON DELETE CASCADE,
  label    TEXT NOT NULL,
  position INT  NOT NULL DEFAULT 0
);

-- One answer per user per poll
CREATE TABLE IF NOT EXISTS "EventPollAnswers" (
  "pollId"     BIGINT NOT NULL REFERENCES "EventPolls"(id)       ON DELETE CASCADE,
  "userId"     TEXT   NOT NULL REFERENCES "Users"(id)             ON DELETE CASCADE,
  "choiceId"   BIGINT NOT NULL REFERENCES "EventPollChoices"(id)  ON DELETE CASCADE,
  "answeredAt" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY ("pollId", "userId")
);
