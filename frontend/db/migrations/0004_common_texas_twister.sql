CREATE TABLE "Settings" (
	"key" text PRIMARY KEY NOT NULL,
	"value" text DEFAULT '' NOT NULL,
	"updatedAt" timestamp DEFAULT now()
);
