import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { events, users } from "@/db/schema"
import { eq } from "drizzle-orm"

async function requireAdmin(session: any): Promise<boolean> {
    if (!session?.user?.id) return false
    if ((session.user.perms ?? 0) >= 999) return true
    const user = await db.select().from(users).where(eq(users.id, session.user.id)).then(r => r[0])
    return (user?.perms ?? 0) >= 999
}

export async function GET() {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

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
        .orderBy(events.date)

    return NextResponse.json(allEvents)
}

export async function DELETE(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const { searchParams } = new URL(req.url)
    const id = searchParams.get("id")
    if (!id) return NextResponse.json({ error: "Event ID required" }, { status: 400 })

    try {
        const deleted = await db.delete(events).where(eq(events.id, Number(id))).returning()
        if (!deleted[0]) return NextResponse.json({ error: "Event not found" }, { status: 404 })
        return NextResponse.json({ success: true })
    } catch (error) {
        console.error("Admin delete event error:", error)
        return NextResponse.json({ error: "Failed to delete event" }, { status: 500 })
    }
}