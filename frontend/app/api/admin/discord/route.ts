import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { users, settings } from "@/db/schema"
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

    const setting = await db.select().from(settings).where(eq(settings.key, "discord_webhook_url")).then(r => r[0])
    return NextResponse.json({ webhookUrl: setting?.value || "" })
}

export async function PUT(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const { webhookUrl } = await req.json()

    // Basic validation
    if (webhookUrl && !webhookUrl.startsWith("https://discord.com/api/webhooks/")) {
        return NextResponse.json({ error: "Invalid Discord webhook URL" }, { status: 400 })
    }

    const existing = await db.select().from(settings).where(eq(settings.key, "discord_webhook_url")).then(r => r[0])

    if (existing) {
        await db.update(settings).set({ value: webhookUrl || "" }).where(eq(settings.key, "discord_webhook_url"))
    } else {
        await db.insert(settings).values({ key: "discord_webhook_url", value: webhookUrl || "" })
    }

    return NextResponse.json({ success: true })
}