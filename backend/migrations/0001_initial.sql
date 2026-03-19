-- Users
CREATE TABLE IF NOT EXISTS "Users" (
    id              TEXT PRIMARY KEY,
    username        TEXT NOT NULL UNIQUE,
    email           TEXT NOT NULL UNIQUE,
    "passwordHash"  TEXT NOT NULL,
    perms           SMALLINT NOT NULL DEFAULT 0,
    "createdAt"     TIMESTAMPTZ DEFAULT NOW()
);

-- Events
CREATE TABLE IF NOT EXISTS "Events" (
    id              BIGSERIAL PRIMARY KEY,
    title           TEXT NOT NULL,
    description     TEXT,
    date            TIMESTAMPTZ NOT NULL,
    location        TEXT,
    color           TEXT,
    "createdBy"     TEXT NOT NULL REFERENCES "Users"(id) ON DELETE CASCADE,
    "createdAt"     TIMESTAMPTZ DEFAULT NOW(),
    private         BOOLEAN NOT NULL DEFAULT FALSE
);

-- Event Members
CREATE TABLE IF NOT EXISTS "EventMembers" (
    "eventId"       BIGINT NOT NULL REFERENCES "Events"(id) ON DELETE CASCADE,
    "userId"        TEXT NOT NULL REFERENCES "Users"(id) ON DELETE CASCADE,
    username        TEXT,
    status          TEXT NOT NULL DEFAULT 'going',
    "lateMinutes"   INT,
    "joinedAt"      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY ("eventId", "userId")
);
