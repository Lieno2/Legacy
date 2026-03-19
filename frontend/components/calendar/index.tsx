"use client"

import { useState, useEffect, useCallback } from "react"

import { CalendarHeader } from "./CalendarHeader"
import { CalendarGrid } from "./CalendarGrid"
import { EventDetailDialog } from "./EventDetailDialog"
import { EventFormDialog } from "./EventFormDialog"
import { DeleteConfirmDialog } from "./DeleteConfirmDialog"

import { CalendarEvent, EventMember, RsvpStatus, DEFAULT_COLOR } from "./types"
import { toKey, hasTime, buildDateISO } from "./utils"

// ── Types ─────────────────────────────────────────────────────────────────────

interface BigCalendarProps {
    currentUserId: string | null
}

interface FormState {
    title: string
    description: string
    location: string
    time: string
    color: string
    isPrivate: boolean
    selectedDate: string | null
}

const EMPTY_FORM: FormState = {
    title: "",
    description: "",
    location: "",
    time: "",
    color: DEFAULT_COLOR,
    isPrivate: false,
    selectedDate: null,
}

// ── Component ─────────────────────────────────────────────────────────────────

export default function BigCalendar({ currentUserId }: BigCalendarProps) {
    const today = new Date()
    const [current, setCurrent] = useState({ year: today.getFullYear(), month: today.getMonth() })
    const [events, setEvents] = useState<CalendarEvent[]>([])
    const [search, setSearch] = useState("")

    // Detail state
    const [detailOpen, setDetailOpen] = useState(false)
    const [detailEvent, setDetailEvent] = useState<CalendarEvent | null>(null)
    const [detailMembers, setDetailMembers] = useState<EventMember[]>([])
    const [rsvpStatus, setRsvpStatus] = useState<RsvpStatus | null>(null)
    const [rsvpLoading, setRsvpLoading] = useState(false)

    // Form state
    const [editOpen, setEditOpen] = useState(false)
    const [editingEvent, setEditingEvent] = useState<CalendarEvent | null>(null)
    const [form, setForm] = useState<FormState>(EMPTY_FORM)
    const [saving, setSaving] = useState(false)

    // Delete state
    const [deleteOpen, setDeleteOpen] = useState(false)
    const [deletingId, setDeletingId] = useState<number | null>(null)
    const [deleting, setDeleting] = useState(false)

    const patchForm = (patch: Partial<FormState>) => setForm(f => ({ ...f, ...patch }))

    // ── Data fetching ──────────────────────────────────────────────────────────

    const fetchEvents = useCallback(async () => {
        try {
            const res = await fetch("/api/events")
            if (res.ok) setEvents(await res.json())
        } catch (e) {
            console.error("Failed to fetch events:", e)
        }
    }, [])

    const fetchMembers = useCallback(async (eventId: number): Promise<EventMember[]> => {
        try {
            const res = await fetch(`/api/rsvp?eventId=${eventId}`)
            if (res.ok) {
                const members: EventMember[] = await res.json()
                setDetailMembers(members)
                return members
            }
        } catch (e) {
            console.error("Failed to fetch members:", e)
        }
        return []
    }, [])

    useEffect(() => { fetchEvents() }, [fetchEvents])

    // ── Navigation ─────────────────────────────────────────────────────────────

    const { year, month } = current
    const prevMonth = () => setCurrent(c => c.month === 0 ? { year: c.year - 1, month: 11 } : { ...c, month: c.month - 1 })
    const nextMonth = () => setCurrent(c => c.month === 11 ? { year: c.year + 1, month: 0 } : { ...c, month: c.month + 1 })
    const goToday   = () => setCurrent({ year: today.getFullYear(), month: today.getMonth() })

    // ── Filtered events ────────────────────────────────────────────────────────

    const filteredEvents = search.trim()
        ? events.filter(e =>
            e.title.toLowerCase().includes(search.toLowerCase()) ||
            e.description?.toLowerCase().includes(search.toLowerCase()) ||
            e.location?.toLowerCase().includes(search.toLowerCase())
        )
        : events

    // ── Open detail ────────────────────────────────────────────────────────────

    async function openDetail(event: CalendarEvent, e: React.MouseEvent) {
        e.stopPropagation()
        setDetailEvent(event)
        setDetailMembers([])
        setDetailOpen(true)
        const members = await fetchMembers(event.id)
        const me = members.find(m => m.userId === currentUserId)
        setRsvpStatus(me ? (me.status as RsvpStatus) : null)
    }

    // ── Open create ────────────────────────────────────────────────────────────

    function openCreate(day: number) {
        const dateKey = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`
        setEditingEvent(null)
        setForm({ ...EMPTY_FORM, selectedDate: dateKey })
        setEditOpen(true)
    }

    // ── Open edit ──────────────────────────────────────────────────────────────

    function openEdit(event: CalendarEvent) {
        setDetailOpen(false)
        setEditingEvent(event)
        const d = new Date(event.date)
        setForm({
            title:        event.title,
            description:  event.description || "",
            location:     event.location || "",
            time:         hasTime(event.date)
                ? `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
                : "",
            color:        event.color || DEFAULT_COLOR,
            isPrivate:    event.private ?? false,
            selectedDate: toKey(d),
        })
        setEditOpen(true)
    }

    // ── Save event ─────────────────────────────────────────────────────────────

    async function saveEvent() {
        if (!form.title.trim() || !form.selectedDate) return
        setSaving(true)
        try {
            const payload = {
                title:       form.title.trim(),
                description: form.description.trim() || null,
                location:    form.location.trim() || null,
                date:        buildDateISO(form.selectedDate, form.time),
                color:       form.color,
                private:     form.isPrivate,
            }

            const res = editingEvent
                ? await fetch("/api/events", {
                    method: "PUT",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ id: editingEvent.id, ...payload }),
                })
                : await fetch("/api/events", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify(payload),
                })

            if (res.ok) {
                await fetchEvents()
                setEditOpen(false)
            } else {
                const err = await res.json()
                alert(`Failed: ${err.error || "Unknown error"}`)
            }
        } catch (e) {
            alert(`Error: ${e instanceof Error ? e.message : String(e)}`)
        } finally {
            setSaving(false)
        }
    }

    // ── Delete ─────────────────────────────────────────────────────────────────

    function confirmDelete(id: number) {
        setDeletingId(id)
        setDetailOpen(false)
        setDeleteOpen(true)
    }

    async function doDelete() {
        if (!deletingId) return
        setDeleting(true)
        try {
            const res = await fetch(`/api/events?id=${deletingId}`, { method: "DELETE" })
            if (res.ok) {
                await fetchEvents()
                setDeleteOpen(false)
                setDeletingId(null)
            }
        } finally {
            setDeleting(false)
        }
    }

    // ── RSVP ──────────────────────────────────────────────────────────────────

    async function submitRsvp(status: RsvpStatus, lateMinutes?: number) {
        if (!detailEvent) return
        setRsvpLoading(true)
        try {
            const res = await fetch("/api/rsvp", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ eventId: detailEvent.id, status, lateMinutes: lateMinutes ?? null }),
            })
            if (res.ok) {
                setRsvpStatus(status)
                await fetchMembers(detailEvent.id)
            }
        } finally {
            setRsvpLoading(false)
        }
    }

    async function removeRsvp() {
        if (!detailEvent) return
        setRsvpLoading(true)
        try {
            const res = await fetch(`/api/rsvp?eventId=${detailEvent.id}`, { method: "DELETE" })
            if (res.ok) {
                setRsvpStatus(null)
                await fetchMembers(detailEvent.id)
            }
        } finally {
            setRsvpLoading(false)
        }
    }

    // ── Render ─────────────────────────────────────────────────────────────────

    return (
        <div className="flex flex-col h-screen bg-background text-foreground">
            <CalendarHeader
                year={year}
                month={month}
                search={search}
                onSearchChange={setSearch}
                onPrevMonth={prevMonth}
                onNextMonth={nextMonth}
                onGoToday={goToday}
            />

            <CalendarGrid
                year={year}
                month={month}
                today={today}
                events={filteredEvents}
                onDayClick={openCreate}
                onEventClick={openDetail}
            />

            <EventDetailDialog
                open={detailOpen}
                event={detailEvent}
                members={detailMembers}
                rsvpStatus={rsvpStatus}
                rsvpLoading={rsvpLoading}
                currentUserId={currentUserId}
                onClose={() => setDetailOpen(false)}
                onEdit={openEdit}
                onDelete={confirmDelete}
                onRsvp={submitRsvp}
                onRemoveRsvp={removeRsvp}
            />

            <EventFormDialog
                open={editOpen}
                editingEvent={editingEvent}
                form={form}
                saving={saving}
                onChange={patchForm}
                onSave={saveEvent}
                onClose={() => setEditOpen(false)}
            />

            <DeleteConfirmDialog
                open={deleteOpen}
                deleting={deleting}
                onConfirm={doDelete}
                onCancel={() => setDeleteOpen(false)}
            />
        </div>
    )
}