CREATE TABLE "EventMembers" (
	"eventId" bigint NOT NULL,
	"userId" bigint NOT NULL,
	"joinedAt" timestamp DEFAULT now(),
	CONSTRAINT "EventMembers_eventId_userId_pk" PRIMARY KEY("eventId","userId")
);
--> statement-breakpoint
CREATE TABLE "Events" (
	"id" bigserial PRIMARY KEY NOT NULL,
	"title" text NOT NULL,
	"date" timestamp NOT NULL,
	"createdBy" bigint NOT NULL,
	"createdAt" timestamp DEFAULT now(),
	"private" boolean DEFAULT false NOT NULL
);
--> statement-breakpoint
CREATE TABLE "Users" (
	"id" bigserial PRIMARY KEY NOT NULL,
	"username" text NOT NULL,
	"email" text NOT NULL,
	"passwordHash" text NOT NULL,
	"perms" smallint DEFAULT 0 NOT NULL,
	"createdAt" timestamp DEFAULT now(),
	CONSTRAINT "Users_username_unique" UNIQUE("username"),
	CONSTRAINT "Users_email_unique" UNIQUE("email")
);
--> statement-breakpoint
ALTER TABLE "EventMembers" ADD CONSTRAINT "EventMembers_eventId_Events_id_fk" FOREIGN KEY ("eventId") REFERENCES "public"."Events"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "EventMembers" ADD CONSTRAINT "EventMembers_userId_Users_id_fk" FOREIGN KEY ("userId") REFERENCES "public"."Users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Events" ADD CONSTRAINT "Events_createdBy_Users_id_fk" FOREIGN KEY ("createdBy") REFERENCES "public"."Users"("id") ON DELETE no action ON UPDATE no action;