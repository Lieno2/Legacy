-- Settings table (key/value store for admin config)
CREATE TABLE IF NOT EXISTS "Settings" (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Audit log
CREATE TABLE IF NOT EXISTS "AuditLog" (
    id          BIGSERIAL PRIMARY KEY,
    actor_id    TEXT,
    actor_name  TEXT,
    action      TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   TEXT,
    entity_name TEXT,
    detail      TEXT,
    snapshot    JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
