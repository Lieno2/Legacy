<script lang="ts">
  import { Lock, Search, X, Trash2, Loader2 } from 'lucide-svelte';
  import type { Event } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  export let events: Event[] = [];
  export let eventsLoading = false;
  export let INPUT: string;

  const dispatch = createEventDispatcher<{
    delete: { type: 'event'; id: number; name: string };
  }>();

  let eventSearch = '';

  $: upcomingCount  = events.filter(e => new Date(e.date) > new Date()).length;
  $: filteredEvents = events.filter(e =>
    e.title.toLowerCase().includes(eventSearch.toLowerCase()) ||
    (e.creator_name ?? '').toLowerCase().includes(eventSearch.toLowerCase())
  );
</script>

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
        <button on:click={() => (eventSearch = '')} class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
          <X class="w-3.5 h-3.5" />
        </button>
      {/if}
    </div>
  </div>

  <div class="divide-y divide-border/50">
    {#if eventsLoading}
      <div class="flex items-center justify-center gap-2 py-8 text-muted-foreground text-sm">
        <Loader2 class="w-4 h-4 animate-spin" /> Loading…
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
              {#if e.private}<Lock class="w-3 h-3 text-muted-foreground shrink-0" />{/if}
              {#if isPast}<span class="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full">Past</span>{/if}
            </div>
            <div class="text-xs text-muted-foreground">
              {new Date(e.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
              {#if e.location} · {e.location}{/if}
              · by {e.creator_name ?? 'Unknown'}
            </div>
          </div>
          <button on:click={() => dispatch('delete', { type: 'event', id: e.id, name: e.title })}
            class="w-7 h-7 rounded-lg flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition shrink-0">
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>
      {/each}
    {/if}
  </div>
</div>
