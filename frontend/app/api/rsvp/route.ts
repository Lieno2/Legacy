import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { eventMembers, events, users } from "@/db/schema"
import { eq, and } from "drizzle-orm"

async function getUserId(session: any): Promise<string | null> {
    if (session?.user?.id) return session.user.id
    if (session?.user?.email) {
        const user = await db.select().from(users).where(eq(users.email, session.user.email)).then(r => r[0])
        return user?.id || null
    }
    return null
}

// GET members of an event
export async function GET(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    const { searchParams } = new URL(req.url)
    const eventId = searchParams.get("eventId")
    if (!eventId) return NextResponse.json({ error: "eventId required" }, { status: 400 })

    try {
        const members = await db
            .select({
                userId: eventMembers.userId,
                username: users.username,
                status: eventMembers.status,
                lateMinutes: eventMembers.lateMinutes,
                joinedAt: eventMembers.joinedAt,
            })
            .from(eventMembers)
            .leftJoin(users, eq(eventMembers.userId, users.id))
            .where(eq(eventMembers.eventId, Number(eventId)))

        return NextResponse.json(members)
    } catch (error) {
        return NextResponse.json({ error: "Failed to fetch members" }, { status: 500 })
    }
}

// POST / upsert RSVP
export async function POST(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const body = await req.json()
        const { eventId, status, lateMinutes } = body

        if (!eventId || !status) {
            return NextResponse.json({ error: "eventId and status required" }, { status: 400 })
        }
        if (!["going", "late", "not_going"].includes(status)) {
            return NextResponse.json({ error: "Invalid status" }, { status: 400 })
        }

        // Check event exists
        const event = await db.select().from(events).where(eq(events.id, Number(eventId))).then(r => r[0])
        if (!event) return NextResponse.json({ error: "Event not found" }, { status: 404 })

        // Upsert
        const existing = await db
            .select()
            .from(eventMembers)
            .where(and(eq(eventMembers.eventId, Number(eventId)), eq(eventMembers.userId, userId)))
            .then(r => r[0])

        if (existing) {
            await db
                .update(eventMembers)
                .set({ status, lateMinutes: status === "late" ? (lateMinutes ?? null) : null })
                .where(and(eq(eventMembers.eventId, Number(eventId)), eq(eventMembers.userId, userId)))
        } else {
            await db.insert(eventMembers).values({
                eventId: Number(eventId),
                userId,
                status,
                lateMinutes: status === "late" ? (lateMinutes ?? null) : null,
            })
        }

        return NextResponse.json({ success: true })
    } catch (error) {
        console.error("RSVP error:", error)
        return NextResponse.json({ error: "Failed to RSVP" }, { status: 500 })
    }
}

// DELETE remove RSVP
export async function DELETE(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    const { searchParams } = new URL(req.url)
    const eventId = searchParams.get("eventId")
    if (!eventId) return NextResponse.json({ error: "eventId required" }, { status: 400 })

    try {
        await db
            .delete(eventMembers)
            .where(and(eq(eventMembers.eventId, Number(eventId)), eq(eventMembers.userId, userId)))
        return NextResponse.json({ success: true })
    } catch (error) {
        return NextResponse.json({ error: "Failed to remove RSVP" }, { status: 500 })
    }
}