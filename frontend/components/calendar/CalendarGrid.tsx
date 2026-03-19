"use client"

import { Clock, MapPin, Plus } from "lucide-react"
import { CalendarEvent, DAYS, DEFAULT_COLOR } from "./types"
import { toKey, hasTime, buildGridCells } from "./utils"

interface CalendarGridProps {
    year: number
    month: number
    today: Date
    events: CalendarEvent[]
    onDayClick: (day: number) => void
    onEventClick: (event: CalendarEvent, e: React.MouseEvent) => void
}

export function CalendarGrid({ year, month, today, events, onDayClick, onEventClick }: CalendarGridProps) {
    const cells = buildGridCells(year, month)

    const isToday = (day: number) =>
        day === today.getDate() && month === today.getMonth() && year === today.getFullYear()

    return (
        <div className="flex-1 overflow-hidden flex flex-col">
            {/* Day headers */}
            <div className="grid grid-cols-7 border-b border-border/50">
                {DAYS.map(d => (
                    <div
                        key={d}
                        className="text-[11px] font-semibold text-muted-foreground/70 text-center py-2 uppercase tracking-widest"
                    >
                        {d}
                    </div>
                ))}
            </div>

            {/* Day cells */}
            <div className="flex-1 grid grid-cols-7" style={{ gridAutoRows: "1fr" }}>
                {cells.map((day, i) => {
                    if (!day) return (
                        <div key={`empty-${i}`} className="border-b border-r border-border/30 bg-muted/10" />
                    )

                    const key = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`
                    const dayEvents = events.filter(e => toKey(new Date(e.date)) === key)
                    const todayCell = isToday(day)

                    return (
                        <div
                            key={key}
                            onClick={() => onDayClick(day)}
                            className="border-b border-r border-border/30 p-1.5 flex flex-col gap-1 cursor-pointer hover:bg-muted/20 transition-colors group relative"
                        >
                            <div className="flex items-center justify-between">
                                <span className={`text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full transition-colors
                                    ${todayCell
                                        ? "bg-primary text-primary-foreground font-bold"
                                        : "text-muted-foreground group-hover:text-foreground"
                                    }`}
                                >
                                    {day}
                                </span>
                                <Plus className="w-3 h-3 text-muted-foreground/0 group-hover:text-muted-foreground/50 transition-all" />
                            </div>

                            <div className="flex flex-col gap-0.5 overflow-hidden">
                                {dayEvents.slice(0, 3).map(event => (
                                    <EventChip
                                        key={event.id}
                                        event={event}
                                        onClick={onEventClick}
                                    />
                                ))}
                                {dayEvents.length > 3 && (
                                    <span className="text-[10px] text-muted-foreground pl-1 font-medium">
                                        +{dayEvents.length - 3} more
                                    </span>
                                )}
                            </div>
                        </div>
                    )
                })}
            </div>
        </div>
    )
}

// ── Event chip inside a day cell ───────────────────────────────────────────────

interface EventChipProps {
    event: CalendarEvent
    onClick: (event: CalendarEvent, e: React.MouseEvent) => void
}

function EventChip({ event, onClick }: EventChipProps) {
    return (
        <button
            onClick={e => onClick(event, e)}
            className={`text-[11px] leading-tight px-1.5 py-0.5 rounded text-white font-medium hover:opacity-80 transition-opacity ${event.color || DEFAULT_COLOR} flex items-center gap-1 w-full text-left`}
        >
            {event.location && <MapPin className="w-2.5 h-2.5 flex-shrink-0 opacity-80" />}
            {hasTime(event.date) && <Clock className="w-2.5 h-2.5 flex-shrink-0 opacity-80" />}
            <span className="truncate">{event.title}</span>
        </button>
    )
}
