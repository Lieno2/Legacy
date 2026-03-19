import "server-only"
import { db } from "@/db"
import { settings } from "@/db/schema"
import { eq } from "drizzle-orm"

export async function getDiscordWebhookUrl(): Promise<string | null> {
    try {
        const setting = await db.select().from(settings).where(eq(settings.key, "discord_webhook_url")).then(r => r[0])
        if (setting?.value) return setting.value
    } catch {
        // fallback to env if DB not available
    }
    return process.env.DISCORD_WEBHOOK_URL || null
}

export async function sendDiscordNotification(
    type: "created" | "updated" | "deleted",
    event: { title: string; date?: Date | string; location?: string | null; description?: string | null; private?: boolean; creatorName?: string | null }
) {
    const webhookUrl = await getDiscordWebhookUrl()
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

    if (type !== "deleted" && event.date) {
        const date = new Date(event.date)
        embed.fields.push({
            name: "Date",
            value: date.toLocaleDateString("en-GB", { weekday: "long", year: "numeric", month: "long", day: "numeric" }),
            inline: false,
        })
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