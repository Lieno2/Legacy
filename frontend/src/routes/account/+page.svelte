<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch, clearTokens } from '$lib/api';
  import { auth } from '$lib/stores';
  import { goto } from '$app/navigation';
  import type { User } from '$lib/types';
  import { ArrowLeft, LogOut, Shield } from 'lucide-svelte';

  let profile: User | null = null;
  let loading = true;
  let username = '', email = '', currentPw = '', newPw = '', confirmPw = '';
  let saving = false, error = '', success = '';

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
    if (newPw && newPw !== confirmPw) { error = 'Passwords do not match'; return; }
    if (newPw && newPw.length < 8) { error = 'Password must be at least 8 characters'; return; }
    saving = true;
    try {
      const updated = await apiFetch<User>('/api/account', {
        method: 'PUT',
        body: JSON.stringify({ username, email, current_password: currentPw || undefined, new_password: newPw || undefined })
      });
      profile = updated;
      auth.update(s => ({ ...s, user: updated }));
      success = 'Account updated successfully';
      currentPw = ''; newPw = ''; confirmPw = '';
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
  const INPUT = 'h-9 w-full rounded-lg border border-input bg-muted/30 px-3 text-sm outline-none focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/60';
</script>

{#if loading}
  <div class="min-h-screen flex items-center justify-center">
    <div class="w-5 h-5 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin"></div>
  </div>
{:else}
<div class="min-h-screen bg-background text-foreground">
  <header class="border-b border-border px-5 py-3 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <a href="/calendar" class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground">
        <ArrowLeft class="w-4 h-4" />
      </a>
      <h1 class="text-sm font-semibold">Account Settings</h1>
    </div>
    <div class="flex items-center gap-1.5">
      {#if isAdmin}
        <a href="/admin"
          class="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg border border-border text-xs font-medium hover:bg-muted transition">
          <Shield class="w-3.5 h-3.5 text-amber-400" /> Admin Panel
        </a>
      {/if}
      <button on:click={handleLogout}
        class="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg text-xs font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition">
        <LogOut class="w-3.5 h-3.5" /> Sign out
      </button>
    </div>
  </header>

  <div class="max-w-lg mx-auto px-5 py-8 flex flex-col gap-5">
    <!-- Profile card -->
    <div class="bg-card border border-border rounded-2xl p-5 flex items-center gap-4">
      <div class="w-14 h-14 rounded-full bg-muted border border-border flex items-center justify-center text-2xl font-bold uppercase shrink-0">
        {profile?.username?.[0] ?? '?'}
      </div>
      <div class="min-w-0">
        <div class="font-semibold">{profile?.username}</div>
        <div class="text-sm text-muted-foreground truncate">{profile?.email}</div>
        <div class="flex items-center gap-2 mt-1.5 flex-wrap">
          {#if isAdmin}
            <span class="inline-flex items-center gap-1 text-[11px] bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded-full font-medium">
              <Shield class="w-2.5 h-2.5" /> Admin
            </span>
          {:else}
            <span class="text-[11px] bg-muted text-muted-foreground border border-border px-2 py-0.5 rounded-full">User</span>
          {/if}
          {#if profile?.created_at}
            <span class="text-[11px] text-muted-foreground">Joined {new Date(profile.created_at).toLocaleDateString()}</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Edit form -->
    <div class="bg-card border border-border rounded-2xl p-5 flex flex-col gap-5">
      <div>
        <h2 class="font-semibold text-sm">Edit Profile</h2>
        <p class="text-xs text-muted-foreground mt-0.5">Update your username, email, or password</p>
      </div>

      <form on:submit={handleSave} class="flex flex-col gap-3.5">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Username</label>
          <input class={INPUT} bind:value={username} required />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Email</label>
          <input class={INPUT} type="email" bind:value={email} required />
        </div>

        <div class="border-t border-border pt-4 flex flex-col gap-3">
          <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Change Password <span class="normal-case">(optional)</span></p>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs text-muted-foreground">Current Password</label>
            <input class={INPUT} type="password" bind:value={currentPw} placeholder="Required to change password" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs text-muted-foreground">New Password</label>
            <input class={INPUT} type="password" bind:value={newPw} placeholder="Min. 8 characters" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs text-muted-foreground">Confirm New Password</label>
            <input class={INPUT} type="password" bind:value={confirmPw} placeholder="Repeat new password" />
          </div>
        </div>

        {#if error}
          <div class="px-3 py-2.5 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg">{error}</div>
        {/if}
        {#if success}
          <div class="px-3 py-2.5 text-sm text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded-lg">{success}</div>
        {/if}

        <button type="submit" disabled={saving}
          class="h-9 w-full rounded-lg bg-primary text-primary-foreground text-sm font-medium
                 hover:bg-primary/90 disabled:opacity-50 transition active:scale-[0.98]">
          {saving ? 'Saving…' : 'Save Changes'}
        </button>
      </form>
    </div>
  </div>
</div>
{/if}
