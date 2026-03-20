<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Clock, MapPin, ChevronRight } from 'lucide-svelte';
  import { formatTime } from '$lib/utils';
  import type { Event } from '$lib/types';

  export let events: Event[] = [];

  const dispatch = createEventDispatcher<{ open: Event }>();

  const now = new Date();
  const in48h = new Date(now.getTime() + 48 * 60 * 60 * 1000);

  $: soonEvents = events
    .filter(e => {
      const d = new Date(e.date);
      return d >= now && d <= in48h;
    })
    .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())
    .slice(0, 5);

  function dayLabel(dateStr: string): string {
    const d = new Date(dateStr);
    if (d.toDateString() === now.toDateString()) return 'Today';
    const tomorrow = new Date(now.getTime() + 86400000);
    if (d.toDateString() === tomorrow.toDateString()) return 'Tomorrow';
    return d.toLocaleDateString([], { weekday: 'short' });
  }

  function hasTime(dateStr: string): boolean {
    const d = new Date(dateStr);
    return d.getHours() !== 0 || d.getMinutes() !== 0;
  }
</script>

{#if soonEvents.length > 0}
  <div class="border-b border-border bg-amber-500/5 px-4 py-2 shrink-0">
    <div class="flex items-center gap-2 overflow-x-auto scrollbar-none">
      <!-- Label -->
      <div class="flex items-center gap-1.5 shrink-0 text-amber-500 text-xs font-semibold uppercase tracking-wide">
        <Clock class="w-3.5 h-3.5" />
        Soon
      </div>
      <div class="w-px h-4 bg-border/50 shrink-0"></div>

      <!-- Pills -->
      {#each soonEvents as ev}
        <button
          on:click={() => dispatch('open', ev)}
          class="flex items-center gap-1.5 bg-background border border-border/50 rounded-full px-3 py-1
                 text-xs hover:border-amber-500/50 hover:bg-amber-500/5 transition-colors shrink-0 group"
        >
          <span class="w-2 h-2 rounded-full shrink-0" style="background:{ev.color ?? '#6366f1'};"></span>
          <span class="font-medium text-amber-600 dark:text-amber-400">
            {dayLabel(ev.date)}{hasTime(ev.date) ? ` ${formatTime(ev.date)}` : ''}
          </span>
          <span class="text-foreground truncate max-w-[120px]">{ev.title}</span>
          {#if ev.location}
            <MapPin class="w-3 h-3 text-muted-foreground shrink-0" />
          {/if}
          <ChevronRight class="w-3 h-3 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity shrink-0" />
        </button>
      {/each}
    </div>
  </div>
{/if}