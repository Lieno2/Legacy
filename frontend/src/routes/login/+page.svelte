<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores';
  import { apiFetch, setTokens } from '$lib/api';
  import type { User } from '$lib/types';

  let email    = '';
  let password = '';
  let error    = '';
  let loading  = false;

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error   = '';
    loading = true;
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
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex items-center justify-center min-h-screen bg-background">
  <div class="w-full max-w-sm rounded-xl bg-card ring-1 ring-foreground/10 shadow p-8 flex flex-col gap-6">
    <div>
      <h1 class="text-lg font-semibold">Sign in</h1>
      <p class="text-sm text-muted-foreground mt-1">Enter your credentials to access your account</p>
    </div>

    <form on:submit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <label for="email" class="text-sm font-medium">Email</label>
        <input
          id="email" type="email" bind:value={email} required disabled={loading}
          placeholder="you@example.com"
          class="h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm outline-none
                 focus:border-ring focus:ring-2 focus:ring-ring/30 disabled:opacity-50 transition"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="password" class="text-sm font-medium">Password</label>
        <input
          id="password" type="password" bind:value={password} required disabled={loading}
          placeholder="Enter your password"
          class="h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm outline-none
                 focus:border-ring focus:ring-2 focus:ring-ring/30 disabled:opacity-50 transition"
        />
      </div>

      {#if error}
        <div class="flex items-center gap-2 p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md">
          <span>⚠</span> {error}
        </div>
      {/if}

      <button
        type="submit" disabled={loading}
        class="h-9 w-full rounded-md bg-primary text-primary-foreground text-sm font-medium
               hover:bg-primary/80 disabled:opacity-50 transition"
      >
        {loading ? 'Signing in…' : 'Sign in'}
      </button>
    </form>
  </div>
</div>
