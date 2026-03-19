"use client"

import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { ChevronLeft, ChevronRight, Plus, Edit, MapPin } from "lucide-react"

// ── Types ────────────────────────────────────────────────────────────────────

interface CalendarEvent {
  id: number
  title: string
  description?: string | null
  date: string // ISO timestamp
  location?: string | null
  color?: string
  createdBy: string
  createdAt?: string
  private?: boolean
}

// ── Helpers ──────────────────────────────────────────────────────────────────

const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
]

function toKey(date: Date) {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`
}

function getDaysInMonth(year: number, month: number) {
  return new Date(year, month + 1, 0).getDate()
}

function getFirstDayOfMonth(year: number, month: number) {
  return new Date(year, month, 1).getDay()
}

const EVENT_COLORS = [
  "bg-blue-500",
  "bg-violet-500",
  "bg-emerald-500",
  "bg-rose-500",
  "bg-amber-500",
]

// ── Component ─────────────────────────────────────────────────────────────────

export default function BigCalendar() {
  const today = new Date()
  const [current, setCurrent] = useState({ year: today.getFullYear(), month: today.getMonth() })
  const [events, setEvents] = useState<CalendarEvent[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const [selectedDate, setSelectedDate] = useState<string | null>(null)
  const [editingEvent, setEditingEvent] = useState<CalendarEvent | null>(null)
  const [newTitle, setNewTitle] = useState("")
  const [newDescription, setNewDescription] = useState("")
  const [newLocation, setNewLocation] = useState("")
  const [selectedColor, setSelectedColor] = useState(EVENT_COLORS[0])
  const [loading, setLoading] = useState(false)

  // Fetch events from API
  useEffect(() => {
    fetchEvents()
  }, [])

  async function fetchEvents() {
    try {
      const res = await fetch("/api/events")
      if (res.ok) {
        const data = await res.json()
        setEvents(data)
      }
    } catch (error) {
      console.error("Failed to fetch events:", error)
    }
  }

  const { year, month } = current
  const daysInMonth = getDaysInMonth(year, month)
  const firstDay = getFirstDayOfMonth(year, month)

  // Build grid cells (leading empty + day numbers)
  const cells: (number | null)[] = [
    ...Array(firstDay).fill(null),
    ...Array.from({ length: daysInMonth }, (_, i) => i + 1),
  ]
  // Pad to complete last row
  while (cells.length % 7 !== 0) cells.push(null)

  function prevMonth() {
    setCurrent(c => c.month === 0
        ? { year: c.year - 1, month: 11 }
        : { year: c.year, month: c.month - 1 }
    )
  }

  function nextMonth() {
    setCurrent(c => c.month === 11
        ? { year: c.year + 1, month: 0 }
        : { year: c.year, month: c.month + 1 }
    )
  }

  function goToday() {
    setCurrent({ year: today.getFullYear(), month: today.getMonth() })
  }

  function openDialog(day: number) {
    const key = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`
    setSelectedDate(key)
    setEditingEvent(null)
    setNewTitle("")
    setNewDescription("")
    setNewLocation("")
    setSelectedColor(EVENT_COLORS[0])
    setDialogOpen(true)
  }

  function openEditDialog(event: CalendarEvent, e: React.MouseEvent) {
    e.stopPropagation()
    setEditingEvent(event)
    setNewTitle(event.title)
    setNewDescription(event.description || "")
    setNewLocation(event.location || "")
    setSelectedColor(event.color || EVENT_COLORS[0])
    setSelectedDate(toKey(new Date(event.date)))
    setDialogOpen(true)
  }

  async function saveEvent() {
    if (!newTitle.trim() || !selectedDate) return
    setLoading(true)

    try {
      if (editingEvent) {
        // Update existing event
        const res = await fetch("/api/events", {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            id: editingEvent.id,
            title: newTitle.trim(),
            description: newDescription.trim() || null,
            location: newLocation.trim() || null,
            date: new Date(selectedDate).toISOString(),
            private: editingEvent.private,
          }),
        })

        if (res.ok) {
          await fetchEvents()
          setDialogOpen(false)
        }
      } else {
        // Create new event
        const res = await fetch("/api/events", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            title: newTitle.trim(),
            description: newDescription.trim() || null,
            location: newLocation.trim() || null,
            date: new Date(selectedDate).toISOString(),
            private: false,
          }),
        })

        if (res.ok) {
          await fetchEvents()
          setDialogOpen(false)
        }
      }
    } catch (error) {
      console.error("Failed to save event:", error)
    } finally {
      setLoading(false)
    }
  }

  async function deleteEvent(id: number, e: React.MouseEvent) {
    e.stopPropagation()
    if (!confirm("Delete this event?")) return

    try {
      const res = await fetch(`/api/events?id=${id}`, { method: "DELETE" })
      if (res.ok) {
        await fetchEvents()
      }
    } catch (error) {
      console.error("Failed to delete event:", error)
    }
  }

  const isToday = (day: number) =>
      day === today.getDate() && month === today.getMonth() && year === today.getFullYear()

  return (
      <div className="flex flex-col h-screen bg-background text-foreground p-4 gap-4">

        {/* ── Header ── */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold tracking-tight">
              {MONTHS[month]} <span className="text-muted-foreground font-normal">{year}</span>
            </h1>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={goToday}>Today</Button>
            <Button variant="ghost" size="icon" onClick={prevMonth}><ChevronLeft className="w-4 h-4" /></Button>
            <Button variant="ghost" size="icon" onClick={nextMonth}><ChevronRight className="w-4 h-4" /></Button>
          </div>
        </div>

        {/* ── Grid ── */}
        <Card className="flex-1 overflow-hidden p-0">
          <div className="grid grid-cols-7 h-full" style={{ gridTemplateRows: "auto repeat(auto-fill, 1fr)" }}>

            {/* Day headers */}
            {DAYS.map(d => (
                <div key={d} className="text-xs font-medium text-muted-foreground text-center py-2 border-b border-border">
                  {d}
                </div>
            ))}

            {/* Day cells */}
            {cells.map((day, i) => {
              if (!day) return (
                  <div key={`empty-${i}`} className="border-b border-r border-border bg-muted/20" />
              )

              const key = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`
              const dayEvents = events.filter(e => toKey(new Date(e.date)) === key)
              const today_ = isToday(day)

              return (
                  <div
                      key={key}
                      onClick={() => openDialog(day)}
                      className="border-b border-r border-border p-1.5 flex flex-col gap-1 cursor-pointer hover:bg-muted/30 transition-colors min-h-[80px]"
                  >
                    {/* Day number */}
                    <span className={`text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full self-start
                  ${today_ ? "bg-primary text-primary-foreground" : "text-muted-foreground"}`}>
                  {day}
                </span>

                    {/* Events */}
                    <div className="flex flex-col gap-0.5 overflow-hidden">
                      {dayEvents.slice(0, 3).map(event => (
                          <div
                              key={event.id}
                              onClick={e => openEditDialog(event, e)}
                              title="Click to edit"
                              className={`text-[11px] leading-tight px-1.5 py-0.5 rounded truncate text-white font-medium cursor-pointer hover:opacity-70 transition-opacity ${event.color || EVENT_COLORS[0]} flex items-center gap-1`}
                          >
                            {event.location && <MapPin className="w-3 h-3 flex-shrink-0" />}
                            <span className="truncate">{event.title}</span>
                          </div>
                      ))}
                      {dayEvents.length > 3 && (
                          <span className="text-[10px] text-muted-foreground pl-1">
                      +{dayEvents.length - 3} more
                    </span>
                      )}
                    </div>
                  </div>
              )
            })}
          </div>
        </Card>

        {/* ── Add/Edit Event Dialog ── */}
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>
                {editingEvent ? "Edit Event" : "New Event"} — {selectedDate}
              </DialogTitle>
            </DialogHeader>

            <div className="flex flex-col gap-4 py-2">
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="title">Event title *</Label>
                <Input
                    id="title"
                    placeholder="e.g. Team meeting"
                    value={newTitle}
                    onChange={e => setNewTitle(e.target.value)}
                    autoFocus
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <Label htmlFor="description">Description</Label>
                <Textarea
                    id="description"
                    placeholder="Add details about the event"
                    value={newDescription}
                    onChange={e => setNewDescription(e.target.value)}
                    rows={3}
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <Label htmlFor="location">Location</Label>
                <Input
                    id="location"
                    placeholder="e.g. Conference Room A or https://maps.google.com/..."
                    value={newLocation}
                    onChange={e => setNewLocation(e.target.value)}
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <Label>Color</Label>
                <div className="flex gap-2">
                  {EVENT_COLORS.map(color => (
                      <button
                          key={color}
                          onClick={() => setSelectedColor(color)}
                          className={`w-6 h-6 rounded-full ${color} transition-transform ${selectedColor === color ? "ring-2 ring-offset-2 ring-offset-background ring-primary scale-110" : ""}`}
                      />
                  ))}
                </div>
              </div>
            </div>

            <DialogFooter className="gap-2">
              {editingEvent && (
                  <Button
                      variant="destructive"
                      onClick={(e) => {
                        deleteEvent(editingEvent.id, e)
                        setDialogOpen(false)
                      }}
                      disabled={loading}
                      className="mr-auto"
                  >
                    Delete
                  </Button>
              )}
              <Button variant="ghost" onClick={() => setDialogOpen(false)} disabled={loading}>
                Cancel
              </Button>
              <Button onClick={saveEvent} disabled={!newTitle.trim() || loading}>
                {loading ? "Saving..." : editingEvent ? "Update" : "Add Event"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
  )
}