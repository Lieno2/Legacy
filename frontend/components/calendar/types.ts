export interface CalendarEvent {
    id: number
    title: string
    description?: string | null
    date: string
    location?: string | null
    color?: string | null
    createdBy: string
    creatorName?: string | null
    createdAt?: string
    private?: boolean
}

export interface EventMember {
    userId: string
    username: string | null
    status: string
    lateMinutes: number | null
    joinedAt: string | null
}

export type RsvpStatus = "going" | "late" | "not_going"

export const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]

export const MONTHS = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
]

export const EVENT_COLORS = [
    "bg-blue-500",
    "bg-violet-500",
    "bg-emerald-500",
    "bg-rose-500",
    "bg-amber-500",
    "bg-cyan-500",
    "bg-pink-500",
]

export const DEFAULT_COLOR = EVENT_COLORS[0]

export const COLOR_HEX: Record<string, string> = {
    "bg-blue-500":    "#3b82f6",
    "bg-violet-500":  "#8b5cf6",
    "bg-emerald-500": "#10b981",
    "bg-rose-500":    "#f43f5e",
    "bg-amber-500":   "#f59e0b",
    "bg-cyan-500":    "#06b6d4",
    "bg-pink-500":    "#ec4899",
}
