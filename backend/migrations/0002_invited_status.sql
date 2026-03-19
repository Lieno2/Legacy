-- Allow 'invited' as a valid status in EventMembers (no status constraint exists, just documenting)
-- Also ensure the status column has no NOT NULL default that blocks invited rows
ALTER TABLE "EventMembers" ALTER COLUMN status SET DEFAULT 'invited';
