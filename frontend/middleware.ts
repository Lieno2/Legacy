import { auth } from "@/auth"
import { NextResponse } from "next/server"
import type { NextRequest } from "next/server"

export async function middleware(req: NextRequest) {
    const session = await auth()
    const { pathname } = req.nextUrl

    // Not logged in → redirect to login
    if (!session?.user) {
        if (pathname === "/login") return NextResponse.next()
        return NextResponse.redirect(new URL("/login", req.url))
    }

    // Logged in, trying to access login → redirect to calendar
    if (pathname === "/login") {
        return NextResponse.redirect(new URL("/calendar", req.url))
    }

    // Admin routes → require perms >= 999
    if (pathname.startsWith("/admin")) {
        const perms = (session.user as any).perms ?? 0
        if (perms < 999) {
            return NextResponse.redirect(new URL("/calendar", req.url))
        }
    }

    return NextResponse.next()
}

export const config = {
    matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
}