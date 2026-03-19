"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog"
import { VisuallyHidden } from "@radix-ui/react-visually-hidden"
import {
    MapPin, Pencil, Trash2, X, Clock, User, Users,
    Check, CalendarDays, LogOut,
} from "lucide-react"
import { CalendarEvent, EventMember, RsvpStatus, COLOR_HEX, DEFAULT_COLOR } from "./types"
import { formatDate, formatTime, hasTime } from "./utils"

interface EventDetailDialogProps {
    open: boolean
    event: CalendarEvent | null
    members: EventMember[]
    rsvpStatus: RsvpStatus | null
    rsvpLoading: boolean
    currentUserId: string | null
    onClose: () => void
    onEdit: (event: CalendarEvent) => void
    onDelete: (id: number) => void
    onRsvp: (status: RsvpStatus, lateMinutes?: number) => void
    onRemoveRsvp: () => void
}

export function EventDetailDialog({
                                      open, event, members, rsvpStatus, rsvpLoading,
                                      currentUserId, onClose, onEdit, onDelete, onRsvp, onRemoveRsvp,
                                  }: EventDetailDialogProps) {
    const [showLateInput, setShowLateInput] = useState(false)
    const [lateMinutes, setLateMinutes] = useState(15)

    if (!event) return null

    const d = new Date(event.date)
    const colorHex = COLOR_HEX[event.color || DEFAULT_COLOR] || "#3b82f6"
    const isOwner = event.createdBy === currentUserId

    const goingCount = members.filter(m => m.status === "going").length
    const lateCount = members.filter(m => m.status === "late").length
    const notGoingCount = members.filter(m => m.status === "not_going").length

    return (
        <Dialog open={open} onOpenChange={onClose}>
            <DialogContent className="sm:max-w-lg p-0 overflow-hidden gap-0" showCloseButton={false}>
                <div className="h-2 w-full" style={{ background: colorHex }} />
                <VisuallyHidden>
                    <DialogTitle>{event.title}</DialogTitle>
                </VisuallyHidden>

                <div className="p-6 flex flex-col gap-5">
                    {/* Header row */}
                    <div className="flex items-start justify-between gap-3">
                        <div className="flex-1 min-w-0">
                            <h2 className="text-xl font-semibold leading-tight truncate">{event.title}</h2>
                            <div className="flex items-center gap-1.5 mt-1 text-sm text-muted-foreground">
                                <User className="w-3.5 h-3.5 flex-shrink-0" />
                                <span>{event.creatorName || "Unknown"}</span>
                                {isOwner && (
                                    <span className="text-xs bg-primary/10 text-primary px-1.5 py-0.5 rounded-full font-medium">
                                        you
                                    </span>
                                )}
                            </div>
                        </div>
                        <div className="flex items-center gap-1 flex-shrink-0">
                            {isOwner && (
                                <>
                                    <Button variant="ghost" size="icon-sm" onClick={() => onEdit(event)} title="Edit event">
                                        <Pencil className="w-4 h-4" />
                                    </Button>
                                    <Button
                                        variant="ghost" size="icon-sm"
                                        className="text-destructive hover:text-destructive"
                                        onClick={() => onDelete(event.id)}
                                        title="Delete event"
                                    >
                                        <Trash2 className="w-4 h-4" />
                                    </Button>
                                </>
                            )}
                            <Button variant="ghost" size="icon-sm" onClick={onClose}>
                                <X className="w-4 h-4" />
                            </Button>
                        </div>
                    </div>

                    {/* Meta info */}
                    <div className="flex flex-col gap-2.5">
                        <div className="flex items-center gap-2.5 text-sm">
                            <CalendarDays className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                            <span>{formatDate(d)}</span>
                        </div>
                        {hasTime(event.date) && (
                            <div className="flex items-center gap-2.5 text-sm">
                                <Clock className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                                <span>{formatTime(d)}</span>
                            </div>
                        )}
                        {event.location && (
                            <div className="flex items-center gap-2.5 text-sm">
                                <MapPin className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                                {event.location.startsWith("http") ? (
                                    <a
                                        href={event.location}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-primary underline underline-offset-3 truncate"
                                    >
                                        {event.location}
                                    </a>
                                ) : (
                                    <span>{event.location}</span>
                                )}
                            </div>
                        )}
                    </div>

                    {/* Description */}
                    {event.description && (
                        <p className="text-sm text-muted-foreground leading-relaxed border-t border-border/50 pt-4">
                            {event.description}
                        </p>
                    )}

                    {/* Attendees + RSVP */}
                    <div className="border-t border-border/50 pt-4 flex flex-col gap-3">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2 text-sm font-medium">
                                <Users className="w-4 h-4 text-muted-foreground" />
                                <span>Attendees</span>
                            </div>
                            <div className="flex items-center gap-2 text-xs">
                                {goingCount > 0 && <span className="text-emerald-500 font-medium">✓ {goingCount}</span>}
                                {lateCount > 0 && <span className="text-amber-500 font-medium">⏱ {lateCount}</span>}
                                {notGoingCount > 0 && <span className="text-rose-500 font-medium">✗ {notGoingCount}</span>}
                            </div>
                        </div>

                        <MemberList members={members} currentUserId={currentUserId} />

                        {/* RSVP controls */}
                        <div className="pt-2 border-t border-border/30">
                            {rsvpStatus ? (
                                <div className="flex items-center justify-between">
                                    <span className="text-sm text-muted-foreground">
                                        Your RSVP:{" "}
                                        <span className={`font-medium ${
                                            rsvpStatus === "going" ? "text-emerald-500" :
                                                rsvpStatus === "late"  ? "text-amber-500"  : "text-rose-500"
                                        }`}>
                                            {rsvpStatus === "going" ? "Going ✓" :
                                                rsvpStatus === "late"  ? "Coming late" : "Not going"}
                                        </span>
                                    </span>
                                    <Button variant="ghost" size="xs" onClick={onRemoveRsvp} disabled={rsvpLoading}>
                                        <LogOut className="w-3 h-3 mr-1" /> Remove
                                    </Button>
                                </div>
                            ) : (
                                <div className="flex flex-col gap-2">
                                    <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
                                        Will you attend?
                                    </span>
                                    <div className="flex gap-2 flex-wrap">
                                        <Button
                                            size="sm" variant="outline"
                                            className="border-emerald-500/30 text-emerald-500 hover:bg-emerald-500/10 hover:border-emerald-500"
                                            onClick={() => onRsvp("going")}
                                            disabled={rsvpLoading}
                                        >
                                            <Check className="w-3.5 h-3.5 mr-1" /> Going
                                        </Button>
                                        <Button
                                            size="sm" variant="outline"
                                            className="border-amber-500/30 text-amber-500 hover:bg-amber-500/10 hover:border-amber-500"
                                            onClick={() => setShowLateInput(v => !v)}
                                            disabled={rsvpLoading}
                                        >
                                            <Clock className="w-3.5 h-3.5 mr-1" /> Coming late
                                        </Button>
                                        <Button
                                            size="sm" variant="outline"
                                            className="border-rose-500/30 text-rose-500 hover:bg-rose-500/10 hover:border-rose-500"
                                            onClick={() => onRsvp("not_going")}
                                            disabled={rsvpLoading}
                                        >
                                            <X className="w-3.5 h-3.5 mr-1" /> Not going
                                        </Button>
                                    </div>
                                    {showLateInput && (
                                        <div className="flex items-center gap-2 mt-1">
                                            <span className="text-sm text-muted-foreground">~</span>
                                            <Input
                                                type="number" min={1} max={180}
                                                value={lateMinutes}
                                                onChange={e => setLateMinutes(Number(e.target.value))}
                                                className="h-8 w-20 text-sm"
                                            />
                                            <span className="text-sm text-muted-foreground">min late</span>
                                            <Button
                                                size="sm"
                                                onClick={() => { onRsvp("late", lateMinutes); setShowLateInput(false) }}
                                                disabled={rsvpLoading}
                                            >
                                                Confirm
                                            </Button>
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    )
}

function MemberList({ members, currentUserId }: { members: EventMember[], currentUserId: string | null }) {
    if (members.length === 0) {
        return <p className="text-xs text-muted-foreground">No RSVPs yet.</p>
    }

    return (
        <div className="flex flex-col gap-1.5 max-h-32 overflow-y-auto">
            {members.map(m => (
                <div key={m.userId} className="flex items-center justify-between text-sm">
                    <div className="flex items-center gap-2">
                        <div className="w-6 h-6 rounded-full bg-muted flex items-center justify-center text-[10px] font-semibold uppercase">
                            {m.username?.[0] || "?"}
                        </div>
                        <span>{m.username || "Unknown"}</span>
                        {m.userId === currentUserId && (
                            <span className="text-xs text-muted-foreground">(you)</span>
                        )}
                    </div>
                    <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                        m.status === "going" ? "bg-emerald-500/10 text-emerald-500" :
                            m.status === "late"  ? "bg-amber-500/10 text-amber-500"    :
                                "bg-rose-500/10 text-rose-500"
                    }`}>
                        {m.status === "going" ? "Going" :
                            m.status === "late"  ? `Late ~${m.lateMinutes ?? "?"}min` :
                                "Not going"}
                    </span>
                </div>
            ))}
        </div>
    )
}