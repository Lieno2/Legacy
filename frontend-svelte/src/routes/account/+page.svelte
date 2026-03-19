<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch, clearTokens } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User } from '$lib/types';
  import { ArrowLeft, LogOut, Shield, User as UserIcon, Mail, Lock } from 'lucide-svelte';

  let profile: User | null = null;
  let loading = true;

  let username        = '';
  let email           = '';
  let currentPassword = '';
  let newPassword     = '';
  let confirmPassword = '';
  let saving  = false;
  let error   = '';
  let success = '';

  onMount(async () => {
    try {
      profile  = await apiFetch<User>('/api/account');
      username = profile.username;
      email    = profile.email;
    } catch { goto('/login'); }
    finally { loading = false; }
  });

  async function handleSave(e: SubmitEvent) {
    e.preventDefault();
    error = ''; success = '';

    if (newPassword && newPassword !== confirmPassword) { error = 'Passwords do not match'; return; }
    if (newPassword && newPassword.length < 8) { error = 'Password must be at least 8 characters'; return; }

    saving = true;
    try {
      const updated = await apiFetch<User>('/api/account', {
        method: 'PUT',
        body: JSON.stringify({
          username, email,
          current_password: currentPassword || undefined,
          new_password:     newPassword     || undefined
        })
      });
      profile = updated;
      auth.update(s => ({ ...s, user: updated }));
      success = 'Account updated successfully';
      currentPassword = ''; newPassword = ''; confirmPassword = '';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to update';
    } finally { saving = false; }
  }

  async function handleLogout() {
    const rt = localStorage.getItem('refresh_token');
    if (rt) await apiFetch('/api/auth/logout', { method: 'POST', body: JSON.stringify({ refresh_token: rt }) }).catch(() => {});
    clearTokens();
    auth.set({ user: null, loading: false });
    goto('/login');
  }

  $: isAdmin = (profile?.perms ?? 0) >= 999;

  const INPUT = 'h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm outline-none focus:border-ring focus:ring-2 focus:ring-ring/30 transition';
</script>

{#if loading}
  <div class="min-h-screen flex items-center justify-center text-muted-foreground text-sm">Loading…</div>
{:else}
<div class="min-h-screen bg-background text-foreground">
  <!-- Header -->
  <header class="border-b border-border/50 px-6 py-4 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <a href="/calendar" class="p-1.5 rounded hover:bg-muted transition"><ArrowLeft class="w-4 h-4" /></a>
      <h1 class="text-lg font-semibold">Account Settings</h1>
    </div>
    <div class="flex items-center gap-2">
      {#if isAdmin}
        <a href="/admin" class="inline-flex items-center gap-1.5 h-8 px-3 rounded-md border border-border text-sm hover:bg-muted transition">
          <Shield class="w-3.5 h-3.5" /> Admin Panel
        </a>
      {/if}
      <button on:click={handleLogout} class="inline-flex items-center gap-1.5 h-8 px-3 rounded-md text-sm hover:bg-muted transition">
        <LogOut class="w-3.5 h-3.5" /> Sign out
      </button>
    </div>
  </header>

  <div class="max-w-xl mx-auto px-6 py-10 flex flex-col gap-6">
    <!-- Profile summary card -->
    <div class="rounded-xl bg-card ring-1 ring-foreground/10 p-6 flex items-center gap-4">
      <div class="w-14 h-14 rounded-full bg-primary/10 flex items-center justify-center text-2xl font-bold text-primary uppercase">
        {profile?.username?.[0] ?? '?'}
      </div>
      <div>
        <div class="font-semibold">{profile?.username}</div>
        <div class="text-sm text-muted-foreground">{profile?.email}</div>
        <div class="flex items-center gap-2 mt-1">
          {#if isAdmin}
            <span class="text-xs bg-amber-500/10 text-amber-400 px-2 py-0.5 rounded-full flex items-center gap-1">
              <Shield class="w-3 h-3" /> Admin
            </span>
          {:else}
            <span class="text-xs bg-muted text-muted-foreground px-2 py-0.5 rounded-full">User</span>
          {/if}
          <span class="text-xs text-muted-foreground">Joined {new Date(profile?.created_at ?? '').toLocaleDateString()}</span>
        </div>
      </div>
    </div>

    <!-- Edit form -->
    <div class="rounded-xl bg-card ring-1 ring-foreground/10 p-6 flex flex-col gap-5">
      <div>
        <h2 class="font-semibold">Edit Profile</h2>
        <p class="text-sm text-muted-foreground mt-0.5">Update your username, email, or password</p>
      </div>
      <form on:submit={handleSave} class="flex flex-col gap-4">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium flex items-center gap-1.5"><UserIcon class="w-3.5 h-3.5" /> Username</label>
          <input class={INPUT} bind:value={username} required />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium flex items-center gap-1.5"><Mail class="w-3.5 h-3.5" /> Email</label>
          <input class={INPUT} type="email" bind:value={email} required />
        </div>

        <div class="border-t border-border/50 pt-4 flex flex-col gap-3">
          <p class="text-sm font-medium flex items-center gap-1.5">
            <Lock class="w-3.5 h-3.5" /> Change Password
            <span class="text-muted-foreground font-normal">(optional)</span>
          </p>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">Current Password</label>
            <input class={INPUT} type="password" bind:value={currentPassword} placeholder="Required to change password" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">New Password</label>
            <input class={INPUT} type="password" bind:value={newPassword} placeholder="Min. 8 characters" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">Confirm New Password</label>
            <input class={INPUT} type="password" bind:value={confirmPassword} placeholder="Repeat new password" />
          </div>
        </div>

        {#if error}
          <div class="p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md">{error}</div>
        {/if}
        {#if success}
          <div class="p-3 text-sm text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded-md">{success}</div>
        {/if}

        <button type="submit" disabled={saving}
          class="h-9 w-full rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/80 disabled:opacity-50 transition">
          {saving ? 'Saving…' : 'Save Changes'}
        </button>
      </form>
    </div>
  </div>
</div>
{/if}
