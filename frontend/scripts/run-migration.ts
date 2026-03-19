#!/usr/bin/env tsx

import postgres from "postgres"
import { readFileSync } from "fs"
import { join } from "path"

const sql = postgres(process.env.DATABASE_URL!)

async function runMigration() {
    console.log("Running migration to fix user ID types...")

    const migrationSQL = readFileSync(
        join(__dirname, "../db/migrations/0002_fix_user_id_type.sql"),
        "utf-8"
    )

    try {
        await sql.unsafe(migrationSQL)
        console.log("✓ Migration completed successfully")
    } catch (error) {
        console.error("Migration failed:", error)
        throw error
    } finally {
        await sql.end()
    }
}

runMigration().catch((error) => {
    console.error(error)
    process.exit(1)
})
