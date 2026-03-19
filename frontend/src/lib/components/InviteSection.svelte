<script lang="ts">
  import { Search, Plus, UserMinus, Loader2 } from 'lucide-svelte';
  import { apiFetch } from '$lib/api';

  export let eventId: number;
  export let BASE_INPUT: string;

  interface InviteUser { id: string; username: string; email: string; }

  let searchQ       = '';
  let searchResults: InviteUser[] = [];
  let searchLoading = false;
  let invited: InviteUser[]       = [];
  let inviteLoading: Record<string, boolean> = {};
  let searchDebounce: ReturnType<typeof setTimeout>;

  // Load current invites on mount
  $: if (eventId) loadInvited();

  async function loadInvited() {
    try { invited = await apiFetch<InviteUser[]>(`/api/invites?event_id=${eventId}`); }
    catch { invited = []; }
  }

  function onSearchInput() {
    clearTimeout(searchDebounce);
    if (!searchQ.trim()) { searchResults = []; return; }
    searchDebounce = setTimeout(async () => {
      searchLoading = true;
      try {
        searchResults = await apiFetch<InviteUser[]>(
          `/api/invites/search?q=${encodeURIComponent(searchQ)}&event_id=${eventId}`
        );
      } catch { searchResults = []; }
      finally { searchLoading = false; }
    }, 300);
  }

  async function addInvite(u: InviteUser) {
    inviteLoading[u.id] = true;
    try {
      await apiFetch('/api/invites', { method: 'POST', body: JSON.stringify({ event_id: eventId, user_id: u.id }) });
      searchResults = searchResults.filter(r => r.id !== u.id);
      searchQ = '';
      await loadInvited();
    } catch { /* ignore */ }
    finally { inviteLoading[u.id] = false; }
  }

  async function removeInvite(u: InviteUser) {
    inviteLoading[u.id] = true;
    try {
      await apiFetch('/api/invites', { method: 'DELETE', body: JSON.stringify({ event_id: eventId, user_id: u.id }) });
      await loadInvited();
    } catch { /* ignore */ }
    finally { inviteLoading[u.id] = false; }
  }
</script>

<div class="flex flex-col gap-3">
  <p class="text-xs text-muted-foreground -mt-1">Add people who can see this private event and RSVP to it.</p>

  <div class="relative">
    <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
    <input
      bind:value={searchQ}
      on:input={onSearchInput}
      placeholder="Search by username or email..."
      class="{BASE_INPUT} h-9 border-input pl-8"
    />
    {#if searchLoading}
      <Loader2 class="absolute right-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground animate-spin" />
    {/if}
  </div>

  {#if searchResults.length > 0}
    <div class="flex flex-col gap-px rounded-xl border border-border bg-muted/30 overflow-hidden -mt-1">
      {#each searchResults as u}
        <button type="button" on:click={() => addInvite(u)} disabled={inviteLoading[u.id]}
          class="flex items-center gap-3 px-3 py-2 hover:bg-muted transition text-left disabled:opacity-50">
          <div class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold uppercase shrink-0"
            style="background:hsl({(u.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(u.username.charCodeAt(0)*47)%360},70%,70%);"
          >{u.username[0]}</div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate">{u.username}</p>
            <p class="text-xs text-muted-foreground truncate">{u.email}</p>
          </div>
          {#if inviteLoading[u.id]}
            <Loader2 class="w-3.5 h-3.5 animate-spin text-muted-foreground" />
          {:else}
            <Plus class="w-3.5 h-3.5 text-muted-foreground" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  {#if invited.length > 0}
    <div class="flex flex-col gap-1">
      <p class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Invited ({invited.length})</p>
      {#each invited as u}
        <div class="flex items-center gap-3 px-3 py-2 rounded-xl bg-muted/20 border border-border">
          <div class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold uppercase shrink-0"
            style="background:hsl({(u.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(u.username.charCodeAt(0)*47)%360},70%,70%);"
          >{u.username[0]}</div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate">{u.username}</p>
            <p class="text-xs text-muted-foreground truncate">{u.email}</p>
          </div>
          <button type="button" on:click={() => removeInvite(u)} disabled={inviteLoading[u.id]}
            aria-label="Remove {u.username}"
            class="w-6 h-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-red-400 hover:bg-red-500/10 transition disabled:opacity-40">
            {#if inviteLoading[u.id]}
              <Loader2 class="w-3.5 h-3.5 animate-spin" />
            {:else}
              <UserMinus class="w-3.5 h-3.5" />
            {/if}
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-xs text-muted-foreground">No one invited yet.</p>
  {/if}
</div>
