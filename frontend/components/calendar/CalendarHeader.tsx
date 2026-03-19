"use client"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ChevronLeft, ChevronRight, CalendarDays, Search, X, User } from "lucide-react"
import Link from "next/link"
import { MONTHS } from "./types"

interface CalendarHeaderProps {
    year: number
    month: number
    search: string
    onSearchChange: (value: string) => void
    onPrevMonth: () => void
    onNextMonth: () => void
    onGoToday: () => void
}

export function CalendarHeader({
                                   year, month, search, onSearchChange, onPrevMonth, onNextMonth, onGoToday,
                               }: CalendarHeaderProps) {
    return (
        <div className="flex items-center justify-between px-5 py-3 border-b border-border/50 bg-background/80 backdrop-blur-sm sticky top-0 z-10">
            <div className="flex items-center gap-4">
                <div className="flex items-center gap-1.5">
                    <CalendarDays className="w-5 h-5 text-primary" />
                    <h1 className="text-lg font-semibold tracking-tight">
                        {MONTHS[month]}{" "}
                        <span className="text-muted-foreground font-normal text-base">{year}</span>
                    </h1>
                </div>
                <div className="flex items-center gap-1">
                    <Button variant="ghost" size="icon-sm" onClick={onPrevMonth}>
                        <ChevronLeft className="w-4 h-4" />
                    </Button>
                    <Button variant="outline" size="xs" onClick={onGoToday}>Today</Button>
                    <Button variant="ghost" size="icon-sm" onClick={onNextMonth}>
                        <ChevronRight className="w-4 h-4" />
                    </Button>
                </div>
            </div>

            <div className="flex items-center gap-2">
                <div className="relative">
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
                    <Input
                        className="pl-8 h-8 w-48 text-xs"
                        placeholder="Search events…"
                        value={search}
                        onChange={e => onSearchChange(e.target.value)}
                    />
                    {search && (
                        <button
                            onClick={() => onSearchChange("")}
                            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                        >
                            <X className="w-3 h-3" />
                        </button>
                    )}
                </div>
                <Link href="/account">
                    <Button variant="ghost" size="icon-sm" title="Account settings">
                        <User className="w-4 h-4" />
                    </Button>
                </Link>
            </div>
        </div>
    )
}