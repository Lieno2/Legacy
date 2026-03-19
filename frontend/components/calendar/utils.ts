export function toKey(date: Date): string {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`
}

export function getDaysInMonth(year: number, month: number): number {
    return new Date(year, month + 1, 0).getDate()
}

export function getFirstDayOfMonth(year: number, month: number): number {
    return new Date(year, month, 1).getDay()
}

export function formatTime(date: Date): string {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
}

export function formatDate(date: Date): string {
    return date.toLocaleDateString([], { weekday: "long", year: "numeric", month: "long", day: "numeric" })
}

export function hasTime(dateStr: string): boolean {
    const d = new Date(dateStr)
    return d.getHours() !== 0 || d.getMinutes() !== 0
}

export function buildGridCells(year: number, month: number): (number | null)[] {
    const daysInMonth = getDaysInMonth(year, month)
    const firstDay = getFirstDayOfMonth(year, month)
    const cells: (number | null)[] = [
        ...Array(firstDay).fill(null),
        ...Array.from({ length: daysInMonth }, (_, i) => i + 1),
    ]
    while (cells.length % 7 !== 0) cells.push(null)
    return cells
}

export function buildDateISO(dateKey: string, time: string): string {
    if (time) return new Date(`${dateKey}T${time}:00`).toISOString()
    return new Date(`${dateKey}T00:00:00`).toISOString()
}
