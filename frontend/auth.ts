// auth.ts
import NextAuth from "next-auth"
import Credentials from "next-auth/providers/credentials"
import { db } from "@/db"
import { users } from "@/db/schema"
import { eq } from "drizzle-orm"
import bcrypt from "bcryptjs"

export const { handlers, signIn, signOut, auth } = NextAuth({
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

                return {
                    id: String(user.id),
                    email: user.email,
                    name: user.username,
                    perms: user.perms,
                }
            }
        })
    ],
    pages: {
        signIn: "/login",
    },
    session: { strategy: "jwt" },
    callbacks: {
        async jwt({ token, user }) {
            if (user) {
                token.id = user.id
                token.perms = (user as any).perms
            }
            return token
        },
        async session({ session, token }) {
            if (token && session.user) {
                session.user.id = token.id as string
                session.user.perms = token.perms as number
            }
            return session
        },
        authorized({ auth, request: { nextUrl } }) {
            const isLoggedIn = !!auth?.user
            const isOnLoginPage = nextUrl.pathname === "/login"
            const isOnAdminPage = nextUrl.pathname.startsWith("/admin")

            if (!isLoggedIn && !isOnLoginPage) return false
            if (isOnAdminPage && (auth?.user as any)?.perms < 999) return false

            return true
        },
    },
})