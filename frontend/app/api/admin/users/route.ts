import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { users } from "@/db/schema"
import { eq } from "drizzle-orm"
import bcrypt from "bcryptjs"

async function requireAdmin(session: any) {
    if (!session?.user?.id) return false
    if ((session.user.perms ?? 0) >= 999) return true
    // fallback DB check
    const user = await db.select().from(users).where(eq(users.id, session.user.id)).then(r => r[0])
    return (user?.perms ?? 0) >= 999
}

// GET all users
export async function GET() {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const allUsers = await db
        .select({ id: users.id, username: users.username, email: users.email, perms: users.perms, createdAt: users.createdAt })
        .from(users)
        .orderBy(users.createdAt)

    return NextResponse.json(allUsers)
}

// POST create user
export async function POST(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    try {
        const { username, email, password, perms } = await req.json()

        if (!username?.trim() || !email?.trim() || !password) {
            return NextResponse.json({ error: "Username, email and password are required" }, { status: 400 })
        }
        if (password.length < 8) {
            return NextResponse.json({ error: "Password must be at least 8 characters" }, { status: 400 })
        }

        const existing = await db.select().from(users).where(eq(users.email, email.trim())).then(r => r[0])
        if (existing) return NextResponse.json({ error: "Email already in use" }, { status: 409 })

        const passwordHash = await bcrypt.hash(password, 10)
        const newUser = await db
            .insert(users)
            .values({ username: username.trim(), email: email.trim(), passwordHash, perms: perms ?? 0 })
            .returning({ id: users.id, username: users.username, email: users.email, perms: users.perms, createdAt: users.createdAt })

        return NextResponse.json(newUser[0], { status: 201 })
    } catch (error) {
        console.error("Create user error:", error)
        return NextResponse.json({ error: "Failed to create user" }, { status: 500 })
    }
}

// PUT update user
export async function PUT(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    try {
        const { id, username, email, perms, newPassword } = await req.json()
        if (!id || !username?.trim() || !email?.trim()) {
            return NextResponse.json({ error: "ID, username and email are required" }, { status: 400 })
        }

        const updateData: any = { username: username.trim(), email: email.trim(), perms: perms ?? 0 }

        if (newPassword) {
            if (newPassword.length < 8) {
                return NextResponse.json({ error: "Password must be at least 8 characters" }, { status: 400 })
            }
            updateData.passwordHash = await bcrypt.hash(newPassword, 10)
        }

        const updated = await db
            .update(users)
            .set(updateData)
            .where(eq(users.id, id))
            .returning({ id: users.id, username: users.username, email: users.email, perms: users.perms, createdAt: users.createdAt })

        if (!updated[0]) return NextResponse.json({ error: "User not found" }, { status: 404 })
        return NextResponse.json(updated[0])
    } catch (error) {
        return NextResponse.json({ error: "Failed to update user" }, { status: 500 })
    }
}

// DELETE user
export async function DELETE(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const { searchParams } = new URL(req.url)
    const id = searchParams.get("id")
    if (!id) return NextResponse.json({ error: "User ID required" }, { status: 400 })

    // Prevent self-deletion
    if (id === session!.user!.id) {
        return NextResponse.json({ error: "Cannot delete your own account" }, { status: 400 })
    }

    try {
        const deleted = await db.delete(users).where(eq(users.id, id)).returning()
        if (!deleted[0]) return NextResponse.json({ error: "User not found" }, { status: 404 })
        return NextResponse.json({ success: true })
    } catch (error) {
        return NextResponse.json({ error: "Failed to delete user" }, { status: 500 })
    }
}