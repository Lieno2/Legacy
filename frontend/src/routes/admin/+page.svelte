<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User, Event } from '$lib/types';
  import { ArrowLeft, Shield, Users, CalendarDays, Plus, Pencil, Trash2, Search, X, Eye, EyeOff } from 'lucide-svelte';

  type Tab = 'users' | 'events';
  let tab: Tab = 'users';

  let users:  User[]  = [];
  let events: Event[] = [];
  let usersLoading  = true;
  let eventsLoading = true;
  let userSearch  = '';
  let eventSearch = '';
  let globalError = '';

  // User dialog
  let userDialogOpen = false;
  let editingUser: User | null = null;
  let userForm = { username: '', email: '', password: '', perms: 0 };
  let showPassword = false;
  let userSaving = false;
  let userError  = '';

  // Delete
  let deleteUserOpen  = false;
  let deletingUserId: string | null = null;
  let deleteEventOpen = false;
  let deletingEventId: number | null = null;
  let deleteLoading = false;

  onMount(async () => {
    if (($auth.user?.perms ?? 0) < 999) { goto('/calendar'); return; }
    fetchUsers();
    fetchEvents();
  });

  async function fetchUsers() {
    usersLoading = true;
    try { users = await apiFetch<User[]>('/api/admin/users'); }
    catch { globalError = 'Failed to load users'; }
    finally { usersLoading = false; }
  }

  async function fetchEvents() {
    eventsLoading = true;
    try { events = await apiFetch<Event[]>('/api/admin/events'); }
    catch { globalError = 'Failed to load events'; }
    finally { eventsLoading = false; }
  }

  function openCreateUser() {
    editingUser = null;
    userForm = { username: '', email: '', password: '', perms: 0 };
    userError = ''; showPassword = false; userDialogOpen = true;
  }

  function openEditUser(u: User) {
    editingUser = u;
    userForm = { username: u.username, email: u.email, password: '', perms: u.perms };
    userError = ''; showPassword = false; userDialogOpen = true;
  }

  async function saveUser() {
    userError = '';
    if (!userForm.username.trim() || !userForm.email.trim()) { userError = 'Username and email are required'; return; }
    if (!editingUser && !userForm.password) { userError = 'Password is required for new users'; return; }
    if (userForm.password && userForm.password.length < 8) { userError = 'Password must be at least 8 characters'; return; }

    userSaving = true;
    try {
      const payload = editingUser
        ? { id: editingUser.id, username: userForm.username, email: userForm.email, perms: userForm.perms, new_password: userForm.password || undefined }
        : { username: userForm.username, email: userForm.email, password: userForm.password, perms: userForm.perms };

      await apiFetch('/api/admin/users', {
        method: editingUser ? 'PUT' : 'POST',
        body: JSON.stringify(payload)
      });
      await fetchUsers();
      userDialogOpen = false;
    } catch (err: unknown) {
      userError = err instanceof Error ? err.message : 'Failed to save user';
    } finally { userSaving = false; }
  }

  async function deleteUser() {
    if (!deletingUserId) return;
    deleteLoading = true;
    try {
      await apiFetch(`/api/admin/users?id=${deletingUserId}`, { method: 'DELETE' });
      await fetchUsers();
      deleteUserOpen = false; deletingUserId = null;
    } finally { deleteLoading = false; }
  }

  async function deleteEvent() {
    if (!deletingEventId) return;
    deleteLoading = true;
    try {
      await apiFetch(`/api/admin/events?id=${deletingEventId}`, { method: 'DELETE' });
      await fetchEvents();
      deleteEventOpen = false; deletingEventId = null;
    } finally { deleteLoading = false; }
  }

  $: filteredUsers  = users.filter(u =>
    u.username.toLowerCase().includes(userSearch.toLowerCase()) ||
    u.email.toLowerCase().includes(userSearch.toLowerCase())
  );
  $: filteredEvents = events.filter(e =>
    e.title.toLowerCase().includes(eventSearch.toLowerCase()) ||
    (e.creator_name ?? '').toLowerCase().includes(eventSearch.toLowerCase())
  );

  const INPUT = 'h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm outline-none focus:border-ring focus:ring-2 focus:ring-ring/30 transition';
</script>

<div class="min-h-screen bg-background text-foreground">
  <!-- Header -->
  <header class="border-b border-border/50 px-6 py-4 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <a href="/calendar" class="p-1.5 rounded hover:bg-muted transition"><ArrowLeft class="w-4 h-4" /></a>
      <Shield class="w-5 h-5 text-amber-400" />
      <h1 class="text-lg font-semibold">Admin Panel</h1>
    </div>
    <a href="/account" class="h-8 px-3 rounded-md text-sm hover:bg-muted transition inline-flex items-center">Account</a>
  </header>

  {#if globalError}
    <div class="mx-6 mt-4 p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md">{globalError}</div>
  {/if}

  <!-- Stats -->
  <div class="px-6 pt-6 grid grid-cols-2 gap-4 max-w-4xl mx-auto">
    <div class="rounded-xl bg-card ring-1 ring-foreground/10 p-5 flex items-center gap-3">
      <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center"><Users class="w-5 h-5 text-blue-400" /></div>
      <div><div class="text-2xl font-bold">{users.length}</div><div class="text-sm text-muted-foreground">Total Users</div></div>
    </div>
    <div class="rounded-xl bg-card ring-1 ring-foreground/10 p-5 flex items-center gap-3">
      <div class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center"><CalendarDays class="w-5 h-5 text-violet-400" /></div>
      <div><div class="text-2xl font-bold">{events.length}</div><div class="text-sm text-muted-foreground">Total Events</div></div>
    </div>
  </div>

  <!-- Tabs -->
  <div class="px-6 pt-6 max-w-4xl mx-auto">
    <div class="flex gap-1 bg-muted/50 p-1 rounded-lg w-fit mb-6">
      {#each (['users', 'events'] as Tab[]) as t}
        <button
          on:click={() => (tab = t)}
          class="px-4 py-1.5 rounded-md text-sm font-medium transition
                 {tab === t ? 'bg-card shadow text-foreground' : 'text-muted-foreground hover:text-foreground'}"
        >
          {t === 'users' ? 'Users' : 'Events'}
        </button>
      {/each}
    </div>

    <!-- Users Tab -->
    {#if tab === 'users'}
      <div class="rounded-xl bg-card ring-1 ring-foreground/10 overflow-hidden">
        <div class="p-6 border-b border-border/50 flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="font-semibold">Users</h2>
              <p class="text-sm text-muted-foreground">Manage accounts and permissions</p>
            </div>
            <button on:click={openCreateUser}
              class="inline-flex items-center gap-1 h-8 px-3 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/80 transition">
              <Plus class="w-3.5 h-3.5" /> New User
            </button>
          </div>
          <div class="relative">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <input class="{INPUT} pl-8" placeholder="Search users…" bind:value={userSearch} />
            {#if userSearch}
              <button on:click={() => (userSearch = '')} class="absolute right-2.5 top-1/2 -translate-y-1/2">
                <X class="w-3 h-3 text-muted-foreground" />
              </button>
            {/if}
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if usersLoading}
            <p class="p-4 text-sm text-muted-foreground">Loading…</p>
          {:else if filteredUsers.length === 0}
            <p class="p-4 text-sm text-muted-foreground">No users found.</p>
          {:else}
            {#each filteredUsers as u}
              <div class="flex items-center justify-between px-6 py-3">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center text-xs font-bold uppercase">{u.username[0]}</div>
                  <div>
                    <div class="text-sm font-medium flex items-center gap-1.5">
                      {u.username}
                      {#if u.perms >= 999}
                        <span class="text-[10px] bg-amber-500/10 text-amber-400 px-1.5 py-0.5 rounded-full">Admin</span>
                      {/if}
                    </div>
                    <div class="text-xs text-muted-foreground">{u.email}</div>
                  </div>
                </div>
                <div class="flex gap-1">
                  <button on:click={() => openEditUser(u)} class="p-1.5 rounded hover:bg-muted transition"><Pencil class="w-3.5 h-3.5" /></button>
                  <button on:click={() => { deletingUserId = u.id; deleteUserOpen = true; }} class="p-1.5 rounded hover:bg-muted text-destructive transition"><Trash2 class="w-3.5 h-3.5" /></button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- Events Tab -->
    {#if tab === 'events'}
      <div class="rounded-xl bg-card ring-1 ring-foreground/10 overflow-hidden">
        <div class="p-6 border-b border-border/50 flex flex-col gap-3">
          <div>
            <h2 class="font-semibold">All Events</h2>
            <p class="text-sm text-muted-foreground">View and delete any event across all users</p>
          </div>
          <div class="relative">
            <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <input class="{INPUT} pl-8" placeholder="Search events…" bind:value={eventSearch} />
          </div>
        </div>
        <div class="divide-y divide-border/50">
          {#if eventsLoading}
            <p class="p-4 text-sm text-muted-foreground">Loading…</p>
          {:else if filteredEvents.length === 0}
            <p class="p-4 text-sm text-muted-foreground">No events found.</p>
          {:else}
            {#each filteredEvents as e}
              <div class="flex items-center justify-between px-6 py-3 gap-3">
                <div class="flex items-center gap-3 min-w-0">
                  <div class="w-2 h-8 rounded-full flex-shrink-0" style="background:{e.color ?? '#6366f1'}"></div>
                  <div class="min-w-0">
                    <div class="text-sm font-medium truncate">{e.title}</div>
                    <div class="text-xs text-muted-foreground">by {e.creator_name ?? 'Unknown'} · {new Date(e.date).toLocaleDateString()}</div>
                  </div>
                </div>
                <button on:click={() => { deletingEventId = e.id; deleteEventOpen = true; }} class="p-1.5 rounded hover:bg-muted text-destructive transition flex-shrink-0">
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- User Form Dialog -->
{#if userDialogOpen}
  <div class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" role="dialog">
    <div class="w-full max-w-md bg-card rounded-xl ring-1 ring-foreground/10 shadow-xl p-6 flex flex-col gap-5">
      <div class="flex items-center justify-between">
        <h2 class="font-semibold">{editingUser ? 'Edit User' : 'New User'}</h2>
        <button on:click={() => (userDialogOpen = false)} class="p-1 rounded hover:bg-muted transition"><X class="w-4 h-4" /></button>
      </div>
      <div class="flex flex-col gap-4">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">Username</label>
          <input class={INPUT} bind:value={userForm.username} />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">Email</label>
          <input class={INPUT} type="email" bind:value={userForm.email} />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">Password {#if editingUser}<span class="text-muted-foreground font-normal">(leave blank to keep)</span>{/if}</label>
          <div class="relative">
            <input class="{INPUT} pr-10" type={showPassword ? 'text' : 'password'} bind:value={userForm.password} placeholder={editingUser ? 'New password…' : 'Min. 8 characters'} />
            <button type="button" on:click={() => (showPassword = !showPassword)} class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground">
              {#if showPassword}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">Role</label>
          <div class="flex gap-2">
            <button type="button" on:click={() => (userForm.perms = 0)}
              class="flex-1 py-2 rounded-md border text-sm font-medium transition
                     {userForm.perms < 999 ? 'border-primary bg-primary/10 text-primary' : 'border-border text-muted-foreground'}">
              User
            </button>
            <button type="button" on:click={() => (userForm.perms = 999)}
              class="flex-1 py-2 rounded-md border text-sm font-medium transition
                     {userForm.perms >= 999 ? 'border-amber-500 bg-amber-500/10 text-amber-400' : 'border-border text-muted-foreground'}">
              Admin
            </button>
          </div>
        </div>
        {#if userError}
          <div class="p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md">{userError}</div>
        {/if}
      </div>
      <div class="flex gap-2 justify-end">
        <button on:click={() => (userDialogOpen = false)} disabled={userSaving}
          class="h-9 px-4 rounded-md border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={saveUser} disabled={userSaving}
          class="h-9 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/80 disabled:opacity-50 transition">
          {userSaving ? 'Saving…' : editingUser ? 'Update' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete User Dialog -->
{#if deleteUserOpen}
  <div class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" role="dialog">
    <div class="w-full max-w-sm bg-card rounded-xl ring-1 ring-foreground/10 shadow-xl p-6 flex flex-col gap-4">
      <h2 class="font-semibold text-destructive">Delete User</h2>
      <p class="text-sm text-muted-foreground">This will permanently delete the user and all their data. This cannot be undone.</p>
      <div class="flex gap-2 justify-end">
        <button on:click={() => (deleteUserOpen = false)} disabled={deleteLoading}
          class="h-9 px-4 rounded-md border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={deleteUser} disabled={deleteLoading}
          class="h-9 px-4 rounded-md bg-destructive/10 text-destructive text-sm font-medium hover:bg-destructive/20 disabled:opacity-50 transition">
          {deleteLoading ? 'Deleting…' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Event Dialog -->
{#if deleteEventOpen}
  <div class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" role="dialog">
    <div class="w-full max-w-sm bg-card rounded-xl ring-1 ring-foreground/10 shadow-xl p-6 flex flex-col gap-4">
      <h2 class="font-semibold text-destructive">Delete Event</h2>
      <p class="text-sm text-muted-foreground">This will permanently delete this event and all its RSVPs. This cannot be undone.</p>
      <div class="flex gap-2 justify-end">
        <button on:click={() => (deleteEventOpen = false)} disabled={deleteLoading}
          class="h-9 px-4 rounded-md border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={deleteEvent} disabled={deleteLoading}
          class="h-9 px-4 rounded-md bg-destructive/10 text-destructive text-sm font-medium hover:bg-destructive/20 disabled:opacity-50 transition">
          {deleteLoading ? 'Deleting…' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}
