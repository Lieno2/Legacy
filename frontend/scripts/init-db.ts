#!/usr/bin/env tsx

import { seedAdminUser } from "../db/seed"

async function main() {
    console.log("🚀 Initializing database...")
    await seedAdminUser()
    console.log("✓ Database initialization complete\n")
    process.exit(0)
}

main().catch((error) => {
    console.error("Database initialization failed:", error)
    process.exit(1)
})
