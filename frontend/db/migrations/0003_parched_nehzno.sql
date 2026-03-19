ALTER TABLE "EventMembers" DROP CONSTRAINT "EventMembers_eventId_Events_id_fk";
--> statement-breakpoint
ALTER TABLE "EventMembers" DROP CONSTRAINT "EventMembers_userId_Users_id_fk";
--> statement-breakpoint
ALTER TABLE "EventMembers" ADD COLUMN "status" text DEFAULT 'going' NOT NULL;--> statement-breakpoint
ALTER TABLE "EventMembers" ADD COLUMN "lateMinutes" integer;--> statement-breakpoint
ALTER TABLE "EventMembers" ADD CONSTRAINT "EventMembers_eventId_Events_id_fk" FOREIGN KEY ("eventId") REFERENCES "public"."Events"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "EventMembers" ADD CONSTRAINT "EventMembers_userId_Users_id_fk" FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE cascade ON UPDATE no action;