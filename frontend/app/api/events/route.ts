import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { events, users } from "@/db/schema"
import { eq, and, or } from "drizzle-orm"

// ── Helpers ───────────────────────────────────────────────────────────────────

async function getUserId(session: any): Promise<string | null> {
    if (session?.user?.id) return session.user.id
    if (session?.user?.email) {
        const user = await db.select().from(users).where(eq(users.email, session.user.email)).then(r => r[0])
        return user?.id || null
    }
    return null
}

async function sendDiscordNotification(type: "created" | "updated" | "deleted", event: any) {
    const webhookUrl = process.env.DISCORD_WEBHOOK_URL
    if (!webhookUrl) return

    const colors = { created: 0x10b981, updated: 0xf59e0b, deleted: 0xf43f5e }
    const emojis = { created: "📅", updated: "✏️", deleted: "🗑️" }
    const titles = { created: "New Event Created", updated: "Event Updated", deleted: "Event Deleted" }

    const embed: any = {
        title: `${emojis[type]} ${titles[type]}`,
        color: colors[type],
        timestamp: new Date().toISOString(),
        fields: [
            { name: "Event", value: event.title, inline: true },
            { name: "By", value: event.creatorName || "Unknown", inline: true },
        ],
    }

    if (type !== "deleted") {
        const date = new Date(event.date)
        embed.fields.push({ name: "Date", value: date.toLocaleDateString("en-GB", { weekday: "long", year: "numeric", month: "long", day: "numeric" }), inline: false })
        if (event.location) embed.fields.push({ name: "Location", value: event.location, inline: true })
        if (event.description) embed.fields.push({ name: "Description", value: event.description.slice(0, 200), inline: false })
        if (event.private) embed.fields.push({ name: "Visibility", value: "🔒 Private", inline: true })
    }

    try {
        await fetch(webhookUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ embeds: [embed] }),
        })
    } catch (e) {
        console.error("Discord webhook failed:", e)
    }
}

// ── GET all visible events ────────────────────────────────────────────────────
// Returns: all public events from any user + the current user's own private events

export async function GET() {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const allEvents = await db
            .select({
                id: events.id,
                title: events.title,
                description: events.description,
                date: events.date,
                location: events.location,
                color: events.color,
                createdBy: events.createdBy,
                createdAt: events.createdAt,
                private: events.private,
                creatorName: users.username,
            })
            .from(events)
            .leftJoin(users, eq(events.createdBy, users.id))
            // Show all public events + your own private ones
            .where(or(eq(events.private, false), eq(events.createdBy, userId)))
            .orderBy(events.date)

        return NextResponse.json(allEvents)
    } catch (error) {
        console.error("Failed to fetch events:", error)
        return NextResponse.json({ error: "Failed to fetch events" }, { status: 500 })
    }
}

// ── POST create event ─────────────────────────────────────────────────────────

export async function POST(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const body = await req.json()
        const { title, description, date, location, color, private: isPrivate } = body

        if (!title || !date) {
            return NextResponse.json({ error: "Title and date are required" }, { status: 400 })
        }

        const [newEvent] = await db
            .insert(events)
            .values({
                title,
                description: description || null,
                date: new Date(date),
                location: location || null,
                color: color || null,
                createdBy: userId,
                private: isPrivate ?? false,
            })
            .returning()

        // Fetch creator name for Discord
        const creator = await db.select({ username: users.username }).from(users).where(eq(users.id, userId)).then(r => r[0])
        await sendDiscordNotification("created", { ...newEvent, creatorName: creator?.username })

        return NextResponse.json(newEvent, { status: 201 })
    } catch (error) {
        console.error("Failed to create event:", error)
        return NextResponse.json({ error: "Failed to create event" }, { status: 500 })
    }
}

// ── PUT update event ──────────────────────────────────────────────────────────

export async function PUT(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const body = await req.json()
        const { id, title, description, date, location, color, private: isPrivate } = body

        if (!id || !title || !date) {
            return NextResponse.json({ error: "ID, title and date are required" }, { status: 400 })
        }

        const [updatedEvent] = await db
            .update(events)
            .set({
                title,
                description: description || null,
                date: new Date(date),
                location: location || null,
                color: color || null,
                private: isPrivate,
            })
            .where(and(eq(events.id, id), eq(events.createdBy, userId)))
            .returning()

        if (!updatedEvent) {
            return NextResponse.json({ error: "Event not found or unauthorized" }, { status: 404 })
        }

        const creator = await db.select({ username: users.username }).from(users).where(eq(users.id, userId)).then(r => r[0])
        await sendDiscordNotification("updated", { ...updatedEvent, creatorName: creator?.username })

        return NextResponse.json(updatedEvent)
    } catch (error) {
        console.error("Failed to update event:", error)
        return NextResponse.json({ error: "Failed to update event" }, { status: 500 })
    }
}

// ── DELETE event ──────────────────────────────────────────────────────────────

export async function DELETE(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const { searchParams } = new URL(req.url)
        const id = searchParams.get("id")
        if (!id) return NextResponse.json({ error: "Event ID is required" }, { status: 400 })

        // Fetch event before deleting for Discord notification
        const eventToDelete = await db
            .select({ id: events.id, title: events.title, createdBy: events.createdBy })
            .from(events)
            .where(and(eq(events.id, Number(id)), eq(events.createdBy, userId)))
            .then(r => r[0])

        if (!eventToDelete) {
            return NextResponse.json({ error: "Event not found or unauthorized" }, { status: 404 })
        }

        await db.delete(events).where(eq(events.id, Number(id)))

        const creator = await db.select({ username: users.username }).from(users).where(eq(users.id, userId)).then(r => r[0])
        await sendDiscordNotification("deleted", { ...eventToDelete, creatorName: creator?.username })

        return NextResponse.json({ success: true })
    } catch (error) {
        console.error("Failed to delete event:", error)
        return NextResponse.json({ error: "Failed to delete event" }, { status: 500 })
    }
}