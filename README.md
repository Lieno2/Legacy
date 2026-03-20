# Legacy Calendar

A self-hosted shared calendar for friends and small teams. Create events, RSVP, run date polls, manage private events with invite lists, and get Discord notifications — all in a clean, fast interface.

---

## Stack

| Layer | Technology |
|---|---|
| Frontend | SvelteKit 5 + TypeScript + Tailwind CSS |
| Backend | Rust (Axum 0.7) |
| Database | PostgreSQL |
| Cache / Sessions | Redis (refresh token storage) |
| Auth | JWT — access token (15min) + refresh token (7 days, stored in Redis) |
| API Docs | Scalar UI at `/scalar` |

---

## Features

### Calendar
- Month view with event chips per day
- Click any day to create an event, click an event to open details
- Search events by title, description or location
- **Happening Soon** banner — events in the next 48 hours shown as pills
- Keyboard shortcuts: `n` new event · `t` today · `←/→` change month · `Esc` close
- Dark / light theme toggle

### Events
- Title, description, date, time, location, color, private toggle
- **@ mention** users in descriptions
- **Map picker** — search via OpenStreetMap/Nominatim, shows preview
- **Open in Maps** link (Google Maps) in event detail
- **Public share link** — copy a shareable URL for any public event (no login needed to view)
- Undo delete — 5 second grace period after deleting an event

### RSVPs
- Going / Coming Late (with configurable minutes) / Not Going
- Attendee list with avatars and status badges
- Poll integration — if a date poll exists, users are asked to vote before RSVPing

### Private Events
- Mark any event as private on creation
- Creator can add/remove specific users who can see and RSVP to it
- Invited users see it in their calendar like any other event
- Private events are never sent to Discord

### Event Polls
- Creator can attach a date/option poll to any event
- Single or multiple choice
- Users are prompted to vote when they RSVP (Going or Late)
- Results shown as a bar chart with voter breakdown per option
- Poll can be edited or deleted by the creator

### Profiles
- Username, email, password change
- **Profile picture** — upload any image, cropped and resized to 128×128 client-side (stored as base64, no file server needed)
- Avatar shown in navbar and RSVP attendee list

### Admin Panel
- **Users** — create, edit, delete users, set admin role
- **Events** — view and delete any event across all users
- **Discord** — webhook notifications with embed or plain text format, per-action message templates, role ping with placement control, live preview
- **Stats** — events per month bar chart, most active users, RSVP breakdown
- **Audit Log** — full history of all create/update/delete actions, with one-click revert for deleted events

---

## Project Structure

```
Legacy/
├── backend/          Rust/Axum API server
│   ├── src/
│   │   ├── main.rs           Entry point, router, CORS
│   │   ├── config.rs         Env var config
│   │   ├── auth.rs           JWT generation/validation, Redis refresh tokens
│   │   ├── db.rs             PgPool + Redis client setup
│   │   ├── error.rs          AppError type with HTTP response mapping
│   │   ├── models.rs         Shared DB structs (User, Event, EventMember...)
│   │   ├── setup.rs          Admin seed account on startup
│   │   └── routes/
│   │       ├── auth.rs       login, logout, refresh, me
│   │       ├── events.rs     list, create, update, delete + Discord notify
│   │       ├── public_events.rs  Public share page endpoint + Discord helper
│   │       ├── rsvp.rs       list, upsert, remove
│   │       ├── invites.rs    Private event access management
│   │       ├── polls.rs      Create, answer, delete polls + voter breakdown
│   │       ├── account.rs    Get/update profile, avatar upload
│   │       └── admin/
│   │           ├── users.rs
│   │           ├── events.rs
│   │           ├── discord.rs
│   │           ├── stats.rs
│   │           └── audit.rs  List + revert deleted events
│   └── migrations/
│       ├── 0001_initial.sql
│       ├── 0002_invited_status.sql
│       ├── 0003_settings_audit.sql
│       ├── 0004_event_polls.sql
│       ├── 0005_poll_multi_choice.sql
│       ├── 0006_poll_allow_multiple.sql
│       ├── 0007_share_token.sql
│       └── 0008_avatar.sql
│
└── frontend/         SvelteKit app
    └── src/
        ├── routes/
        │   ├── +layout.svelte      Auth init, ModeWatcher (theme)
        │   ├── login/
        │   ├── calendar/           Main calendar view
        │   ├── account/            Profile settings + avatar upload
        │   ├── admin/              Admin panel (Users/Events/Discord/Stats/Audit)
        │   └── event/[id]/         Public share page (no login needed)
        └── lib/
            ├── api.ts              apiFetch with auto JWT + 401 refresh
            ├── stores.ts           Auth store, theme store
            ├── types.ts            Shared TypeScript interfaces
            ├── utils.ts            Date/calendar helpers
            └── components/
                ├── Avatar.svelte
                ├── EventDetail.svelte
                ├── EventModal.svelte
                ├── HappeningSoon.svelte
                ├── PollEditor.svelte
                ├── PollResults.svelte
                ├── PollAnswerModal.svelte
                ├── InviteSection.svelte
                ├── MemberList.svelte
                ├── RsvpBar.svelte
                └── admin/
                    ├── AdminUsers.svelte
                    ├── AdminEvents.svelte
                    ├── AdminDiscord.svelte
                    ├── AdminStats.svelte
                    └── AdminAudit.svelte
```

---

## API Reference

Full interactive docs available at `http://localhost:3001/scalar` when the backend is running.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/login` | — | Login, returns access + refresh tokens |
| POST | `/api/auth/logout` | — | Revoke refresh token |
| POST | `/api/auth/refresh` | — | Get new access token |
| GET | `/api/auth/me` | ✓ | Current user profile |
| GET | `/api/events` | ✓ | List visible events (public + own private + invited) |
| POST | `/api/events` | ✓ | Create event |
| PUT | `/api/events/:id` | ✓ | Update event (creator only) |
| DELETE | `/api/events/:id` | ✓ | Delete event (creator only) |
| GET | `/api/events/public/:token` | — | Public share page data |
| GET | `/api/rsvp?event_id=` | ✓ | List RSVPs for an event |
| POST | `/api/rsvp` | ✓ | Add/update RSVP |
| DELETE | `/api/rsvp?event_id=` | ✓ | Remove RSVP |
| GET | `/api/invites?event_id=` | ✓ | List invited users (creator only) |
| POST | `/api/invites` | ✓ | Invite a user to a private event |
| DELETE | `/api/invites` | ✓ | Remove invite |
| GET | `/api/invites/search?q=&event_id=` | ✓ | Search users to invite |
| GET | `/api/polls?event_id=` | ✓ | Get poll for an event |
| POST | `/api/polls` | ✓ | Create/update poll |
| DELETE | `/api/polls?event_id=` | ✓ | Delete poll |
| POST | `/api/polls/answer` | ✓ | Submit poll vote |
| GET | `/api/polls/voters?poll_id=` | ✓ | Get per-choice voter list |
| GET | `/api/account` | ✓ | Get own profile |
| PUT | `/api/account` | ✓ | Update profile (username, email, password, avatar) |
| GET | `/api/admin/users` | Admin | List all users |
| POST | `/api/admin/users` | Admin | Create user |
| PUT | `/api/admin/users` | Admin | Update user |
| DELETE | `/api/admin/users?id=` | Admin | Delete user |
| GET | `/api/admin/events` | Admin | List all events |
| DELETE | `/api/admin/events?id=` | Admin | Delete any event |
| GET | `/api/admin/discord` | Admin | Get Discord settings |
| POST | `/api/admin/discord` | Admin | Save Discord settings |
| GET | `/api/admin/stats` | Admin | Usage stats |
| GET | `/api/admin/audit` | Admin | Audit log (last 200 entries) |
| POST | `/api/admin/audit/revert` | Admin | Restore a deleted event |

---

## Setup

### Prerequisites

- Rust (stable) — [rustup.rs](https://rustup.rs)
- Bun — [bun.sh](https://bun.sh)
- PostgreSQL database (local or [Neon](https://neon.tech))
- Redis instance (local or [Upstash](https://upstash.com))

### Backend

```bash
cd backend

# Copy and fill in env vars
cp .env.example .env

# Run DB migrations
cargo install sqlx-cli --no-default-features --features postgres,rustls
sqlx migrate run

# Start the server (listens on :3001)
cargo run --release
```

**`backend/.env`**
```env
DATABASE_URL=postgres://user:password@localhost:5432/legacy
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your_long_random_secret_here
ACCESS_TOKEN_EXPIRY_SECS=900
REFRESH_TOKEN_EXPIRY_SECS=604800
FRONTEND_URL=http://localhost:5173

# Optional: seed an admin account on startup
SETUP_ACCOUNT_ENABLED=true
SETUP_ACCOUNT_EMAIL=admin@example.com
SETUP_ACCOUNT_PASSWORD=changeme123
```

### Frontend

```bash
cd frontend

# Copy and fill in env vars
cp .env.example .env

# Install dependencies
bun install

# Start dev server (listens on :5173)
bun run dev

# Or build for production
bun run build
```

**`frontend/.env`**
```env
PUBLIC_API_URL=http://localhost:3001
```

---

## Deployment

### Backend
```bash
cargo build --release
./target/release/legacy-backend
```

### Frontend
```bash
bun run build
# Outputs to build/ — serve with Node or behind Nginx
node build
```

### Nginx example (reverse proxy)
```nginx
server {
    listen 80;
    server_name yourdomain.com;

    # Frontend
    location / {
        proxy_pass http://localhost:5173;
    }

    # Backend API
    location /api {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## Environment Variables Reference

### Backend

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | ✓ | — | PostgreSQL connection string |
| `REDIS_URL` | ✓ | — | Redis connection string |
| `JWT_SECRET` | ✓ | — | Secret key for signing JWTs |
| `ACCESS_TOKEN_EXPIRY_SECS` | — | `900` | Access token lifetime (15 min) |
| `REFRESH_TOKEN_EXPIRY_SECS` | — | `604800` | Refresh token lifetime (7 days) |
| `FRONTEND_URL` | — | `http://localhost:5173` | CORS allowed origin (no trailing slash) |
| `SETUP_ACCOUNT_ENABLED` | — | `false` | Seed admin account on startup |
| `SETUP_ACCOUNT_EMAIL` | — | `admin@legacy.local` | Seed account email |
| `SETUP_ACCOUNT_PASSWORD` | — | `admin123` | Seed account password |
| `SQLX_OFFLINE` | — | `false` | Use cached query metadata (for CI/offline builds) |

### Frontend

| Variable | Required | Default | Description |
|---|---|---|---|
| `PUBLIC_API_URL` | — | `http://localhost:3001` | Backend base URL |

---

## Discord Notifications

Configure in the Admin Panel → Discord tab.

- **Embed mode** — rich card with color bar, structured fields (date, location), footer, timestamp
- **Plain text mode** — full control with Markdown and placeholders
- **Role ping** — optionally ping a Discord role, placed before or after the message
- **Per-action templates** — separate messages for event created, updated, and deleted

Available placeholders: `{event.title}` `{event.creator}` `{event.date}` `{event.time}` `{event.location}` `{event.description}` `{event.url}`

Private events never trigger Discord notifications.

---

## License

MIT
