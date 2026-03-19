<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores';
  import { apiFetch, setTokens } from '$lib/api';
  import type { User } from '$lib/types';
  import { Eye, EyeOff } from 'lucide-svelte';

  let email    = '';
  let password = '';
  let error    = '';
  let loading  = false;
  let showPw   = false;

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = ''; loading = true;
    try {
      const data = await apiFetch<{ access_token: string; refresh_token: string; user: User }>(
        '/api/auth/login',
        { method: 'POST', body: JSON.stringify({ email, password }) }
      );
      setTokens(data.access_token, data.refresh_token);
      auth.set({ user: data.user, loading: false });
      goto('/calendar');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Login failed';
    } finally { loading = false; }
  }
</script>

<div class="min-h-screen bg-background flex items-center justify-center p-4">
  <div class="w-full max-w-sm">
    <!-- Logo mark -->
    <div class="flex items-center justify-center mb-8">
      <div class="w-10 h-10 rounded-xl bg-primary/10 border border-border flex items-center justify-center text-lg font-bold">L</div>
    </div>

    <div class="bg-card border border-border rounded-2xl p-8 shadow-xl shadow-black/20">
      <div class="mb-6">
        <h1 class="text-xl font-semibold tracking-tight">Sign in</h1>
        <p class="text-sm text-muted-foreground mt-1">Enter your credentials to access your account</p>
      </div>

      <form on:submit={handleSubmit} class="flex flex-col gap-4">
        <div class="flex flex-col gap-1.5">
          <label for="email" class="text-sm font-medium">Email</label>
          <input
            id="email" type="email" bind:value={email} required disabled={loading}
            placeholder="you@example.com"
            class="h-10 w-full rounded-lg border border-input bg-muted/30 px-3 text-sm outline-none
                   focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20
                   disabled:opacity-50 transition placeholder:text-muted-foreground/60"
          />
        </div>

        <div class="flex flex-col gap-1.5">
          <label for="password" class="text-sm font-medium">Password</label>
          <div class="relative">
            <input
              id="password" type={showPw ? 'text' : 'password'} bind:value={password} required disabled={loading}
              placeholder="Enter your password"
              class="h-10 w-full rounded-lg border border-input bg-muted/30 px-3 pr-10 text-sm outline-none
                     focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20
                     disabled:opacity-50 transition placeholder:text-muted-foreground/60"
            />
            <button type="button" on:click={() => (showPw = !showPw)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition">
              {#if showPw}<EyeOff class="w-4 h-4" />{:else}<Eye class="w-4 h-4" />{/if}
            </button>
          </div>
        </div>

        {#if error}
          <div class="flex items-center gap-2 px-3 py-2.5 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg">
            <span class="text-base">⚠</span> {error}
          </div>
        {/if}

        <button type="submit" disabled={loading}
          class="mt-1 h-10 w-full rounded-lg bg-primary text-primary-foreground text-sm font-medium
                 hover:bg-primary/90 disabled:opacity-50 transition-all active:scale-[0.98]">
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
    </div>
  </div>
</div>
