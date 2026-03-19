"use client"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Pencil, Plus } from "lucide-react"
import { CalendarEvent, EVENT_COLORS, DEFAULT_COLOR } from "./types"

interface EventFormState {
    title: string
    description: string
    location: string
    time: string
    color: string
    isPrivate: boolean
    selectedDate: string | null
}

interface EventFormDialogProps {
    open: boolean
    editingEvent: CalendarEvent | null
    form: EventFormState
    saving: boolean
    onChange: (patch: Partial<EventFormState>) => void
    onSave: () => void
    onClose: () => void
}

export function EventFormDialog({
    open, editingEvent, form, saving, onChange, onSave, onClose,
}: EventFormDialogProps) {
    return (
        <Dialog open={open} onOpenChange={onClose}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        {editingEvent
                            ? <><Pencil className="w-4 h-4" /> Edit Event</>
                            : <><Plus className="w-4 h-4" /> New Event</>
                        }
                        <span className="text-muted-foreground font-normal text-sm">
                            — {form.selectedDate}
                        </span>
                    </DialogTitle>
                </DialogHeader>

                <div className="flex flex-col gap-4 py-1">
                    {/* Title */}
                    <div className="flex flex-col gap-1.5">
                        <Label htmlFor="evt-title">
                            Title <span className="text-destructive">*</span>
                        </Label>
                        <Input
                            id="evt-title"
                            placeholder="e.g. Team meeting"
                            value={form.title}
                            onChange={e => onChange({ title: e.target.value })}
                            autoFocus
                        />
                    </div>

                    {/* Date + Time */}
                    <div className="grid grid-cols-2 gap-3">
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor="evt-date">Date</Label>
                            <Input
                                id="evt-date"
                                type="date"
                                value={form.selectedDate || ""}
                                onChange={e => onChange({ selectedDate: e.target.value })}
                                className="text-sm"
                            />
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor="evt-time">
                                Time{" "}
                                <span className="text-muted-foreground text-xs">(optional)</span>
                            </Label>
                            <Input
                                id="evt-time"
                                type="time"
                                value={form.time}
                                onChange={e => onChange({ time: e.target.value })}
                                className="text-sm"
                            />
                        </div>
                    </div>

                    {/* Description */}
                    <div className="flex flex-col gap-1.5">
                        <Label htmlFor="evt-desc">Description</Label>
                        <Textarea
                            id="evt-desc"
                            placeholder="Add details…"
                            value={form.description}
                            onChange={e => onChange({ description: e.target.value })}
                            rows={3}
                        />
                    </div>

                    {/* Location */}
                    <div className="flex flex-col gap-1.5">
                        <Label htmlFor="evt-loc">Location</Label>
                        <Input
                            id="evt-loc"
                            placeholder="e.g. Office or https://meet.google.com/…"
                            value={form.location}
                            onChange={e => onChange({ location: e.target.value })}
                        />
                    </div>

                    {/* Color + Private */}
                    <div className="flex items-center justify-between">
                        <div className="flex flex-col gap-1.5">
                            <Label>Color</Label>
                            <div className="flex gap-1.5">
                                {EVENT_COLORS.map(color => (
                                    <button
                                        key={color}
                                        onClick={() => onChange({ color })}
                                        className={`w-5 h-5 rounded-full ${color} transition-all ${
                                            form.color === color
                                                ? "ring-2 ring-offset-2 ring-offset-background ring-primary scale-110"
                                                : "opacity-70 hover:opacity-100"
                                        }`}
                                    />
                                ))}
                            </div>
                        </div>
                        <div className="flex flex-col gap-1.5 items-end">
                            <Label htmlFor="evt-private">Private</Label>
                            <button
                                id="evt-private"
                                role="switch"
                                aria-checked={form.isPrivate}
                                onClick={() => onChange({ isPrivate: !form.isPrivate })}
                                className={`relative w-9 h-5 rounded-full transition-colors ${
                                    form.isPrivate ? "bg-primary" : "bg-muted"
                                }`}
                            >
                                <span className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform ${
                                    form.isPrivate ? "translate-x-4" : ""
                                }`} />
                            </button>
                        </div>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={onClose} disabled={saving}>Cancel</Button>
                    <Button onClick={onSave} disabled={!form.title.trim() || saving}>
                        {saving ? "Saving…" : editingEvent ? "Update" : "Create"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
