import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { events, users, eventInvites } from "@/db/schema"
import { eq, and, or } from "drizzle-orm"
import { sendDiscordNotification } from "@/lib/discord"

async function getUserId(session: any): Promise<string | null> {
    if (session?.user?.id) return session.user.id
    if (session?.user?.email) {
        const user = await db.select().from(users).where(eq(users.email, session.user.email)).then(r => r[0])
        return user?.id || null
    }
    return null
}

// ── GET events visible to current user ────────────────────────────────────────
// Visible = public events + your own private events + private events you're invited to

export async function GET() {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        // Get event IDs the user is invited to
        const invitedEventIds = await db
            .select({ eventId: eventInvites.eventId })
            .from(eventInvites)
            .where(eq(eventInvites.userId, userId))
            .then(rows => rows.map(r => r.eventId))

        // Fetch all events: public ones + own private + invited private
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
            .where(
                or(
                    eq(events.private, false),           // all public events
                    eq(events.createdBy, userId),         // own private events
                    invitedEventIds.length > 0            // invited private events
                        ? events.id.in(invitedEventIds)
                        : undefined,
                )
            )
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

        const creator = await db.select({ username: users.username }).from(users).where(eq(users.id, userId)).then(r => r[0])

        // Only notify Discord for public events (private events are private)
        if (!newEvent.private) {
            await sendDiscordNotification("created", { ...newEvent, creatorName: creator?.username })
        }

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

        if (!updatedEvent.private) {
            await sendDiscordNotification("updated", { ...updatedEvent, creatorName: creator?.username })
        }

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

        const eventToDelete = await db
            .select()
            .from(events)
            .where(and(eq(events.id, Number(id)), eq(events.createdBy, userId)))
            .then(r => r[0])

        if (!eventToDelete) {
            return NextResponse.json({ error: "Event not found or unauthorized" }, { status: 404 })
        }

        await db.delete(events).where(eq(events.id, Number(id)))

        const creator = await db.select({ username: users.username }).from(users).where(eq(users.id, userId)).then(r => r[0])

        if (!eventToDelete.private) {
            await sendDiscordNotification("deleted", { ...eventToDelete, creatorName: creator?.username })
        }

        return NextResponse.json({ success: true })
    } catch (error) {
        console.error("Failed to delete event:", error)
        return NextResponse.json({ error: "Failed to delete event" }, { status: 500 })
    }
}