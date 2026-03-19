<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User, Event } from '$lib/types';
  import { ArrowLeft, Shield, Users, CalendarDays, Plus, Pencil, Trash2, Search, X, Eye, EyeOff, Bell } from 'lucide-svelte';

  type Tab = 'users' | 'events';
  let tab: Tab = 'users';

  let users: User[]   = [];
  let events: Event[] = [];
  let usersLoading = true, eventsLoading = true;
  let userSearch = '', eventSearch = '';

  let userDialogOpen = false;
  let editingUser: User | null = null;
  let userForm = { username: '', email: '', password: '', perms: 0 };
  let showPw = false, userSaving = false, userError = '';

  let deleteUserOpen = false,  deletingUserId: string | null = null;
  let deleteEventOpen = false, deletingEventId: number | null = null;
  let deleteLoading = false;

  onMount(async () => {
    if (($auth.user?.perms ?? 0) < 999) { goto('/calendar'); return; }
    fetchUsers(); fetchEvents();
  });

  async function fetchUsers()  { usersLoading  = true; try { users  = await apiFetch<User[]>('/api/admin/users');  } finally { usersLoading  = false; } }
  async function fetchEvents() { eventsLoading = true; try { events = await apiFetch<Event[]>('/api/admin/events'); } finally { eventsLoading = false; } }

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

  async function deleteUser()  { if (!deletingUserId)  return; deleteLoading = true; try { await apiFetch(`/api/admin/users?id=${deletingUserId}`, { method: 'DELETE' });  await fetchUsers();  deleteUserOpen  = false; deletingUserId  = null; } finally { deleteLoading = false; } }
  async function deleteEvent() { if (!deletingEventId) return; deleteLoading = true; try { await apiFetch(`/api/admin/events?id=${deletingEventId}`, { method: 'DELETE' }); await fetchEvents(); deleteEventOpen = false; deletingEventId = null; } finally { deleteLoading = false; } }

  $: filteredUsers  = users.filter(u  => u.username.toLowerCase().includes(userSearch.toLowerCase()) || u.email.toLowerCase().includes(userSearch.toLowerCase()));
  $: filteredEvents = events.filter(e => e.title.toLowerCase().includes(eventSearch.toLowerCase()) || (e.creator_name ?? '').toLowerCase().includes(eventSearch.toLowerCase()));

  const INPUT = 'h-9 w-full rounded-lg border border-input bg-muted/30 px-3 text-sm outline-none focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/60';
  const TABS: { id: Tab; label: string }[] = [
    { id: 'users',  label: 'Users'  },
    { id: 'events', label: 'Events' },
  ];
</script>

<div class="min-h-screen bg-background text-foreground">
  <header class="border-b border-border px-5 py-3 flex items-center justify-between">
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
    <!-- Stat cards -->
    <div class="grid grid-cols-3 gap-3">
      <div class="bg-card border border-border rounded-2xl p-4 flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-blue-500/10 flex items-center justify-center shrink-0"><Users class="w-4 h-4 text-blue-400" /></div>
        <div><div class="text-xl font-bold">{users.length}</div><div class="text-xs text-muted-foreground">Users</div></div>
      </div>
      <div class="bg-card border border-border rounded-2xl p-4 flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-violet-500/10 flex items-center justify-center shrink-0"><CalendarDays class="w-4 h-4 text-violet-400" /></div>
        <div><div class="text-xl font-bold">{events.length}</div><div class="text-xs text-muted-foreground">Events</div></div>
      </div>
      <div class="bg-card border border-border rounded-2xl p-4 flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-emerald-500/10 flex items-center justify-center shrink-0"><Bell class="w-4 h-4 text-emerald-400" /></div>
        <div><div class="text-xl font-bold">—</div><div class="text-xs text-muted-foreground">Audit</div></div>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex gap-1 border-b border-border">
      {#each TABS as t}
        <button
          on:click={() => (tab = t.id)}
          class="px-4 py-2 text-sm font-medium border-b-2 transition -mb-px
                 {tab === t.id ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >{t.label}</button>
      {/each}
    </div>

    <!-- Users -->
    {#if tab === 'users'}
      <div class="bg-card border border-border rounded-2xl overflow-hidden">
        <div class="p-5 border-b border-border flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="font-semibold text-sm">Users</h2>
              <p class="text-xs text-muted-foreground mt-0.5">Manage user accounts and permissions</p>
            </div>
            <button on:click={openCreateUser}
              class="inline-flex items-center gap-1 h-8 px-3 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition">
              <Plus class="w-3.5 h-3.5" /> New User
            </button>
          </div>
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
            <input class="{INPUT} pl-9" placeholder="Search users..." bind:value={userSearch} />
          </div>
        </div>
        <div class="divide-y divide-border/60">
          {#if usersLoading}
            <div class="p-5 text-sm text-muted-foreground text-center">Loading…</div>
          {:else if filteredUsers.length === 0}
            <div class="p-5 text-sm text-muted-foreground text-center">No users found.</div>
          {:else}
            {#each filteredUsers as u}
              <div class="flex items-center justify-between px-5 py-3">
                <div class="flex items-center gap-3 min-w-0">
                  <div class="w-8 h-8 rounded-full bg-muted border border-border flex items-center justify-center text-xs font-bold uppercase shrink-0">{u.username[0]}</div>
                  <div class="min-w-0">
                    <div class="text-sm font-medium flex items-center gap-1.5">
                      {u.username}
                      {#if u.perms >= 999}
                        <span class="text-[10px] bg-amber-500/10 text-amber-400 border border-amber-500/20 px-1.5 py-0.5 rounded-full">Admin</span>
                      {/if}
                    </div>
                    <div class="text-xs text-muted-foreground truncate">{u.email}</div>
                  </div>
                </div>
                <div class="flex gap-0.5 shrink-0">
                  <button on:click={() => openEditUser(u)} class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground"><Pencil class="w-3.5 h-3.5" /></button>
                  <button on:click={() => { deletingUserId = u.id; deleteUserOpen = true; }} class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-red-500/10 transition text-muted-foreground hover:text-red-400"><Trash2 class="w-3.5 h-3.5" /></button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- Events -->
    {#if tab === 'events'}
      <div class="bg-card border border-border rounded-2xl overflow-hidden">
        <div class="p-5 border-b border-border flex flex-col gap-3">
          <div>
            <h2 class="font-semibold text-sm">All Events</h2>
            <p class="text-xs text-muted-foreground mt-0.5">View and delete any event across all users</p>
          </div>
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
            <input class="{INPUT} pl-9" placeholder="Search events..." bind:value={eventSearch} />
          </div>
        </div>
        <div class="divide-y divide-border/60">
          {#if eventsLoading}
            <div class="p-5 text-sm text-muted-foreground text-center">Loading…</div>
          {:else if filteredEvents.length === 0}
            <div class="p-5 text-sm text-muted-foreground text-center">No events found.</div>
          {:else}
            {#each filteredEvents as e}
              <div class="flex items-center justify-between px-5 py-3 gap-3">
                <div class="flex items-center gap-3 min-w-0">
                  <div class="w-1 h-8 rounded-full shrink-0" style="background:{e.color ?? '#6366f1'}"></div>
                  <div class="min-w-0">
                    <div class="text-sm font-medium truncate flex items-center gap-1.5">
                      {e.title}
                      {#if e.private}<span class="text-[10px] text-muted-foreground">🔒</span>{/if}
                    </div>
                    <div class="text-xs text-muted-foreground">by {e.creator_name ?? 'Unknown'} · {new Date(e.date).toLocaleDateString()}{e.location ? ` · ${e.location}` : ''}</div>
                  </div>
                </div>
                <button on:click={() => { deletingEventId = e.id; deleteEventOpen = true; }}
                  class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-red-500/10 transition text-muted-foreground hover:text-red-400 shrink-0">
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

<!-- User dialog -->
{#if userDialogOpen}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4" role="dialog">
    <div class="w-full max-w-md bg-card border border-border rounded-2xl shadow-2xl p-6 flex flex-col gap-5">
      <div class="flex items-center justify-between">
        <h2 class="font-semibold">{editingUser ? 'Edit User' : 'New User'}</h2>
        <button on:click={() => (userDialogOpen = false)} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition"><X class="w-4 h-4" /></button>
      </div>
      <div class="flex flex-col gap-3.5">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Username</label>
          <input class={INPUT} bind:value={userForm.username} />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Email</label>
          <input class={INPUT} type="email" bind:value={userForm.email} />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Password {#if editingUser}<span class="normal-case text-muted-foreground font-normal">(leave blank to keep)</span>{/if}</label>
          <div class="relative">
            <input class="{INPUT} pr-10" type={showPw ? 'text' : 'password'} bind:value={userForm.password} placeholder={editingUser ? 'New password…' : 'Min. 8 characters'} />
            <button type="button" on:click={() => (showPw = !showPw)} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
              {#if showPw}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Role</label>
          <div class="grid grid-cols-2 gap-2">
            <button type="button" on:click={() => (userForm.perms = 0)}
              class="h-9 rounded-lg border text-sm font-medium transition
                     {userForm.perms < 999 ? 'border-primary bg-primary/10 text-primary' : 'border-border text-muted-foreground hover:bg-muted'}">User</button>
            <button type="button" on:click={() => (userForm.perms = 999)}
              class="h-9 rounded-lg border text-sm font-medium transition
                     {userForm.perms >= 999 ? 'border-amber-500/50 bg-amber-500/10 text-amber-400' : 'border-border text-muted-foreground hover:bg-muted'}">Admin</button>
          </div>
        </div>
        {#if userError}
          <div class="px-3 py-2.5 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg">{userError}</div>
        {/if}
      </div>
      <div class="flex gap-2">
        <button on:click={() => (userDialogOpen = false)} disabled={userSaving} class="flex-1 h-9 rounded-lg border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={saveUser} disabled={userSaving} class="flex-1 h-9 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition">
          {userSaving ? 'Saving…' : editingUser ? 'Update' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Confirm delete dialogs -->
{#if deleteUserOpen || deleteEventOpen}
  {@const isUser = deleteUserOpen}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4" role="dialog">
    <div class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl p-6 flex flex-col gap-4">
      <h2 class="font-semibold">Delete {isUser ? 'User' : 'Event'}?</h2>
      <p class="text-sm text-muted-foreground">This action is permanent and cannot be undone.</p>
      <div class="flex gap-2">
        <button on:click={() => { deleteUserOpen = false; deleteEventOpen = false; }} disabled={deleteLoading}
          class="flex-1 h-9 rounded-lg border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button on:click={isUser ? deleteUser : deleteEvent} disabled={deleteLoading}
          class="flex-1 h-9 rounded-lg bg-red-500/10 text-red-400 border border-red-500/20 text-sm font-medium hover:bg-red-500/20 disabled:opacity-50 transition">
          {deleteLoading ? 'Deleting…' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}
