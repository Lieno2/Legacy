import { NextRequest, NextResponse } from "next/server"
import { auth } from "@/auth"
import { db } from "@/db"
import { users } from "@/db/schema"
import { eq } from "drizzle-orm"
import { getDiscordSettings, saveDiscordSettings } from "@/lib/discord"

async function requireAdmin(session: any): Promise<boolean> {
    if (!session?.user?.id) return false
    if ((session.user.perms ?? 0) >= 999) return true
    const user = await db.select().from(users).where(eq(users.id, session.user.id)).then(r => r[0])
    return (user?.perms ?? 0) >= 999
}

export async function GET() {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const discordSettings = await getDiscordSettings()
    return NextResponse.json(discordSettings)
}

export async function PUT(req: NextRequest) {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    try {
        const body = await req.json()
        const { webhookUrl, enabled, pingRoleId, templates } = body

        if (webhookUrl && !webhookUrl.startsWith("https://discord.com/api/webhooks/")) {
            return NextResponse.json({ error: "Invalid Discord webhook URL — must start with https://discord.com/api/webhooks/" }, { status: 400 })
        }

        await saveDiscordSettings({
            webhookUrl: webhookUrl ?? "",
            enabled: Boolean(enabled),
            pingRoleId: pingRoleId ?? "",
            templates: {
                created: templates?.created ?? "",
                updated: templates?.updated ?? "",
                deleted: templates?.deleted ?? "",
            },
        })

        return NextResponse.json({ success: true })
    } catch (error) {
        console.error("Discord settings save error:", error)
        return NextResponse.json({ error: "Failed to save settings" }, { status: 500 })
    }
}

// Test the webhook with a sample message
export async function POST() {
    const session = await auth()
    if (!await requireAdmin(session)) return NextResponse.json({ error: "Forbidden" }, { status: 403 })

    const { sendDiscordNotification } = await import("@/lib/discord")
    await sendDiscordNotification("created", {
        title: "Test Event",
        date: new Date(),
        location: "Test Location",
        description: "This is a test notification from Legacy Calendar.",
        creatorName: session!.user!.name || "Admin",
    })

    return NextResponse.json({ success: true })
}