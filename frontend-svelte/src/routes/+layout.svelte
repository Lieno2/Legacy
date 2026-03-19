<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores';
  import { apiFetch } from '$lib/api';
  import type { User } from '$lib/types';

  onMount(async () => {
    try {
      const user = await apiFetch<User>('/api/auth/me');
      auth.set({ user, loading: false });
    } catch {
      auth.set({ user: null, loading: false });
    }
  });
</script>

<slot />
