CREATE TABLE "EventInvites" (
	"eventId" bigint NOT NULL,
	"userId" text NOT NULL,
	"invitedAt" timestamp DEFAULT now(),
	CONSTRAINT "EventInvites_eventId_userId_pk" PRIMARY KEY("eventId","userId")
);
--> statement-breakpoint
ALTER TABLE "EventInvites" ADD CONSTRAINT "EventInvites_eventId_Events_id_fk" FOREIGN KEY ("eventId") REFERENCES "public"."Events"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "EventInvites" ADD CONSTRAINT "EventInvites_userId_Users_id_fk" FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE cascade ON UPDATE no action;