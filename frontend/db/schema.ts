import {
    pgTable, bigserial, text, smallint, boolean,
    timestamp, bigint, primaryKey, integer
} from "drizzle-orm/pg-core"

export const users = pgTable("Users", {
    id: text("id").primaryKey().$defaultFn(() => crypto.randomUUID()),
    username: text("username").notNull().unique(),
    email: text("email").notNull().unique(),
    passwordHash: text("passwordHash").notNull(),
    perms: smallint("perms").notNull().default(0),
    createdAt: timestamp("createdAt").defaultNow(),
})

export const events = pgTable("Events", {
    id: bigserial("id", { mode: "number" }).primaryKey(),
    title: text("title").notNull(),
    description: text("description"),
    date: timestamp("date").notNull(),
    location: text("location"),
    color: text("color"),
    createdBy: text("createdBy").notNull().references(() => users.id),
    createdAt: timestamp("createdAt").defaultNow(),
    private: boolean("private").notNull().default(false),
})

export const eventMembers = pgTable("EventMembers", {
    eventId: bigint("eventId", { mode: "number" }).notNull().references(() => events.id, { onDelete: "cascade" }),
    userId: text("userId").notNull().references(() => users.id, { onDelete: "cascade" }),
    status: text("status").notNull().default("going"),  // "going" | "late" | "not_going"
    lateMinutes: integer("lateMinutes"),
    joinedAt: timestamp("joinedAt").defaultNow(),
}, (t) => ({
    pk: primaryKey({ columns: [t.eventId, t.userId] }),
}))

// Key-value store for app settings (Discord webhook URL, etc.)
export const settings = pgTable("Settings", {
    key: text("key").primaryKey(),
    value: text("value").notNull().default(""),
    updatedAt: timestamp("updatedAt").defaultNow(),
})

// ── Auth.js tables ────────────────────────────────────────────────────────────

export const accounts = pgTable("accounts", {
    userId: text("userId").notNull().references(() => users.id, { onDelete: "cascade" }),
    type: text("type").notNull(),
    provider: text("provider").notNull(),
    providerAccountId: text("providerAccountId").notNull(),
    refresh_token: text("refresh_token"),
    access_token: text("access_token"),
    expires_at: integer("expires_at"),
    token_type: text("token_type"),
    scope: text("scope"),
    id_token: text("id_token"),
    session_state: text("session_state"),
}, (t) => ({
    pk: primaryKey({ columns: [t.provider, t.providerAccountId] }),
}))

export const sessions = pgTable("sessions", {
    sessionToken: text("sessionToken").primaryKey(),
    userId: text("userId").notNull().references(() => users.id, { onDelete: "cascade" }),
    expires: timestamp("expires").notNull(),
})

export const verificationTokens = pgTable("verificationTokens", {
    identifier: text("identifier").notNull(),
    token: text("token").notNull(),
    expires: timestamp("expires").notNull(),
}, (t) => ({
    pk: primaryKey({ columns: [t.identifier, t.token] }),
}))