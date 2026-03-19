import { db } from "./index"
import { users } from "./schema"
import { eq } from "drizzle-orm"
import bcrypt from "bcryptjs"

const ADMIN_EMAIL = process.env.ADMIN_EMAIL || "admin@legacy.local"
const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || "admin123"

export async function seedAdminUser() {
    try {
        // Check if admin user already exists
        const existingAdmin = await db
            .select()
            .from(users)
            .where(eq(users.email, ADMIN_EMAIL))
            .then(r => r[0])

        if (existingAdmin) {
            console.log("✓ Admin user already exists")
            return null
        }

        // Hash password
        const passwordHash = await bcrypt.hash(ADMIN_PASSWORD, 10)

        // Create admin user
        await db.insert(users).values({
            username: "admin",
            email: ADMIN_EMAIL,
            passwordHash,
            perms: 999, // Admin permission level
        })

        console.log("\n" + "=".repeat(60))
        console.log("🔐 ADMIN USER CREATED")
        console.log("=".repeat(60))
        console.log(`Email:    ${ADMIN_EMAIL}`)
        console.log(`Password: ${ADMIN_PASSWORD}`)
        console.log("=".repeat(60))
        console.log("⚠️  Change the password in production!")
        console.log("=".repeat(60) + "\n")

        return { email: ADMIN_EMAIL, password: ADMIN_PASSWORD }
    } catch (error) {
        console.error("Failed to seed admin user:", error)
        return null
    }
}
