-- Migration to convert Users.id from bigint to text (UUID)
-- This will preserve existing data by converting IDs

-- Step 1: Drop foreign key constraints
ALTER TABLE "EventMembers" DROP CONSTRAINT IF EXISTS "EventMembers_userId_Users_id_fk";
ALTER TABLE "Events" DROP CONSTRAINT IF EXISTS "Events_createdBy_Users_id_fk";
ALTER TABLE "accounts" DROP CONSTRAINT IF EXISTS "accounts_userId_Users_id_fk";
ALTER TABLE "sessions" DROP CONSTRAINT IF EXISTS "sessions_userId_Users_id_fk";

-- Step 2: Convert all ID columns to text
ALTER TABLE "Users" ALTER COLUMN "id" SET DATA TYPE text USING "id"::text;
ALTER TABLE "EventMembers" ALTER COLUMN "userId" SET DATA TYPE text USING "userId"::text;
ALTER TABLE "Events" ALTER COLUMN "createdBy" SET DATA TYPE text USING "createdBy"::text;

-- Step 3: Add description and location to Events if not exists
ALTER TABLE "Events" ADD COLUMN IF NOT EXISTS "description" text;
ALTER TABLE "Events" ADD COLUMN IF NOT EXISTS "location" text;

-- Step 4: Recreate foreign key constraints
ALTER TABLE "EventMembers" ADD CONSTRAINT "EventMembers_userId_Users_id_fk"
  FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE no action ON UPDATE no action;

ALTER TABLE "Events" ADD CONSTRAINT "Events_createdBy_Users_id_fk"
  FOREIGN KEY ("createdBy") REFERENCES "public"."Users"("id") ON DELETE no action ON UPDATE no action;

ALTER TABLE "accounts" ADD CONSTRAINT "accounts_userId_Users_id_fk"
  FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE cascade ON UPDATE no action;

ALTER TABLE "sessions" ADD CONSTRAINT "sessions_userId_Users_id_fk"
  FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE cascade ON UPDATE no action;
