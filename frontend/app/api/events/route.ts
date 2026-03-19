import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { events } from "@/db/schema"
import { eq, and } from "drizzle-orm"

// GET all events for the current user
export async function GET() {
    const session = await auth()
    if (!session?.user?.id) {
        return NextResponse.json({ error: "Unauthorized" }, { status: 401 })
    }

    try {
        const userEvents = await db
            .select()
            .from(events)
            .where(eq(events.createdBy, session.user.id))
            .orderBy(events.date)

        return NextResponse.json(userEvents)
    } catch (error) {
        return NextResponse.json({ error: "Failed to fetch events" }, { status: 500 })
    }
}

// POST create a new event
export async function POST(req: NextRequest) {
    const session = await auth()
    if (!session?.user?.id) {
        return NextResponse.json({ error: "Unauthorized" }, { status: 401 })
    }

    try {
        const body = await req.json()
        const { title, description, date, location, private: isPrivate } = body

        if (!title || !date) {
            return NextResponse.json({ error: "Title and date are required" }, { status: 400 })
        }

        const newEvent = await db
            .insert(events)
            .values({
                title,
                description,
                date: new Date(date),
                location,
                createdBy: session.user.id,
                private: isPrivate ?? false,
            })
            .returning()

        return NextResponse.json(newEvent[0], { status: 201 })
    } catch (error) {
        console.error("Failed to create event:", error)
        return NextResponse.json({ error: "Failed to create event", details: error instanceof Error ? error.message : String(error) }, { status: 500 })
    }
}

// PUT update an event
export async function PUT(req: NextRequest) {
    const session = await auth()
    if (!session?.user?.id) {
        return NextResponse.json({ error: "Unauthorized" }, { status: 401 })
    }

    try {
        const body = await req.json()
        const { id, title, description, date, location, private: isPrivate } = body

        if (!id || !title || !date) {
            return NextResponse.json({ error: "ID, title and date are required" }, { status: 400 })
        }

        const updatedEvent = await db
            .update(events)
            .set({
                title,
                description,
                date: new Date(date),
                location,
                private: isPrivate,
            })
            .where(and(eq(events.id, id), eq(events.createdBy, session.user.id)))
            .returning()

        if (!updatedEvent[0]) {
            return NextResponse.json({ error: "Event not found or unauthorized" }, { status: 404 })
        }

        return NextResponse.json(updatedEvent[0])
    } catch (error) {
        return NextResponse.json({ error: "Failed to update event" }, { status: 500 })
    }
}

// DELETE an event
export async function DELETE(req: NextRequest) {
    const session = await auth()
    if (!session?.user?.id) {
        return NextResponse.json({ error: "Unauthorized" }, { status: 401 })
    }

    try {
        const { searchParams } = new URL(req.url)
        const id = searchParams.get("id")

        if (!id) {
            return NextResponse.json({ error: "Event ID is required" }, { status: 400 })
        }

        const deleted = await db
            .delete(events)
            .where(and(eq(events.id, Number(id)), eq(events.createdBy, session.user.id)))
            .returning()

        if (!deleted[0]) {
            return NextResponse.json({ error: "Event not found or unauthorized" }, { status: 404 })
        }

        return NextResponse.json({ success: true })
    } catch (error) {
        return NextResponse.json({ error: "Failed to delete event" }, { status: 500 })
    }
}
