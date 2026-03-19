<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User, Event } from '$lib/types';
  import {
    ArrowLeft, Shield, Users, CalendarDays, Plus, Pencil,
    Trash2, Search, X, Eye, EyeOff, Bell, BarChart2, Hash,
    AlertTriangle, Lock, RotateCcw, ClipboardList, Loader2,
    CheckCircle2, Clock, XCircle, UserCheck, Save, Webhook
  } from 'lucide-svelte';

  type Tab = 'users' | 'events' | 'discord' | 'stats' | 'audit';
  let tab: Tab = 'users';

  // ── data ──────────────────────────────────────────────────────────────────
  let users:  User[]  = [];
  let events: Event[] = [];
  let usersLoading = true, eventsLoading = true;
  let userSearch = '', eventSearch = '';

  // ── user dialog ───────────────────────────────────────────────────────────
  let userDialogOpen = false;
  let editingUser: User | null = null;
  let userForm = { username: '', email: '', password: '', perms: 0 };
  let showPw = false, userSaving = false, userError = '';

  // ── delete confirm ────────────────────────────────────────────────────────
  let deleteTarget: { type: 'user' | 'event'; id: string | number; name: string } | null = null;
  let deleteLoading = false;

  // ── discord ───────────────────────────────────────────────────────────────
  interface DiscordConfig {
    webhook_url: string; enabled: boolean; format: string;
    msg_created: string; msg_updated: string; msg_deleted: string;
  }
  let discord: DiscordConfig = {
    webhook_url: '', enabled: false, format: 'embed',
    msg_created: '📅 **{event.title}** has been created by {event.creator} on {event.date}{event.location}.',
    msg_updated: '✏️ **{event.title}** was updated by {event.creator}. New date: {event.date}{event.location}.',
    msg_deleted: '🗑️ **{event.title}** (was on {event.date}) has been deleted by {event.creator}.',
  };
  let discordLoading = false, discordSaving = false, discordSaved = false;
  let discordLoaded = false;

  // ── stats ─────────────────────────────────────────────────────────────────
  interface MonthStat   { month: string; count: number }
  interface ActiveUser  { username: string; rsvp_count: number }
  interface RsvpBreak   { going: number; late: number; not_going: number; invited: number }
  interface StatsData   { events_per_month: MonthStat[]; most_active_users: ActiveUser[]; rsvp_breakdown: RsvpBreak }
  let stats: StatsData | null = null;
  let statsLoading = false;
  let statsLoaded = false;

  // ── audit ─────────────────────────────────────────────────────────────────
  interface AuditEntry {
    id: number;
    user_id: string | null;
    username: string | null;
    action: string;
    target_type: string | null;
    target_id: string | null;
    target_name: string | null;
    metadata: Record<string, unknown> | null;
    created_at: string;
  }
  let audit: AuditEntry[] = [];
  let auditLoading = false;
  let auditLoaded = false;
  let reverting: number | null = null;

  // ── auth ready flag ───────────────────────────────────────────────────────
  // True once the layout has finished the /api/auth/me call
  let authReady = false;
  $: if (!$auth.loading) authReady = true;

  // ── lifecycle ─────────────────────────────────────────────────────────────
  onMount(async () => {
    // Wait until auth store is resolved before doing anything
    await waitForAuth();
    if (($auth.user?.perms ?? 0) < 999) { goto('/calendar'); return; }
    fetchUsers();
    fetchEvents();
  });

  function waitForAuth(): Promise<void> {
    return new Promise(resolve => {
      const unsub = auth.subscribe(a => {
        if (!a.loading) { unsub(); resolve(); }
      });
    });
  }

  async function fetchUsers()  { usersLoading  = true; try { users  = await apiFetch<User[]>('/api/admin/users');  } finally { usersLoading  = false; } }
  async function fetchEvents() { eventsLoading = true; try { events = await apiFetch<Event[]>('/api/admin/events'); } finally { eventsLoading = false; } }

  async function loadDiscord() {
    if (discordLoading) return;
    discordLoading = true;
    try { discord = await apiFetch<DiscordConfig>('/api/admin/discord'); discordLoaded = true; } catch { /* use defaults */ }
    finally { discordLoading = false; }
  }

  async function saveDiscord() {
    discordSaving = true; discordSaved = false;
    try {
      await apiFetch('/api/admin/discord', { method: 'POST', body: JSON.stringify(discord) });
      discordSaved = true;
      setTimeout(() => (discordSaved = false), 2500);
    } catch { /* ignore */ }
    finally { discordSaving = false; }
  }

  async function loadStats() {
    if (statsLoading) return;
    statsLoading = true;
    try { stats = await apiFetch<StatsData>('/api/admin/stats'); statsLoaded = true; } catch { }
    finally { statsLoading = false; }
  }

  async function loadAudit() {
    if (auditLoading) return;
    auditLoading = true;
    try { audit = await apiFetch<AuditEntry[]>('/api/admin/audit'); auditLoaded = true; } catch { }
    finally { auditLoading = false; }
  }

  async function revertAudit(id: number) {
    reverting = id;
    try {
      await apiFetch('/api/admin/audit/revert', { method: 'POST', body: JSON.stringify({ audit_id: id }) });
      await loadAudit();
      await fetchEvents();
    } catch { }
    finally { reverting = null; }
  }

  // Lazy-load per tab — only fires once authReady is true and data not yet loaded
  $: if (authReady && tab === 'discord' && !discordLoaded && !discordLoading) loadDiscord();
  $: if (authReady && tab === 'stats'   && !statsLoaded   && !statsLoading)   loadStats();
  $: if (authReady && tab === 'audit'   && !auditLoaded   && !auditLoading)   loadAudit();

  // ── user CRUD ─────────────────────────────────────────────────────────────
  function openCreateUser() { editingUser = null; userForm = { username: '', email: '', password: '', perms: 0 }; userError = ''; showPw = false; userDialogOpen = true; }
  function openEditUser(u: User) { editingUser = u; userForm = { username: u.username, email: u.email, password: '', perms: u.perms }; userError = ''; showPw = false; userDialogOpen = true; }

  async function saveUser() {
    userError = '';
    if (!userForm.username.trim() || !userForm.email.trim()) { userError = 'Username and email are required'; return; }
    if (!editingUser && !userForm.password) { userError = 'Password is required'; return; }
    if (userForm.password && userForm.password.length < 8) { userError = 'Password must be at least 8 characters'; return; }
    userSaving = true;
    try {
      const payload = editingUser
        ? { id: editingUser.id, username: userForm.username, email: userForm.email, perms: userForm.perms, new_password: userForm.password || undefined }
        : { username: userForm.username, email: userForm.email, password: userForm.password, perms: userForm.perms };
      await apiFetch('/api/admin/users', { method: editingUser ? 'PUT' : 'POST', body: JSON.stringify(payload) });
      await fetchUsers();
      userDialogOpen = false;
    } catch (err: unknown) { userError = err instanceof Error ? err.message : 'Failed to save'; }
    finally { userSaving = false; }
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    deleteLoading = true;
    try {
      if (deleteTarget.type === 'user') {
        await apiFetch(`/api/admin/users?id=${deleteTarget.id}`, { method: 'DELETE' });
        await fetchUsers();
      } else {
        await apiFetch(`/api/admin/events?id=${deleteTarget.id}`, { method: 'DELETE' });
        await fetchEvents();
        if (tab === 'audit') await loadAudit();
      }
      deleteTarget = null;
    } finally { deleteLoading = false; }
  }

  // ── derived ───────────────────────────────────────────────────────────────
  $: filteredUsers  = users.filter(u  => u.username.toLowerCase().includes(userSearch.toLowerCase()) || u.email.toLowerCase().includes(userSearch.toLowerCase()));
  $: filteredEvents = events.filter(e => e.title.toLowerCase().includes(eventSearch.toLowerCase()) || (e.creator_name ?? '').toLowerCase().includes(eventSearch.toLowerCase()));
  $: adminCount    = users.filter(u => u.perms >= 999).length;
  $: upcomingCount = events.filter(e => new Date(e.date) > new Date()).length;
  $: privateCount  = events.filter(e => e.private).length;

  // ── stats helpers ─────────────────────────────────────────────────────────
  function barHeight(count: number, max: number) { return max === 0 ? 0 : Math.round((count / max) * 100); }
  $: maxMonth = stats ? Math.max(...stats.events_per_month.map(m => m.count), 1) : 1;
  $: rsvpTotal = stats ? (stats.rsvp_breakdown.going + stats.rsvp_breakdown.late + stats.rsvp_breakdown.not_going + stats.rsvp_breakdown.invited) : 0;

  function monthLabel(ym: string) {
    const [y, m] = ym.split('-');
    return new Date(+y, +m - 1).toLocaleDateString('en-US', { month: 'short' });
  }

  // ── audit helpers ─────────────────────────────────────────────────────────
  const ACTION_STYLE: Record<string, string> = {
    create: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    update: 'bg-blue-500/10    text-blue-400    border-blue-500/20',
    delete: 'bg-red-500/10     text-red-400     border-red-500/20',
  };

  function timeAgo(iso: string) {
    const diff = Date.now() - new Date(iso).getTime();
    const m = Math.floor(diff / 60000);
    if (m < 1)   return 'just now';
    if (m < 60)  return `${m}m ago`;
    const h = Math.floor(m / 60);
    if (h < 24)  return `${h}h ago`;
    return `${Math.floor(h / 24)}d ago`;
  }

  // ── style constants ───────────────────────────────────────────────────────
  const INPUT = 'h-9 w-full rounded-lg border border-input bg-muted/20 px-3 text-sm outline-none focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/50';
  const TEXTAREA = 'w-full rounded-lg border border-input bg-muted/20 px-3 py-2 text-sm outline-none focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/50 resize-none font-mono';

  const TABS: { id: Tab; label: string; icon: any }[] = [
    { id: 'users',   label: 'Users',     icon: Users         },
    { id: 'events',  label: 'Events',    icon: CalendarDays  },
    { id: 'discord', label: 'Discord',   icon: Hash          },
    { id: 'stats',   label: 'Stats',     icon: BarChart2     },
    { id: 'audit',   label: 'Audit Log', icon: ClipboardList },
  ];
</script>

<div class="min-h-screen bg-background text-foreground">
  <header class="border-b border-border px-5 py-3 flex items-center justify-between sticky top-0 bg-background/80 backdrop-blur-md z-10">
    <div class="flex items-center gap-2">
      <a href="/calendar" class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground">
        <ArrowLeft class="w-4 h-4" />
      </a>
      <Shield class="w-4 h-4 text-amber-400" />
      <h1 class="text-sm font-semibold">Admin Panel</h1>
    </div>
    <a href="/account" class="h-8 px-3 rounded-lg text-xs font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition inline-flex items-center">Account</a>
  </header>

  <div class="max-w-3xl mx-auto px-5 py-6 flex flex-col gap-6">

    <!-- Quick stats cards -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      {#each [
        { label: 'Total Users',  value: users.length,  color: '#3b82f6', icon: Users        },
        { label: 'Total Events', value: events.length, color: '#a855f7', icon: CalendarDays },
        { label: 'Upcoming',     value: upcomingCount, color: '#22c55e', icon: Bell         },
        { label: 'Private',      value: privateCount,  color: '#f97316', icon: Lock         },
      ] as stat}
        <div class="bg-card border border-border rounded-2xl p-4 flex items-center gap-3 overflow-hidden relative">
          <div class="absolute inset-0 opacity-5 rounded-2xl" style="background:{stat.color};"></div>
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background:{stat.color}18;">
            <svelte:component this={stat.icon} class="w-4 h-4" style="color:{stat.color};" />
          </div>
          <div>
            <div class="text-xl font-bold tabular-nums">{stat.value}</div>
            <div class="text-xs text-muted-foreground leading-tight">{stat.label}</div>
          </div>
        </div>
      {/each}
    </div>

    <!-- Tab bar -->
    <div class="flex gap-0.5 bg-muted/40 border border-border p-1 rounded-xl overflow-x-auto">
      {#each TABS as t}
        <button
          on:click={() => (tab = t.id)}
          class="flex-1 flex items-center justify-center gap-1.5 h-8 rounded-lg text-xs font-medium transition-all whitespace-nowrap px-2
                 {tab === t.id ? 'bg-card shadow-sm text-foreground border border-border/60' : 'text-muted-foreground hover:text-foreground'}"
        >
          <svelte:component this={t.icon} class="w-3.5 h-3.5" />
          {t.label}
        </button>
      {/each}
    </div>

    <!-- ════ USERS ════ -->
    {#if tab === 'users'}
      <div class="bg-card border border-border rounded-2xl overflow-hidden">
        <div class="p-5 border-b border-border flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="font-semibold text-sm">Users</h2>
              <p class="text-xs text-muted-foreground mt-0.5">{users.length} total · {adminCount} admin{adminCount !== 1 ? 's' : ''}</p>
            </div>
            <button on:click={openCreateUser}
              class="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition">
              <Plus class="w-3.5 h-3.5" /> New User
            </button>
          </div>
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
            <input class="{INPUT} pl-9" placeholder="Search by name or email..." bind:value={userSearch} />
            {#if userSearch}<button on:click={() => (userSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"><X class="w-3.5 h-3.5" /></button>{/if}
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if usersLoading}
            <div class="flex items-center justify-center gap-2 py-8 text-muted-foreground text-sm">
              <Loader2 class="w-4 h-4 animate-spin" /> Loading…
            </div>
          {:else if filteredUsers.length === 0}
            <div class="py-8 text-center text-sm text-muted-foreground">No users found</div>
          {:else}
            {#each filteredUsers as u}
              <div class="flex items-center gap-3 px-5 py-3 hover:bg-muted/30 transition group">
                <div class="w-8 h-8 rounded-full border border-border flex items-center justify-center text-xs font-bold uppercase shrink-0"
                  style="background:hsl({((u.username.charCodeAt(0)*47)%360)},40%,25%); color:hsl({((u.username.charCodeAt(0)*47)%360)},70%,70%);">
                  {u.username[0]}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-sm font-medium">{u.username}</span>
                    {#if u.perms >= 999}
                      <span class="inline-flex items-center gap-0.5 text-[10px] bg-amber-500/10 text-amber-400 border border-amber-500/20 px-1.5 py-0.5 rounded-full font-medium">
                        <Shield class="w-2.5 h-2.5" /> Admin
                      </span>
                    {/if}
                  </div>
                  <div class="text-xs text-muted-foreground truncate">{u.email}</div>
                </div>
                <div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition shrink-0">
                  <button on:click={() => openEditUser(u)} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition"><Pencil class="w-3.5 h-3.5" /></button>
                  <button on:click={() => (deleteTarget = { type: 'user', id: u.id, name: u.username })} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition"><Trash2 class="w-3.5 h-3.5" /></button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- ════ EVENTS ════ -->
    {#if tab === 'events'}
      <div class="bg-card border border-border rounded-2xl overflow-hidden">
        <div class="p-5 border-b border-border flex flex-col gap-3">
          <div>
            <h2 class="font-semibold text-sm">All Events</h2>
            <p class="text-xs text-muted-foreground mt-0.5">{events.length} total · {upcomingCount} upcoming</p>
          </div>
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
            <input class="{INPUT} pl-9" placeholder="Search events..." bind:value={eventSearch} />
            {#if eventSearch}<button on:click={() => (eventSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"><X class="w-3.5 h-3.5" /></button>{/if}
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if eventsLoading}
            <div class="flex items-center justify-center gap-2 py-8 text-muted-foreground text-sm">
              <Loader2 class="w-4 h-4 animate-spin" /> Loading…
            </div>
          {:else if filteredEvents.length === 0}
            <div class="py-8 text-center text-sm text-muted-foreground">No events found</div>
          {:else}
            {#each filteredEvents as e}
              {@const isPast = new Date(e.date) < new Date()}
              <div class="flex items-center gap-3 px-5 py-3 hover:bg-muted/30 transition group {isPast ? 'opacity-60' : ''}">
                <div class="w-1.5 h-10 rounded-full shrink-0" style="background:{e.color ?? '#6366f1'};"></div>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium truncate flex items-center gap-1.5">
                    {e.title}
                    {#if e.private}<Lock class="w-3 h-3 text-muted-foreground shrink-0" />{/if}
                    {#if isPast}<span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full">Past</span>{/if}
                  </div>
                  <div class="text-xs text-muted-foreground">
                    {new Date(e.date).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'})}
                    {#if e.location} · {e.location}{/if}
                    · by {e.creator_name ?? 'Unknown'}
                  </div>
                </div>
                <button on:click={() => (deleteTarget = { type: 'event', id: e.id, name: e.title })}
                  class="w-7 h-7 rounded-lg flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition shrink-0">
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- ════ DISCORD ════ -->
    {#if tab === 'discord'}
      {#if discordLoading}
        <div class="flex items-center justify-center gap-2 py-16 text-muted-foreground">
          <Loader2 class="w-5 h-5 animate-spin" /> Loading Discord settings…
        </div>
      {:else}
        <div class="flex flex-col gap-4">
          <div class="bg-card border border-border rounded-2xl p-5 flex items-center gap-4">
            <div class="w-11 h-11 rounded-2xl bg-[#5865f2]/10 border border-[#5865f2]/20 flex items-center justify-center shrink-0">
              <Hash class="w-5 h-5 text-[#5865f2]" />
            </div>
            <div class="flex-1 min-w-0">
              <h2 class="font-semibold text-sm">Discord Webhook</h2>
              <p class="text-xs text-muted-foreground mt-0.5">Send notifications to a Discord channel when events are created, updated or deleted.</p>
            </div>
            <label class="flex items-center gap-2 cursor-pointer shrink-0">
              <span class="text-xs text-muted-foreground">{discord.enabled ? 'On' : 'Off'}</span>
              <div class="relative">
                <input type="checkbox" bind:checked={discord.enabled} class="sr-only peer" />
                <div class="w-9 h-5 rounded-full border transition-colors bg-muted border-border peer-checked:bg-[#5865f2] peer-checked:border-[#5865f2]"></div>
                <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-transform peer-checked:translate-x-4"></div>
              </div>
            </label>
          </div>

          <div class="bg-card border border-border rounded-2xl p-5 flex flex-col gap-3">
            <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
              <Webhook class="w-3.5 h-3.5" /> Webhook URL
            </label>
            <input class={INPUT} bind:value={discord.webhook_url} placeholder="https://discord.com/api/webhooks/..." />
            <div class="flex flex-col gap-1.5 pt-1">
              <span class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Message Format</span>
              <div class="grid grid-cols-2 gap-2">
                <button type="button" on:click={() => (discord.format = 'embed')}
                  class="h-9 rounded-xl border text-sm font-medium transition
                         {discord.format === 'embed' ? 'border-[#5865f2]/50 bg-[#5865f2]/10 text-[#5865f2]' : 'border-border text-muted-foreground hover:bg-muted'}">
                  Embed
                </button>
                <button type="button" on:click={() => (discord.format = 'plain')}
                  class="h-9 rounded-xl border text-sm font-medium transition
                         {discord.format === 'plain' ? 'border-[#5865f2]/50 bg-[#5865f2]/10 text-[#5865f2]' : 'border-border text-muted-foreground hover:bg-muted'}">
                  Plain Text
                </button>
              </div>
            </div>
          </div>

          <div class="bg-card border border-border rounded-2xl p-5 flex flex-col gap-4">
            <div>
              <h3 class="text-sm font-semibold">Message Templates</h3>
              <p class="text-xs text-muted-foreground mt-0.5">
                Available placeholders:
                <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.title&#125;</code>
                <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.date&#125;</code>
                <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.location&#125;</code>
                <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.creator&#125;</code>
              </p>
            </div>
            {#each [
              { key: 'msg_created' as const, label: '✅ Event Created', emoji: '📅' },
              { key: 'msg_updated' as const, label: '✏️ Event Updated', emoji: '✏️' },
              { key: 'msg_deleted' as const, label: '🗑️ Event Deleted', emoji: '🗑️' },
            ] as tpl}
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-foreground">{tpl.label}</label>
                <textarea class="{TEXTAREA} h-20" bind:value={discord[tpl.key]}
                  placeholder="{tpl.emoji} Message for {tpl.label.toLowerCase()}..."
                ></textarea>
              </div>
            {/each}
          </div>

          <button on:click={saveDiscord} disabled={discordSaving}
            class="h-10 rounded-xl font-semibold text-sm transition active:scale-[0.97] disabled:opacity-50 flex items-center justify-center gap-2
                   {discordSaved ? 'bg-emerald-500/10 border border-emerald-500/20 text-emerald-400' : 'bg-[#5865f2] text-white hover:bg-[#4752c4]'}">
            {#if discordSaving}
              <Loader2 class="w-4 h-4 animate-spin" /> Saving…
            {:else if discordSaved}
              <CheckCircle2 class="w-4 h-4" /> Saved!
            {:else}
              <Save class="w-4 h-4" /> Save Discord Settings
            {/if}
          </button>
        </div>
      {/if}
    {/if}

    <!-- ════ STATS ════ -->
    {#if tab === 'stats'}
      {#if statsLoading}
        <div class="flex items-center justify-center gap-2 py-16 text-muted-foreground">
          <Loader2 class="w-5 h-5 animate-spin" /> Loading stats…
        </div>
      {:else if stats}
        <div class="flex flex-col gap-4">
          <div class="bg-card border border-border rounded-2xl p-5">
            <h2 class="font-semibold text-sm mb-4">Events per Month <span class="text-muted-foreground font-normal">(last 12 months)</span></h2>
            {#if stats.events_per_month.length === 0}
              <p class="text-sm text-muted-foreground">No events in the last 12 months.</p>
            {:else}
              <div class="flex items-end gap-1.5 h-36">
                {#each stats.events_per_month as m}
                  <div class="flex-1 flex flex-col items-center gap-1 group">
                    <span class="text-[10px] text-muted-foreground opacity-0 group-hover:opacity-100 transition tabular-nums">{m.count}</span>
                    <div class="w-full rounded-t-md bg-primary/80 hover:bg-primary transition-all duration-300"
                      style="height:{barHeight(m.count, maxMonth)}%;"
                    ></div>
                    <span class="text-[10px] text-muted-foreground">{monthLabel(m.month)}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div class="bg-card border border-border rounded-2xl p-5">
              <div class="flex items-center gap-2 mb-4">
                <UserCheck class="w-4 h-4 text-muted-foreground" />
                <h2 class="font-semibold text-sm">Most Active Users</h2>
              </div>
              {#if stats.most_active_users.length === 0}
                <p class="text-sm text-muted-foreground">No RSVP data yet.</p>
              {:else}
                <div class="flex flex-col gap-3">
                  {#each stats.most_active_users as u, i}
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold text-muted-foreground/50 w-4 tabular-nums">{i+1}</span>
                      <div class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold uppercase shrink-0"
                        style="background:hsl({(u.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(u.username.charCodeAt(0)*47)%360},70%,70%);">{u.username[0]}</div>
                      <span class="text-sm flex-1 truncate">{u.username}</span>
                      <span class="text-xs font-semibold tabular-nums text-primary">{u.rsvp_count}</span>
                    </div>
                    {#if i < stats.most_active_users.length - 1}
                      <div class="h-px bg-border/40"></div>
                    {/if}
                  {/each}
                </div>
              {/if}
            </div>

            <div class="bg-card border border-border rounded-2xl p-5">
              <div class="flex items-center gap-2 mb-4">
                <BarChart2 class="w-4 h-4 text-muted-foreground" />
                <h2 class="font-semibold text-sm">RSVP Breakdown</h2>
              </div>
              {#if rsvpTotal === 0}
                <p class="text-sm text-muted-foreground">No RSVPs yet.</p>
              {:else}
                {@const b = stats.rsvp_breakdown}
                <div class="flex flex-col gap-3">
                  {#each [
                    { label: 'Going',     value: b.going,     color: '#22c55e', icon: CheckCircle2 },
                    { label: 'Late',      value: b.late,      color: '#eab308', icon: Clock        },
                    { label: 'Not going', value: b.not_going, color: '#ef4444', icon: XCircle      },
                    { label: 'Invited',   value: b.invited,   color: '#6366f1', icon: UserCheck    },
                  ] as row}
                    <div class="flex flex-col gap-1">
                      <div class="flex items-center justify-between text-xs">
                        <span class="flex items-center gap-1.5 text-muted-foreground">
                          <svelte:component this={row.icon} class="w-3 h-3" style="color:{row.color};" />
                          {row.label}
                        </span>
                        <span class="font-medium tabular-nums">{row.value}</span>
                      </div>
                      <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                        <div class="h-full rounded-full transition-all duration-500"
                          style="width:{rsvpTotal ? (row.value/rsvpTotal*100).toFixed(1) : 0}%; background:{row.color};"
                        ></div>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <div class="py-16 text-center text-sm text-muted-foreground">Failed to load stats. <button on:click={() => { statsLoaded = false; }} class="underline hover:text-foreground transition">Retry</button></div>
      {/if}
    {/if}

    <!-- ════ AUDIT LOG ════ -->
    {#if tab === 'audit'}
      <div class="bg-card border border-border rounded-2xl overflow-hidden">
        <div class="p-5 border-b border-border flex items-center justify-between">
          <div>
            <h2 class="font-semibold text-sm">Audit Log</h2>
            <p class="text-xs text-muted-foreground mt-0.5">Last 200 actions. Deleted events can be reverted.</p>
          </div>
          <button on:click={() => { auditLoaded = false; }}  disabled={auditLoading}
            class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition disabled:opacity-40">
            <RotateCcw class="w-3.5 h-3.5 {auditLoading ? 'animate-spin' : ''}" />
          </button>
        </div>
        <div class="divide-y divide-border/50">
          {#if auditLoading}
            <div class="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
              <Loader2 class="w-4 h-4 animate-spin" /> Loading…
            </div>
          {:else if audit.length === 0}
            <div class="py-10 text-center text-sm text-muted-foreground">No audit entries yet.</div>
          {:else}
            {#each audit as entry}
              {@const isReverted = entry.metadata?.reverted === true}
              <div class="flex items-start gap-3 px-5 py-3 hover:bg-muted/20 transition">
                <span class="shrink-0 mt-0.5 inline-flex items-center text-[10px] font-semibold px-1.5 py-0.5 rounded-full border uppercase tracking-wide
                             {ACTION_STYLE[entry.action] ?? 'bg-muted text-muted-foreground border-border'}">
                  {entry.action}
                </span>
                <div class="flex-1 min-w-0">
                  <p class="text-sm">
                    <span class="font-medium">{entry.username ?? 'System'}</span>
                    <span class="text-muted-foreground"> {entry.action}d </span>
                    <span class="font-medium">{entry.target_name ?? entry.target_id ?? '—'}</span>
                    {#if isReverted}<span class="ml-1 text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-1.5 py-0.5 rounded-full">reverted</span>{/if}
                  </p>
                  <p class="text-[11px] text-muted-foreground/60 mt-0.5">{timeAgo(entry.created_at)}</p>
                </div>
                {#if entry.action === 'delete' && entry.target_type === 'event' && !isReverted && entry.metadata}
                  <button
                    on:click={() => revertAudit(entry.id)}
                    disabled={reverting === entry.id}
                    title="Restore this event"
                    class="shrink-0 h-7 px-2.5 rounded-lg border border-border text-xs text-muted-foreground
                           hover:bg-emerald-500/10 hover:text-emerald-400 hover:border-emerald-500/20
                           transition disabled:opacity-40 flex items-center gap-1"
                  >
                    {#if reverting === entry.id}
                      <Loader2 class="w-3 h-3 animate-spin" />
                    {:else}
                      <RotateCcw class="w-3 h-3" />
                    {/if}
                    Revert
                  </button>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

  </div>
</div>

<!-- User Dialog -->
{#if userDialogOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4"
    style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);" role="dialog">
    <div class="w-full max-w-md bg-card border border-border rounded-2xl shadow-2xl p-6 flex flex-col gap-5"
      style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;">
      <div class="flex items-center justify-between">
        <h2 class="font-semibold">{editingUser ? 'Edit User' : 'New User'}</h2>
        <button on:click={() => (userDialogOpen = false)} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground"><X class="w-4 h-4" /></button>
      </div>
      <div class="flex flex-col gap-3.5">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Username</label>
          <input class={INPUT} bind:value={userForm.username} placeholder="johndoe" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Email</label>
          <input class={INPUT} type="email" bind:value={userForm.email} placeholder="john@example.com" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Password {#if editingUser}<span class="normal-case font-normal text-muted-foreground/60">(leave blank to keep)</span>{/if}
          </label>
          <div class="relative">
            <input class="{INPUT} pr-10" type={showPw ? 'text' : 'password'} bind:value={userForm.password}
              placeholder={editingUser ? 'New password…' : 'Min. 8 characters'} />
            <button type="button" on:click={() => (showPw = !showPw)} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
              {#if showPw}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Role</label>
          <div class="grid grid-cols-2 gap-2">
            <button type="button" on:click={() => (userForm.perms = 0)}
              class="h-9 rounded-xl border text-sm font-medium transition {userForm.perms < 999 ? 'border-primary/60 bg-primary/10 text-primary' : 'border-border text-muted-foreground hover:bg-muted'}">User</button>
            <button type="button" on:click={() => (userForm.perms = 999)}
              class="h-9 rounded-xl border text-sm font-medium transition {userForm.perms >= 999 ? 'border-amber-500/40 bg-amber-500/10 text-amber-400' : 'border-border text-muted-foreground hover:bg-muted'}">Admin</button>
          </div>
        </div>
        {#if userError}
          <div class="flex items-center gap-2 px-3 py-2.5 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-xl">
            <AlertTriangle class="w-4 h-4 shrink-0 text-red-400" /> {userError}
          </div>
        {/if}
      </div>
      <div class="flex gap-2">
        <button on:click={() => (userDialogOpen = false)} disabled={userSaving} class="flex-1 h-9 rounded-xl border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={saveUser} disabled={userSaving} class="flex-1 h-9 rounded-xl bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition active:scale-[0.97]">
          {userSaving ? 'Saving…' : editingUser ? 'Update' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Confirm -->
{#if deleteTarget}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4"
    style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);" role="dialog">
    <div class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl p-6 flex flex-col gap-4"
      style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;">
      <div class="flex items-start gap-3">
        <div class="w-9 h-9 rounded-xl bg-red-500/10 flex items-center justify-center shrink-0"><AlertTriangle class="w-4 h-4 text-red-400" /></div>
        <div>
          <h2 class="font-semibold">Delete {deleteTarget.type}?</h2>
          <p class="text-sm text-muted-foreground mt-1"><span class="text-foreground font-medium">{deleteTarget.name}</span> will be permanently deleted.</p>
        </div>
      </div>
      <div class="flex gap-2">
        <button on:click={() => (deleteTarget = null)} disabled={deleteLoading} class="flex-1 h-9 rounded-xl border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={confirmDelete} disabled={deleteLoading} class="flex-1 h-9 rounded-xl bg-red-500/15 text-red-400 border border-red-500/25 text-sm font-medium hover:bg-red-500/25 disabled:opacity-50 transition active:scale-[0.97]">
          {deleteLoading ? 'Deleting…' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes modal-in {
    from { opacity: 0; transform: scale(0.94) translateY(8px); }
    to   { opacity: 1; transform: scale(1)    translateY(0);   }
  }
</style>
