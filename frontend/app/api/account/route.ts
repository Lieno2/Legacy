import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { users } from "@/db/schema"
import { eq } from "drizzle-orm"
import bcrypt from "bcryptjs"

async function getUserId(session: any): Promise<string | null> {
    if (session?.user?.id) return session.user.id
    if (session?.user?.email) {
        const user = await db.select().from(users).where(eq(users.email, session.user.email)).then(r => r[0])
        return user?.id || null
    }
    return null
}

// GET current user profile
export async function GET() {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    const user = await db
        .select({ id: users.id, username: users.username, email: users.email, perms: users.perms, createdAt: users.createdAt })
        .from(users)
        .where(eq(users.id, userId))
        .then(r => r[0])

    if (!user) return NextResponse.json({ error: "User not found" }, { status: 404 })
    return NextResponse.json(user)
}

// PUT update own profile
export async function PUT(req: NextRequest) {
    const session = await auth()
    const userId = await getUserId(session)
    if (!userId) return NextResponse.json({ error: "Unauthorized" }, { status: 401 })

    try {
        const body = await req.json()
        const { username, email, currentPassword, newPassword } = body

        // Validate required fields
        if (!username?.trim() || !email?.trim()) {
            return NextResponse.json({ error: "Username and email are required" }, { status: 400 })
        }

        // Check username/email not taken by another user
        const existing = await db.select().from(users).where(eq(users.username, username.trim())).then(r => r[0])
        if (existing && existing.id !== userId) {
            return NextResponse.json({ error: "Username already taken" }, { status: 409 })
        }
        const existingEmail = await db.select().from(users).where(eq(users.email, email.trim())).then(r => r[0])
        if (existingEmail && existingEmail.id !== userId) {
            return NextResponse.json({ error: "Email already in use" }, { status: 409 })
        }

        const updateData: any = {
            username: username.trim(),
            email: email.trim(),
        }

        // Handle password change
        if (newPassword) {
            if (!currentPassword) {
                return NextResponse.json({ error: "Current password is required to set a new one" }, { status: 400 })
            }
            const user = await db.select().from(users).where(eq(users.id, userId)).then(r => r[0])
            const valid = await bcrypt.compare(currentPassword, user.passwordHash)
            if (!valid) {
                return NextResponse.json({ error: "Current password is incorrect" }, { status: 400 })
            }
            if (newPassword.length < 8) {
                return NextResponse.json({ error: "New password must be at least 8 characters" }, { status: 400 })
            }
            updateData.passwordHash = await bcrypt.hash(newPassword, 10)
        }

        const updated = await db
            .update(users)
            .set(updateData)
            .where(eq(users.id, userId))
            .returning({ id: users.id, username: users.username, email: users.email, perms: users.perms })

        return NextResponse.json(updated[0])
    } catch (error) {
        console.error("Account update error:", error)
        return NextResponse.json({ error: "Failed to update account" }, { status: 500 })
    }
}