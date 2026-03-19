# Legacy Backend

Axum + Rust backend for the Legacy calendar app.

## Stack

- **Axum** — HTTP framework
- **SQLx** — async PostgreSQL driver (uses the same DB as the original Next.js app)
- **Redis** — refresh token store for true session invalidation
- **jsonwebtoken** — JWT access tokens (15 min expiry)
- **bcrypt** — password hashing

## Setup

```bash
cp .env.example .env
# Fill in DATABASE_URL, REDIS_URL, JWT_SECRET

cargo run
# Listens on http://0.0.0.0:3001
```

## API Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/login` | — | Login, returns access + refresh token |
| POST | `/api/auth/logout` | — | Revoke refresh token |
| POST | `/api/auth/refresh` | — | Get new access token via refresh token |
| GET | `/api/auth/me` | ✓ | Current user info |
| GET | `/api/events` | ✓ | List own + RSVPed events |
| POST | `/api/events` | ✓ | Create event |
| PUT | `/api/events/:id` | ✓ | Update own event |
| DELETE | `/api/events/:id` | ✓ | Delete own event |
| GET | `/api/rsvp?event_id=X` | ✓ | List event members |
| POST | `/api/rsvp` | ✓ | RSVP to event (upsert) |
| DELETE | `/api/rsvp?event_id=X` | ✓ | Remove RSVP |
| GET | `/api/account` | ✓ | Get own profile |
| PUT | `/api/account` | ✓ | Update own profile/password |
| GET | `/api/admin/users` | Admin | List all users |
| POST | `/api/admin/users` | Admin | Create user |
| PUT | `/api/admin/users` | Admin | Update user |
| DELETE | `/api/admin/users?id=X` | Admin | Delete user |
| GET | `/api/admin/events` | Admin | List all events |
| DELETE | `/api/admin/events?id=X` | Admin | Delete any event |

## Auth Flow

1. `POST /api/auth/login` → returns `access_token` (JWT, 15min) + `refresh_token` (UUID, 7 days stored in Redis)
2. Frontend stores both; sends `Authorization: Bearer <access_token>` on every request
3. When access token expires → `POST /api/auth/refresh` with `refresh_token` → get new `access_token`
4. `POST /api/auth/logout` → deletes the refresh token from Redis, session truly dead
