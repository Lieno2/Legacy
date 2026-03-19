<script lang="ts">
  import { Shield, Plus, Search, X, Pencil, Trash2, Loader2 } from 'lucide-svelte';
  import type { User } from '$lib/types';

  export let users: User[] = [];
  export let usersLoading = false;
  export let INPUT: string;

  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{
    create: void;
    edit: User;
    delete: { type: 'user'; id: string; name: string };
  }>();

  let userSearch = '';

  $: adminCount     = users.filter(u => u.perms >= 999).length;
  $: filteredUsers  = users.filter(u =>
    u.username.toLowerCase().includes(userSearch.toLowerCase()) ||
    u.email.toLowerCase().includes(userSearch.toLowerCase())
  );
</script>

<div class="bg-card border border-border rounded-2xl overflow-hidden">
  <div class="p-5 border-b border-border flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="font-semibold text-sm">Users</h2>
        <p class="text-xs text-muted-foreground mt-0.5">
          {users.length} total · {adminCount} admin{adminCount !== 1 ? 's' : ''}
        </p>
      </div>
      <button on:click={() => dispatch('create')}
        class="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:bg-primary/90 transition">
        <Plus class="w-3.5 h-3.5" /> New User
      </button>
    </div>
    <div class="relative">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
      <input class="{INPUT} pl-9" placeholder="Search by name or email..." bind:value={userSearch} />
      {#if userSearch}
        <button on:click={() => (userSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
          <X class="w-3.5 h-3.5" />
        </button>
      {/if}
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
            <button on:click={() => dispatch('edit', u)}
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition">
              <Pencil class="w-3.5 h-3.5" />
            </button>
            <button on:click={() => dispatch('delete', { type: 'user', id: u.id, name: u.username })}
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
