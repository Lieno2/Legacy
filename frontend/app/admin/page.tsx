"use client"

import { useState, useEffect, useCallback } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import {
    AlertCircle, ArrowLeft, Shield, Users, CalendarDays, MessageSquare,
    Plus, Pencil, Trash2, Search, X, CheckCircle, Lock, Eye, EyeOff,
    Bell, BellOff, Send, RotateCcw, Info,
} from "lucide-react"
import Link from "next/link"

// ── Types ─────────────────────────────────────────────────────────────────────

interface AdminUser {
    id: string
    username: string
    email: string
    perms: number
    createdAt: string
}

interface AdminEvent {
    id: number
    title: string
    date: string
    location: string | null
    createdBy: string
    creatorName: string | null
    private: boolean
    color: string | null
}

interface DiscordSettings {
    webhookUrl: string
    enabled: boolean
    pingRoleId: string
    templates: {
        created: string
        updated: string
        deleted: string
    }
}

const DEFAULT_TEMPLATES = {
    created: "📅 **{event.title}** has been created by **{event.creator}**!\n📆 {event.date}{event.time}\n{event.location}{event.description}",
    updated: "✏️ **{event.title}** has been updated by **{event.creator}**.\n📆 {event.date}{event.time}\n{event.location}",
    deleted: "🗑️ **{event.title}** (created by **{event.creator}**) has been deleted.",
}

const PLACEHOLDERS = [
    { key: "{event.title}", description: "Event title" },
    { key: "{event.creator}", description: "Creator username" },
    { key: "{event.date}", description: "Full date" },
    { key: "{event.time}", description: "Time (empty if not set)" },
    { key: "{event.location}", description: "Location line (empty if not set)" },
    { key: "{event.description}", description: "Description (empty if not set)" },
    { key: "{event.url}", description: "Link to calendar" },
]

type Tab = "users" | "events" | "discord"

// ── Component ─────────────────────────────────────────────────────────────────

export default function AdminPage() {
    const [tab, setTab] = useState<Tab>("users")

    // Users
    const [users, setUsers] = useState<AdminUser[]>([])
    const [usersLoading, setUsersLoading] = useState(true)
    const [userSearch, setUserSearch] = useState("")
    const [userDialogOpen, setUserDialogOpen] = useState(false)
    const [editingUser, setEditingUser] = useState<AdminUser | null>(null)
    const [userForm, setUserForm] = useState({ username: "", email: "", password: "", perms: 0 })
    const [showPassword, setShowPassword] = useState(false)
    const [userSaving, setUserSaving] = useState(false)
    const [userError, setUserError] = useState("")
    const [deleteUserOpen, setDeleteUserOpen] = useState(false)
    const [deletingUserId, setDeletingUserId] = useState<string | null>(null)

    // Events
    const [events, setEvents] = useState<AdminEvent[]>([])
    const [eventsLoading, setEventsLoading] = useState(true)
    const [eventSearch, setEventSearch] = useState("")
    const [deleteEventOpen, setDeleteEventOpen] = useState(false)
    const [deletingEventId, setDeletingEventId] = useState<number | null>(null)

    // Discord
    const [discord, setDiscord] = useState<DiscordSettings>({
        webhookUrl: "", enabled: false, pingRoleId: "",
        templates: { ...DEFAULT_TEMPLATES },
    })
    const [discordLoading, setDiscordLoading] = useState(true)
    const [discordSaving, setDiscordSaving] = useState(false)
    const [discordTesting, setDiscordTesting] = useState(false)
    const [discordSuccess, setDiscordSuccess] = useState("")
    const [discordError, setDiscordError] = useState("")
    const [activeTemplate, setActiveTemplate] = useState<"created" | "updated" | "deleted">("created")

    // Shared
    const [deleteLoading, setDeleteLoading] = useState(false)
    const [globalError, setGlobalError] = useState("")

    // ── Fetch ──────────────────────────────────────────────────────────────────

    const fetchUsers = useCallback(async () => {
        setUsersLoading(true)
        try {
            const res = await fetch("/api/admin/users")
            if (res.ok) setUsers(await res.json())
            else setGlobalError("Failed to load users")
        } finally { setUsersLoading(false) }
    }, [])

    const fetchEvents = useCallback(async () => {
        setEventsLoading(true)
        try {
            const res = await fetch("/api/admin/events")
            if (res.ok) setEvents(await res.json())
            else setGlobalError("Failed to load events")
        } finally { setEventsLoading(false) }
    }, [])

    const fetchDiscord = useCallback(async () => {
        setDiscordLoading(true)
        try {
            const res = await fetch("/api/admin/discord")
            if (res.ok) {
                const data = await res.json()
                setDiscord({
                    webhookUrl: data.webhookUrl || "",
                    enabled: data.enabled || false,
                    pingRoleId: data.pingRoleId || "",
                    templates: {
                        created: data.templates?.created || DEFAULT_TEMPLATES.created,
                        updated: data.templates?.updated || DEFAULT_TEMPLATES.updated,
                        deleted: data.templates?.deleted || DEFAULT_TEMPLATES.deleted,
                    },
                })
            }
        } finally { setDiscordLoading(false) }
    }, [])

    useEffect(() => { fetchUsers() }, [fetchUsers])
    useEffect(() => { fetchEvents() }, [fetchEvents])
    useEffect(() => { fetchDiscord() }, [fetchDiscord])

    // ── User CRUD ──────────────────────────────────────────────────────────────

    function openCreateUser() {
        setEditingUser(null)
        setUserForm({ username: "", email: "", password: "", perms: 0 })
        setUserError("")
        setShowPassword(false)
        setUserDialogOpen(true)
    }

    function openEditUser(user: AdminUser) {
        setEditingUser(user)
        setUserForm({ username: user.username, email: user.email, password: "", perms: user.perms })
        setUserError("")
        setShowPassword(false)
        setUserDialogOpen(true)
    }

    async function saveUser() {
        setUserError("")
        if (!userForm.username.trim() || !userForm.email.trim()) { setUserError("Username and email are required"); return }
        if (!editingUser && !userForm.password) { setUserError("Password is required for new users"); return }
        if (userForm.password && userForm.password.length < 8) { setUserError("Password must be at least 8 characters"); return }

        setUserSaving(true)
        try {
            const payload = editingUser
                ? { id: editingUser.id, username: userForm.username, email: userForm.email, perms: userForm.perms, newPassword: userForm.password || undefined }
                : { username: userForm.username, email: userForm.email, password: userForm.password, perms: userForm.perms }

            const res = await fetch("/api/admin/users", {
                method: editingUser ? "PUT" : "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            })
            const data = await res.json()
            if (!res.ok) setUserError(data.error || "Failed to save user")
            else { await fetchUsers(); setUserDialogOpen(false) }
        } finally { setUserSaving(false) }
    }

    async function deleteUser() {
        if (!deletingUserId) return
        setDeleteLoading(true)
        try {
            const res = await fetch(`/api/admin/users?id=${deletingUserId}`, { method: "DELETE" })
            if (res.ok) { await fetchUsers(); setDeleteUserOpen(false); setDeletingUserId(null) }
        } finally { setDeleteLoading(false) }
    }

    async function deleteEvent() {
        if (!deletingEventId) return
        setDeleteLoading(true)
        try {
            const res = await fetch(`/api/admin/events?id=${deletingEventId}`, { method: "DELETE" })
            if (res.ok) { await fetchEvents(); setDeleteEventOpen(false); setDeletingEventId(null) }
        } finally { setDeleteLoading(false) }
    }

    // ── Discord ────────────────────────────────────────────────────────────────

    async function saveDiscord() {
        setDiscordSaving(true)
        setDiscordError("")
        setDiscordSuccess("")
        try {
            const res = await fetch("/api/admin/discord", {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(discord),
            })
            const data = await res.json()
            if (!res.ok) setDiscordError(data.error || "Failed to save")
            else setDiscordSuccess("Settings saved successfully!")
        } finally { setDiscordSaving(false) }
    }

    async function testDiscord() {
        setDiscordTesting(true)
        setDiscordError("")
        setDiscordSuccess("")
        try {
            const res = await fetch("/api/admin/discord", { method: "POST" })
            if (res.ok) setDiscordSuccess("Test message sent! Check your Discord channel.")
            else setDiscordError("Failed to send test message. Check your webhook URL.")
        } finally { setDiscordTesting(false) }
    }

    function resetTemplate(type: "created" | "updated" | "deleted") {
        setDiscord(d => ({ ...d, templates: { ...d.templates, [type]: DEFAULT_TEMPLATES[type] } }))
    }

    function insertPlaceholder(placeholder: string) {
        setDiscord(d => ({
            ...d,
            templates: {
                ...d.templates,
                [activeTemplate]: d.templates[activeTemplate] + placeholder,
            },
        }))
    }

    // ── Filtered ───────────────────────────────────────────────────────────────

    const filteredUsers = users.filter(u =>
        u.username.toLowerCase().includes(userSearch.toLowerCase()) ||
        u.email.toLowerCase().includes(userSearch.toLowerCase())
    )

    const filteredEvents = events.filter(e =>
        e.title.toLowerCase().includes(eventSearch.toLowerCase()) ||
        e.creatorName?.toLowerCase().includes(eventSearch.toLowerCase()) ||
        e.location?.toLowerCase().includes(eventSearch.toLowerCase())
    )

    // ── Render ─────────────────────────────────────────────────────────────────

    return (
        <div className="min-h-screen bg-background text-foreground">
            {/* Header */}
            <div className="border-b border-border/50 px-6 py-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <Link href="/calendar">
                        <Button variant="ghost" size="icon-sm"><ArrowLeft className="w-4 h-4" /></Button>
                    </Link>
                    <Shield className="w-5 h-5 text-amber-500" />
                    <h1 className="text-lg font-semibold">Admin Panel</h1>
                </div>
                <Link href="/account">
                    <Button variant="ghost" size="sm">Account</Button>
                </Link>
            </div>

            {globalError && (
                <div className="mx-6 mt-4 flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 border border-destructive/20 rounded-md">
                    <AlertCircle className="w-4 h-4 flex-shrink-0" /> {globalError}
                </div>
            )}

            {/* Stats */}
            <div className="px-6 pt-6 grid grid-cols-3 gap-4 max-w-4xl mx-auto">
                <Card>
                    <CardContent className="pt-5 flex items-center gap-3">
                        <div className="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
                            <Users className="w-5 h-5 text-blue-500" />
                        </div>
                        <div>
                            <div className="text-2xl font-bold">{users.length}</div>
                            <div className="text-sm text-muted-foreground">Users</div>
                        </div>
                    </CardContent>
                </Card>
                <Card>
                    <CardContent className="pt-5 flex items-center gap-3">
                        <div className="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center">
                            <CalendarDays className="w-5 h-5 text-violet-500" />
                        </div>
                        <div>
                            <div className="text-2xl font-bold">{events.length}</div>
                            <div className="text-sm text-muted-foreground">Events</div>
                        </div>
                    </CardContent>
                </Card>
                <Card>
                    <CardContent className="pt-5 flex items-center gap-3">
                        <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${discord.enabled ? "bg-emerald-500/10" : "bg-muted"}`}>
                            {discord.enabled
                                ? <Bell className="w-5 h-5 text-emerald-500" />
                                : <BellOff className="w-5 h-5 text-muted-foreground" />
                            }
                        </div>
                        <div>
                            <div className="text-2xl font-bold">{discord.enabled ? "On" : "Off"}</div>
                            <div className="text-sm text-muted-foreground">Discord</div>
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* Tabs */}
            <div className="px-6 pt-6 max-w-4xl mx-auto">
                <div className="flex gap-1 bg-muted/50 p-1 rounded-lg w-fit mb-6">
                    {(["users", "events", "discord"] as Tab[]).map(t => (
                        <button
                            key={t}
                            onClick={() => setTab(t)}
                            className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors capitalize flex items-center gap-1.5 ${tab === t ? "bg-background shadow-sm text-foreground" : "text-muted-foreground hover:text-foreground"}`}
                        >
                            {t === "users" && <Users className="w-3.5 h-3.5" />}
                            {t === "events" && <CalendarDays className="w-3.5 h-3.5" />}
                            {t === "discord" && <MessageSquare className="w-3.5 h-3.5" />}
                            {t}
                        </button>
                    ))}
                </div>

                {/* ── Users Tab ── */}
                {tab === "users" && (
                    <Card>
                        <CardHeader>
                            <div className="flex items-center justify-between">
                                <div>
                                    <CardTitle>Users</CardTitle>
                                    <CardDescription>Manage user accounts and permissions</CardDescription>
                                </div>
                                <Button size="sm" onClick={openCreateUser}>
                                    <Plus className="w-4 h-4 mr-1" /> New User
                                </Button>
                            </div>
                            <div className="relative mt-2">
                                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
                                <Input className="pl-8 h-8 text-sm" placeholder="Search users…" value={userSearch} onChange={e => setUserSearch(e.target.value)} />
                                {userSearch && <button onClick={() => setUserSearch("")} className="absolute right-2 top-1/2 -translate-y-1/2"><X className="w-3 h-3 text-muted-foreground" /></button>}
                            </div>
                        </CardHeader>
                        <CardContent>
                            {usersLoading ? <p className="text-sm text-muted-foreground">Loading…</p>
                                : filteredUsers.length === 0 ? <p className="text-sm text-muted-foreground">No users found.</p>
                                    : (
                                        <div className="flex flex-col divide-y divide-border/50">
                                            {filteredUsers.map(user => (
                                                <div key={user.id} className="flex items-center justify-between py-3">
                                                    <div className="flex items-center gap-3">
                                                        <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center text-xs font-bold uppercase">{user.username[0]}</div>
                                                        <div>
                                                            <div className="text-sm font-medium flex items-center gap-1.5">
                                                                {user.username}
                                                                {user.perms >= 999 && <span className="text-[10px] bg-amber-500/10 text-amber-500 px-1.5 py-0.5 rounded-full font-medium">Admin</span>}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">{user.email}</div>
                                                        </div>
                                                    </div>
                                                    <div className="flex items-center gap-1">
                                                        <Button variant="ghost" size="icon-sm" onClick={() => openEditUser(user)}><Pencil className="w-3.5 h-3.5" /></Button>
                                                        <Button variant="ghost" size="icon-sm" className="text-destructive hover:text-destructive" onClick={() => { setDeletingUserId(user.id); setDeleteUserOpen(true) }}><Trash2 className="w-3.5 h-3.5" /></Button>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    )}
                        </CardContent>
                    </Card>
                )}

                {/* ── Events Tab ── */}
                {tab === "events" && (
                    <Card>
                        <CardHeader>
                            <CardTitle>All Events</CardTitle>
                            <CardDescription>View and delete any event across all users</CardDescription>
                            <div className="relative mt-2">
                                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
                                <Input className="pl-8 h-8 text-sm" placeholder="Search events…" value={eventSearch} onChange={e => setEventSearch(e.target.value)} />
                                {eventSearch && <button onClick={() => setEventSearch("")} className="absolute right-2 top-1/2 -translate-y-1/2"><X className="w-3 h-3 text-muted-foreground" /></button>}
                            </div>
                        </CardHeader>
                        <CardContent>
                            {eventsLoading ? <p className="text-sm text-muted-foreground">Loading…</p>
                                : filteredEvents.length === 0 ? <p className="text-sm text-muted-foreground">No events found.</p>
                                    : (
                                        <div className="flex flex-col divide-y divide-border/50">
                                            {filteredEvents.map(event => (
                                                <div key={event.id} className="flex items-center justify-between py-3 gap-3">
                                                    <div className="flex items-center gap-3 min-w-0">
                                                        <div className={`w-2 h-8 rounded-full flex-shrink-0 ${event.color || "bg-blue-500"}`} />
                                                        <div className="min-w-0">
                                                            <div className="text-sm font-medium truncate flex items-center gap-1.5">
                                                                {event.title}
                                                                {event.private && <Lock className="w-3 h-3 text-muted-foreground flex-shrink-0" />}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">
                                                                by {event.creatorName ?? "Unknown"} · {new Date(event.date).toLocaleDateString()}
                                                                {event.location && ` · ${event.location}`}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <Button variant="ghost" size="icon-sm" className="text-destructive hover:text-destructive flex-shrink-0" onClick={() => { setDeletingEventId(event.id); setDeleteEventOpen(true) }}>
                                                        <Trash2 className="w-3.5 h-3.5" />
                                                    </Button>
                                                </div>
                                            ))}
                                        </div>
                                    )}
                        </CardContent>
                    </Card>
                )}

                {/* ── Discord Tab ── */}
                {tab === "discord" && (
                    <div className="flex flex-col gap-4 pb-10">
                        {/* Connection settings */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <MessageSquare className="w-4 h-4" /> Discord Integration
                                </CardTitle>
                                <CardDescription>Send notifications to a Discord channel when events are created, updated, or deleted.</CardDescription>
                            </CardHeader>
                            <CardContent className="flex flex-col gap-4">
                                {/* Enable toggle */}
                                <div className="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-muted/20">
                                    <div>
                                        <div className="text-sm font-medium">Enable notifications</div>
                                        <div className="text-xs text-muted-foreground">Send Discord messages when events change</div>
                                    </div>
                                    <button
                                        role="switch"
                                        aria-checked={discord.enabled}
                                        onClick={() => setDiscord(d => ({ ...d, enabled: !d.enabled }))}
                                        className={`relative w-11 h-6 rounded-full transition-colors ${discord.enabled ? "bg-emerald-500" : "bg-muted"}`}
                                    >
                                        <span className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white shadow transition-transform ${discord.enabled ? "translate-x-5" : ""}`} />
                                    </button>
                                </div>

                                {/* Webhook URL */}
                                <div className="flex flex-col gap-1.5">
                                    <Label htmlFor="webhook-url">Webhook URL</Label>
                                    <Input
                                        id="webhook-url"
                                        type="url"
                                        placeholder="https://discord.com/api/webhooks/..."
                                        value={discord.webhookUrl}
                                        onChange={e => setDiscord(d => ({ ...d, webhookUrl: e.target.value }))}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Discord server → Channel settings → Integrations → Webhooks → New Webhook
                                    </p>
                                </div>

                                {/* Ping role */}
                                <div className="flex flex-col gap-1.5">
                                    <Label htmlFor="ping-role">Ping Role ID <span className="text-muted-foreground font-normal">(optional)</span></Label>
                                    <Input
                                        id="ping-role"
                                        placeholder="e.g. 1234567890123456789"
                                        value={discord.pingRoleId}
                                        onChange={e => setDiscord(d => ({ ...d, pingRoleId: e.target.value }))}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Right-click a role in Discord → Copy Role ID (requires Developer Mode). The bot will ping this role on new events.
                                    </p>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Message templates */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Message Templates</CardTitle>
                                <CardDescription>Customize what gets sent to Discord for each event action.</CardDescription>
                            </CardHeader>
                            <CardContent className="flex flex-col gap-4">
                                {/* Template selector */}
                                <div className="flex gap-1 bg-muted/50 p-1 rounded-lg w-fit">
                                    {(["created", "updated", "deleted"] as const).map(t => (
                                        <button
                                            key={t}
                                            onClick={() => setActiveTemplate(t)}
                                            className={`px-3 py-1 rounded-md text-xs font-medium transition-colors capitalize ${activeTemplate === t ? "bg-background shadow-sm text-foreground" : "text-muted-foreground hover:text-foreground"}`}
                                        >
                                            {t === "created" ? "📅 Created" : t === "updated" ? "✏️ Updated" : "🗑️ Deleted"}
                                        </button>
                                    ))}
                                </div>

                                {/* Template textarea */}
                                <div className="flex flex-col gap-1.5">
                                    <div className="flex items-center justify-between">
                                        <Label>Message for &quot;{activeTemplate}&quot;</Label>
                                        <Button variant="ghost" size="xs" onClick={() => resetTemplate(activeTemplate)}>
                                            <RotateCcw className="w-3 h-3 mr-1" /> Reset to default
                                        </Button>
                                    </div>
                                    <Textarea
                                        value={discord.templates[activeTemplate]}
                                        onChange={e => setDiscord(d => ({ ...d, templates: { ...d.templates, [activeTemplate]: e.target.value } }))}
                                        rows={5}
                                        className="font-mono text-sm"
                                    />
                                </div>

                                {/* Placeholders */}
                                <div className="flex flex-col gap-2">
                                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                        <Info className="w-3.5 h-3.5" />
                                        Click a placeholder to insert it at the end of the current template
                                    </div>
                                    <div className="flex flex-wrap gap-1.5">
                                        {PLACEHOLDERS.map(p => (
                                            <button
                                                key={p.key}
                                                onClick={() => insertPlaceholder(p.key)}
                                                title={p.description}
                                                className="text-xs font-mono bg-muted hover:bg-muted/70 border border-border/50 px-2 py-1 rounded transition-colors"
                                            >
                                                {p.key}
                                            </button>
                                        ))}
                                    </div>
                                </div>

                                {/* Preview */}
                                <div className="flex flex-col gap-1.5">
                                    <Label className="text-muted-foreground text-xs">Preview (with sample data)</Label>
                                    <div className="bg-[#36393f] rounded-lg p-3 text-sm text-[#dcddde] font-mono whitespace-pre-wrap break-words min-h-[60px]">
                                        {discord.templates[activeTemplate]
                                            .replace(/{event\.title}/g, "Team Dinner")
                                            .replace(/{event\.creator}/g, "admin")
                                            .replace(/{event\.date}/g, "Friday, 20 March 2026")
                                            .replace(/{event\.time}/g, "⏰ 19:30\n")
                                            .replace(/{event\.location}/g, "📍 Trattoria Roma\n")
                                            .replace(/{event\.description}/g, "\n> A nice dinner together!")
                                            .replace(/{event\.url}/g, "https://yourapp.com/calendar")
                                        }
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* Actions */}
                        {discordError && (
                            <div className="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 border border-destructive/20 rounded-md">
                                <AlertCircle className="w-4 h-4 flex-shrink-0" /> {discordError}
                            </div>
                        )}
                        {discordSuccess && (
                            <div className="flex items-center gap-2 p-3 text-sm text-emerald-600 bg-emerald-500/10 border border-emerald-500/20 rounded-md">
                                <CheckCircle className="w-4 h-4 flex-shrink-0" /> {discordSuccess}
                            </div>
                        )}
                        <div className="flex gap-2">
                            <Button onClick={saveDiscord} disabled={discordSaving} className="flex-1">
                                {discordSaving ? "Saving…" : "Save Settings"}
                            </Button>
                            <Button variant="outline" onClick={testDiscord} disabled={discordTesting || !discord.webhookUrl}>
                                <Send className="w-4 h-4 mr-1.5" />
                                {discordTesting ? "Sending…" : "Test"}
                            </Button>
                        </div>
                    </div>
                )}
            </div>

            {/* ── User Form Dialog ── */}
            <Dialog open={userDialogOpen} onOpenChange={setUserDialogOpen}>
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            {editingUser ? <><Pencil className="w-4 h-4" /> Edit User</> : <><Plus className="w-4 h-4" /> New User</>}
                        </DialogTitle>
                    </DialogHeader>
                    <div className="flex flex-col gap-4 py-1">
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor="adm-username">Username</Label>
                            <Input id="adm-username" value={userForm.username} onChange={e => setUserForm(f => ({ ...f, username: e.target.value }))} autoFocus />
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor="adm-email">Email</Label>
                            <Input id="adm-email" type="email" value={userForm.email} onChange={e => setUserForm(f => ({ ...f, email: e.target.value }))} />
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor="adm-password">
                                Password {editingUser && <span className="text-muted-foreground font-normal text-xs">(leave blank to keep current)</span>}
                            </Label>
                            <div className="relative">
                                <Input
                                    id="adm-password"
                                    type={showPassword ? "text" : "password"}
                                    placeholder={editingUser ? "New password…" : "Min. 8 characters"}
                                    value={userForm.password}
                                    onChange={e => setUserForm(f => ({ ...f, password: e.target.value }))}
                                    className="pr-9"
                                />
                                <button type="button" onClick={() => setShowPassword(v => !v)} className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                                </button>
                            </div>
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label>Role</Label>
                            <div className="flex gap-2">
                                <button onClick={() => setUserForm(f => ({ ...f, perms: 0 }))} className={`flex-1 py-2 rounded-md border text-sm font-medium transition-colors ${userForm.perms < 999 ? "border-primary bg-primary/10 text-primary" : "border-border text-muted-foreground hover:border-muted-foreground"}`}>User</button>
                                <button onClick={() => setUserForm(f => ({ ...f, perms: 999 }))} className={`flex-1 py-2 rounded-md border text-sm font-medium transition-colors ${userForm.perms >= 999 ? "border-amber-500 bg-amber-500/10 text-amber-500" : "border-border text-muted-foreground hover:border-muted-foreground"}`}>
                                    <Shield className="w-3.5 h-3.5 inline mr-1" />Admin
                                </button>
                            </div>
                        </div>
                        {userError && (
                            <div className="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 border border-destructive/20 rounded-md">
                                <AlertCircle className="w-4 h-4 flex-shrink-0" /> {userError}
                            </div>
                        )}
                    </div>
                    <DialogFooter>
                        <Button variant="ghost" onClick={() => setUserDialogOpen(false)} disabled={userSaving}>Cancel</Button>
                        <Button onClick={saveUser} disabled={userSaving}>{userSaving ? "Saving…" : editingUser ? "Update" : "Create"}</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* ── Delete User Dialog ── */}
            <Dialog open={deleteUserOpen} onOpenChange={setDeleteUserOpen}>
                <DialogContent className="sm:max-w-sm">
                    <DialogHeader><DialogTitle className="flex items-center gap-2 text-destructive"><AlertCircle className="w-5 h-5" /> Delete User</DialogTitle></DialogHeader>
                    <p className="text-sm text-muted-foreground">This will permanently delete the user and all their data. This cannot be undone.</p>
                    <DialogFooter>
                        <Button variant="ghost" onClick={() => setDeleteUserOpen(false)} disabled={deleteLoading}>Cancel</Button>
                        <Button variant="destructive" onClick={deleteUser} disabled={deleteLoading}>{deleteLoading ? "Deleting…" : "Delete"}</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* ── Delete Event Dialog ── */}
            <Dialog open={deleteEventOpen} onOpenChange={setDeleteEventOpen}>
                <DialogContent className="sm:max-w-sm">
                    <DialogHeader><DialogTitle className="flex items-center gap-2 text-destructive"><AlertCircle className="w-5 h-5" /> Delete Event</DialogTitle></DialogHeader>
                    <p className="text-sm text-muted-foreground">This will permanently delete this event and all its RSVPs. This cannot be undone.</p>
                    <DialogFooter>
                        <Button variant="ghost" onClick={() => setDeleteEventOpen(false)} disabled={deleteLoading}>Cancel</Button>
                        <Button variant="destructive" onClick={deleteEvent} disabled={deleteLoading}>{deleteLoading ? "Deleting…" : "Delete"}</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}