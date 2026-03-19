import "server-only"
import { db } from "@/db"
import { settings } from "@/db/schema"
import { eq } from "drizzle-orm"

// ── Default message templates ─────────────────────────────────────────────────

export const DEFAULT_TEMPLATES = {
    created: "📅 **{event.title}** has been created by **{event.creator}**!\n📆 {event.date}{event.time}\n{event.location}{event.description}",
    updated: "✏️ **{event.title}** has been updated by **{event.creator}**.\n📆 {event.date}{event.time}\n{event.location}",
    deleted: "🗑️ **{event.title}** (created by **{event.creator}**) has been deleted.",
}

// ── Available placeholders ────────────────────────────────────────────────────

export const PLACEHOLDERS = [
    { key: "{event.title}",       description: "Event title" },
    { key: "{event.creator}",     description: "Creator username" },
    { key: "{event.date}",        description: "Event date (long format)" },
    { key: "{event.time}",        description: "Event time (empty if no time set)" },
    { key: "{event.location}",    description: "Location (empty if not set)" },
    { key: "{event.description}", description: "Description (empty if not set)" },
    { key: "{event.url}",         description: "Link to the calendar" },
]

// ── Settings helpers ──────────────────────────────────────────────────────────

async function getSetting(key: string): Promise<string> {
    try {
        const row = await db.select().from(settings).where(eq(settings.key, key)).then(r => r[0])
        return row?.value ?? ""
    } catch {
        return ""
    }
}

export async function getDiscordSettings() {
    const [webhookUrl, enabled, pingRoleId, templateCreated, templateUpdated, templateDeleted] = await Promise.all([
        getSetting("discord_webhook_url"),
        getSetting("discord_enabled"),
        getSetting("discord_ping_role_id"),
        getSetting("discord_template_created"),
        getSetting("discord_template_updated"),
        getSetting("discord_template_deleted"),
    ])

    return {
        webhookUrl,
        enabled: enabled === "true",
        pingRoleId,
        templates: {
            created: templateCreated || DEFAULT_TEMPLATES.created,
            updated: templateUpdated || DEFAULT_TEMPLATES.updated,
            deleted: templateDeleted || DEFAULT_TEMPLATES.deleted,
        },
    }
}

export async function saveDiscordSettings(data: {
    webhookUrl: string
    enabled: boolean
    pingRoleId: string
    templates: { created: string; updated: string; deleted: string }
}) {
    const upsert = async (key: string, value: string) => {
        const existing = await db.select().from(settings).where(eq(settings.key, key)).then(r => r[0])
        if (existing) {
            await db.update(settings).set({ value, updatedAt: new Date() }).where(eq(settings.key, key))
        } else {
            await db.insert(settings).values({ key, value })
        }
    }

    await Promise.all([
        upsert("discord_webhook_url", data.webhookUrl),
        upsert("discord_enabled", data.enabled ? "true" : "false"),
        upsert("discord_ping_role_id", data.pingRoleId),
        upsert("discord_template_created", data.templates.created),
        upsert("discord_template_updated", data.templates.updated),
        upsert("discord_template_deleted", data.templates.deleted),
    ])
}

// ── Template renderer ─────────────────────────────────────────────────────────

function renderTemplate(template: string, event: EventPayload, appUrl: string): string {
    const date = new Date(event.date)
    const dateStr = date.toLocaleDateString("en-GB", { weekday: "long", year: "numeric", month: "long", day: "numeric" })
    const hasTime = date.getHours() !== 0 || date.getMinutes() !== 0
    const timeStr = hasTime ? `⏰ ${date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}\n` : ""
    const locationStr = event.location ? `📍 ${event.location}\n` : ""
    const descStr = event.description ? `\n> ${event.description.slice(0, 200)}` : ""
    const urlStr = `${appUrl}/calendar`

    return template
        .replace(/{event\.title}/g, event.title)
        .replace(/{event\.creator}/g, event.creatorName || "Unknown")
        .replace(/{event\.date}/g, dateStr)
        .replace(/{event\.time}/g, timeStr)
        .replace(/{event\.location}/g, locationStr)
        .replace(/{event\.description}/g, descStr)
        .replace(/{event\.url}/g, urlStr)
}

// ── Main notify function ──────────────────────────────────────────────────────

interface EventPayload {
    title: string
    date: Date | string
    location?: string | null
    description?: string | null
    creatorName?: string | null
}

export async function sendDiscordNotification(
    type: "created" | "updated" | "deleted",
    event: EventPayload
) {
    const discordSettings = await getDiscordSettings()

    if (!discordSettings.enabled || !discordSettings.webhookUrl) return
    if (!discordSettings.webhookUrl.startsWith("https://discord.com/api/webhooks/")) return

    const appUrl = process.env.NEXT_PUBLIC_APP_URL || "http://localhost:3000"
    const content = renderTemplate(discordSettings.templates[type], event, appUrl)

    // Build role ping prefix if configured
    const pingPrefix = discordSettings.pingRoleId ? `<@&${discordSettings.pingRoleId}> ` : ""
    const fullContent = pingPrefix + content

    try {
        const res = await fetch(discordSettings.webhookUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ content: fullContent }),
        })
        if (!res.ok) {
            console.error("Discord webhook error:", res.status, await res.text())
        }
    } catch (e) {
        console.error("Discord webhook failed:", e)
    }
}