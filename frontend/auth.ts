// auth.ts  ← at the project root
import NextAuth from "next-auth"
import { DrizzleAdapter } from "@auth/drizzle-adapter"
import Credentials from "next-auth/providers/credentials"
import { db } from "@/db"
import { users } from "@/db/schema"
import { eq } from "drizzle-orm"
import bcrypt from "bcryptjs"

export const { handlers, signIn, signOut, auth } = NextAuth({
    adapter: DrizzleAdapter(db),
    providers: [
        Credentials({
            credentials: {
                email: { label: "Email", type: "email" },
                password: { label: "Password", type: "password" },
            },
            async authorize(credentials) {
                if (!credentials?.email || !credentials?.password) return null

                const user = await db
                    .select()
                    .from(users)
                    .where(eq(users.email, credentials.email as string))
                    .then(r => r[0])

                if (!user) return null

                const valid = await bcrypt.compare(credentials.password as string, user.passwordHash)
                if (!valid) return null

                return { id: String(user.id), email: user.email, name: user.username }
            }
        })
    ],
    pages: {
        signIn: "/login",
    },
    session: { strategy: "jwt" },
    callbacks: {
        authorized({ auth, request: { nextUrl } }) {
            const isLoggedIn = !!auth?.user
            const isOnLoginPage = nextUrl.pathname === "/login"

            if (!isLoggedIn && !isOnLoginPage) {
                return false // Redirect to login page
            }

            return true
        },
    },
})