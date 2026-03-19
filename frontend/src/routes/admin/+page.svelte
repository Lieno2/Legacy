<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User, Event } from '$lib/types';
  import {
    ArrowLeft, Shield, Users, CalendarDays, Plus, Pencil,
    Trash2, Search, X, Eye, EyeOff, Bell, BarChart2, Hash
  } from 'lucide-svelte';

  type Tab = 'users' | 'events' | 'discord' | 'stats';
  let tab: Tab = 'users';

  let users:  User[]  = [];
  let events: Event[] = [];
  let usersLoading = true, eventsLoading = true;
  let userSearch = '', eventSearch = '';

  // User dialog
  let userDialogOpen = false;
  let editingUser: User | null = null;
  let userForm = { username: '', email: '', password: '', perms: 0 };
  let showPw = false, userSaving = false, userError = '';

  // Delete
  let deleteTarget: { type: 'user' | 'event'; id: string | number; name: string } | null = null;
  let deleteLoading = false;

  onMount(async () => {
    if (($auth.user?.perms ?? 0) < 999) { goto('/calendar'); return; }
    fetchUsers(); fetchEvents();
  });

  async function fetchUsers()  { usersLoading  = true; try { users  = await apiFetch<User[]>('/api/admin/users');  } finally { usersLoading  = false; } }
  async function fetchEvents() { eventsLoading = true; try { events = await apiFetch<Event[]>('/api/admin/events'); } finally { eventsLoading = false; } }

  function openCreateUser() {
    editingUser = null;
    userForm = { username: '', email: '', password: '', perms: 0 };
    userError = ''; showPw = false; userDialogOpen = true;
  }
  function openEditUser(u: User) {
    editingUser = u;
    userForm = { username: u.username, email: u.email, password: '', perms: u.perms };
    userError = ''; showPw = false; userDialogOpen = true;
  }

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
        await apiFetch(`/api/admin/users?id=${deleteTarget.id}`,  { method: 'DELETE' });
        await fetchUsers();
      } else {
        await apiFetch(`/api/admin/events?id=${deleteTarget.id}`, { method: 'DELETE' });
        await fetchEvents();
      }
      deleteTarget = null;
    } finally { deleteLoading = false; }
  }

  $: filteredUsers  = users.filter(u  => u.username.toLowerCase().includes(userSearch.toLowerCase())  || u.email.toLowerCase().includes(userSearch.toLowerCase()));
  $: filteredEvents = events.filter(e => e.title.toLowerCase().includes(eventSearch.toLowerCase()) || (e.creator_name ?? '').toLowerCase().includes(eventSearch.toLowerCase()));

  // Stats derived
  $: adminCount     = users.filter(u => u.perms >= 999).length;
  $: upcomingCount  = events.filter(e => new Date(e.date) > new Date()).length;
  $: privateCount   = events.filter(e => e.private).length;

  const INPUT = 'h-9 w-full rounded-lg border border-input bg-muted/20 px-3 text-sm outline-none focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/50';

  const TABS: { id: Tab; label: string; icon: any }[] = [
    { id: 'users',   label: 'Users',   icon: Users       },
    { id: 'events',  label: 'Events',  icon: CalendarDays},
    { id: 'stats',   label: 'Stats',   icon: BarChart2   },
    { id: 'discord', label: 'Discord', icon: Hash        },
  ];
</script>

<div class="min-h-screen bg-background text-foreground">
  <!-- Header -->
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

    <!-- Stat grid -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      {#each [
        { label: 'Total Users',   value: users.length,   color: '#3b82f6', icon: Users       },
        { label: 'Total Events',  value: events.length,  color: '#a855f7', icon: CalendarDays},
        { label: 'Upcoming',      value: upcomingCount,  color: '#22c55e', icon: Bell        },
        { label: 'Private',       value: privateCount,   color: '#f97316', icon: Shield      },
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

    <!-- Tabs -->
    <div class="flex gap-0.5 bg-muted/40 border border-border p-1 rounded-xl">
      {#each TABS as t}
        <button
          on:click={() => (tab = t.id)}
          class="flex-1 flex items-center justify-center gap-1.5 h-8 rounded-lg text-xs font-medium transition-all
                 {tab === t.id ? 'bg-card shadow-sm text-foreground border border-border/60' : 'text-muted-foreground hover:text-foreground'}"
        >
          <svelte:component this={t.icon} class="w-3.5 h-3.5" />
          {t.label}
        </button>
      {/each}
    </div>

    <!-- Users Tab -->
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
            {#if userSearch}
              <button on:click={() => (userSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
                <X class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if usersLoading}
            <div class="flex items-center justify-center gap-2 py-8 text-muted-foreground text-sm">
              <div class="w-4 h-4 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin"></div>
              Loading…
            </div>
          {:else if filteredUsers.length === 0}
            <div class="py-8 text-center text-sm text-muted-foreground">No users found</div>
          {:else}
            {#each filteredUsers as u}
              <div class="flex items-center gap-3 px-5 py-3 hover:bg-muted/30 transition group">
                <div class="w-8 h-8 rounded-full bg-muted border border-border flex items-center justify-center text-xs font-bold uppercase shrink-0"
                  style="background: hsl({((u.username.charCodeAt(0) * 47) % 360)}, 40%, 25%); color: hsl({((u.username.charCodeAt(0) * 47) % 360)}, 70%, 70%);">
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
                  <button on:click={() => openEditUser(u)}
                    class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition">
                    <Pencil class="w-3.5 h-3.5" />
                  </button>
                  <button on:click={() => (deleteTarget = { type: 'user', id: u.id, name: u.username })}
                    class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition">
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- Events Tab -->
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
            {#if eventSearch}
              <button on:click={() => (eventSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
                <X class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if eventsLoading}
            <div class="flex items-center justify-center gap-2 py-8 text-muted-foreground text-sm">
              <div class="w-4 h-4 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin"></div>
              Loading…
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
                    {#if e.private}<span class="text-[10px] opacity-70">🔒</span>{/if}
                    {#if isPast}<span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full">Past</span>{/if}
                  </div>
                  <div class="text-xs text-muted-foreground">
                    {new Date(e.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
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

    <!-- Stats Tab -->
    {#if tab === 'stats'}
      <div class="flex flex-col gap-4">
        <div class="bg-card border border-border rounded-2xl p-5">
          <h2 class="font-semibold text-sm mb-4">User breakdown</h2>
          <div class="flex flex-col gap-2">
            {#each [
              { label: 'Regular users', value: users.length - adminCount, total: users.length, color: '#3b82f6' },
              { label: 'Admins',        value: adminCount,                total: users.length, color: '#eab308' },
            ] as row}
              <div class="flex flex-col gap-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="text-muted-foreground">{row.label}</span>
                  <span class="font-medium tabular-nums">{row.value}</span>
                </div>
                <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                  <div class="h-full rounded-full transition-all" style="width:{row.total ? (row.value/row.total*100).toFixed(1) : 0}%; background:{row.color};"></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
        <div class="bg-card border border-border rounded-2xl p-5">
          <h2 class="font-semibold text-sm mb-4">Event breakdown</h2>
          <div class="flex flex-col gap-2">
            {#each [
              { label: 'Upcoming events', value: upcomingCount,                         total: events.length, color: '#22c55e' },
              { label: 'Past events',     value: events.length - upcomingCount,          total: events.length, color: '#6366f1' },
              { label: 'Private events',  value: privateCount,                          total: events.length, color: '#f97316' },
            ] as row}
              <div class="flex flex-col gap-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="text-muted-foreground">{row.label}</span>
                  <span class="font-medium tabular-nums">{row.value}</span>
                </div>
                <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                  <div class="h-full rounded-full transition-all" style="width:{row.total ? (row.value/row.total*100).toFixed(1) : 0}%; background:{row.color};"></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Discord Tab -->
    {#if tab === 'discord'}
      <div class="bg-card border border-border rounded-2xl p-6 flex flex-col items-center gap-4 text-center">
        <div class="w-14 h-14 rounded-2xl bg-[#5865f2]/10 border border-[#5865f2]/20 flex items-center justify-center">
          <Hash class="w-6 h-6 text-[#5865f2]" />
        </div>
        <div>
          <h2 class="font-semibold">Discord Integration</h2>
          <p class="text-sm text-muted-foreground mt-1 max-w-xs">Discord bot settings and notifications will appear here once configured.</p>
        </div>
        <div class="flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 border border-border px-4 py-2.5 rounded-xl">
          <span class="w-2 h-2 rounded-full bg-muted-foreground/50"></span>
          Not configured
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
        <button on:click={() => (userDialogOpen = false)} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground">
          <X class="w-4 h-4" />
        </button>
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
            <input class="{INPUT} pr-10" type={showPw ? 'text' : 'password'}
              bind:value={userForm.password} placeholder={editingUser ? 'New password…' : 'Min. 8 characters'} />
            <button type="button" on:click={() => (showPw = !showPw)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
              {#if showPw}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Role</label>
          <div class="grid grid-cols-2 gap-2">
            <button type="button" on:click={() => (userForm.perms = 0)}
              class="h-9 rounded-xl border text-sm font-medium transition
                     {userForm.perms < 999 ? 'border-primary/60 bg-primary/10 text-primary' : 'border-border text-muted-foreground hover:bg-muted'}">User</button>
            <button type="button" on:click={() => (userForm.perms = 999)}
              class="h-9 rounded-xl border text-sm font-medium transition
                     {userForm.perms >= 999 ? 'border-amber-500/40 bg-amber-500/10 text-amber-400' : 'border-border text-muted-foreground hover:bg-muted'}">Admin</button>
          </div>
        </div>
        {#if userError}
          <div class="flex items-center gap-2 px-3 py-2.5 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-xl">
            <span>⚠</span> {userError}
          </div>
        {/if}
      </div>
      <div class="flex gap-2">
        <button on:click={() => (userDialogOpen = false)} disabled={userSaving}
          class="flex-1 h-9 rounded-xl border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={saveUser} disabled={userSaving}
          class="flex-1 h-9 rounded-xl bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition active:scale-[0.97]">
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
        <div class="w-9 h-9 rounded-xl bg-red-500/10 flex items-center justify-center shrink-0">
          <Trash2 class="w-4 h-4 text-red-400" />
        </div>
        <div>
          <h2 class="font-semibold">Delete {deleteTarget.type === 'user' ? 'user' : 'event'}?</h2>
          <p class="text-sm text-muted-foreground mt-1">
            <span class="text-foreground font-medium">{deleteTarget.name}</span> will be permanently deleted. This cannot be undone.
          </p>
        </div>
      </div>
      <div class="flex gap-2">
        <button on:click={() => (deleteTarget = null)} disabled={deleteLoading}
          class="flex-1 h-9 rounded-xl border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={confirmDelete} disabled={deleteLoading}
          class="flex-1 h-9 rounded-xl bg-red-500/15 text-red-400 border border-red-500/25 text-sm font-medium hover:bg-red-500/25 disabled:opacity-50 transition active:scale-[0.97]">
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
