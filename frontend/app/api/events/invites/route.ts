import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { eventInvites, events, users } from "@/db/schema"
import { eq, and } from "drizzle-orm"

async function getUserId(session: any): Promise<string | null> {
    if (session?.user?.id) return session.user.id
    if (session?.user?.email) {
        const user = await db.select().from(users).where(eq(users.email, session.user.email)).then(r => r[0])
        return user?.id || null
    }
    return null
}

// GET invites for an event (only the creator can see the full list)
export async function GET(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    const { searchParams } = new URL(req.url)
    const eventId = searchParams.get("eventId")
    if (!eventId) return NextResponse.json({ error: "eventId required" }, { status: 400 })

    // Verify requester is the event creator
    const event = await db.select().from(events).where(eq(events.id, Number(eventId))).then(r => r[0])
    if (!event) return NextResponse.json({ error: "Event not found" }, { status: 404 })
    if (event.createdBy !== userId) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const invites = await db
        .select({
            userId: eventInvites.userId,
            username: users.username,
            email: users.email,
            invitedAt: eventInvites.invitedAt,
        })
        .from(eventInvites)
        .leftJoin(users, eq(eventInvites.userId, users.id))
        .where(eq(eventInvites.eventId, Number(eventId)))

    return NextResponse.json(invites)
}

// POST add a user to a private event's invite list
export async function POST(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const { eventId, inviteUserId } = await req.json()
        if (!eventId || !inviteUserId) {
            return NextResponse.json({ error: "eventId and inviteUserId required" }, { status: 400 })
        }

        // Verify requester is the event creator
        const event = await db.select().from(events).where(eq(events.id, Number(eventId))).then(r => r[0])
        if (!event) return NextResponse.json({ error: "Event not found" }, { status: 404 })
        if (event.createdBy !== userId) return NextResponse.json({ error: "Forbidden" }, { status: 403 })
        if (!event.private) return NextResponse.json({ error: "Event is not private" }, { status: 400 })

        // Check invited user exists
        const invitedUser = await db.select().from(users).where(eq(users.id, inviteUserId)).then(r => r[0])
        if (!invitedUser) return NextResponse.json({ error: "User not found" }, { status: 404 })

        // Upsert invite
        const existing = await db
            .select()
            .from(eventInvites)
            .where(and(eq(eventInvites.eventId, Number(eventId)), eq(eventInvites.userId, inviteUserId)))
            .then(r => r[0])

        if (!existing) {
            await db.insert(eventInvites).values({ eventId: Number(eventId), userId: inviteUserId })
        }

        return NextResponse.json({ success: true, username: invitedUser.username })
    } catch (error) {
        console.error("Invite error:", error)
        return NextResponse.json({ error: "Failed to add invite" }, { status: 500 })
    }
}

// DELETE remove a user from a private event's invite list
export async function DELETE(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    const { searchParams } = new URL(req.url)
    const eventId = searchParams.get("eventId")
    const inviteUserId = searchParams.get("userId")
    if (!eventId || !inviteUserId) {
        return NextResponse.json({ error: "eventId and userId required" }, { status: 400 })
    }

    // Allow creator to remove others, or user to remove themselves
    const event = await db.select().from(events).where(eq(events.id, Number(eventId))).then(r => r[0])
    if (!event) return NextResponse.json({ error: "Event not found" }, { status: 404 })

    if (event.createdBy !== userId && inviteUserId !== userId) {
        return NextResponse.json({ error: "Forbidden" }, { status: 403 })
    }

    await db
        .delete(eventInvites)
        .where(and(eq(eventInvites.eventId, Number(eventId)), eq(eventInvites.userId, inviteUserId)))

    return NextResponse.json({ success: true })
}