<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { apiFetch } from '$lib/api';
  import type { Event } from '$lib/types';
  import { X } from 'lucide-svelte';

  export let event: Event | null = null;
  export let defaultDate: string = '';

  const dispatch = createEventDispatcher();

  const COLORS = ['#6366f1','#a855f7','#22c55e','#eab308','#14b8a6','#ec4899','#f97316'];

  // parse default date/time from defaultDate or event
  function parseDefault() {
    const src = event ? event.date.slice(0, 16) : defaultDate;
    if (!src) return { d: '', t: '' };
    const [d, t] = src.split('T');
    return { d, t: t ?? '' };
  }

  const { d: initD, t: initT } = parseDefault();

  let title       = event?.title ?? '';
  let description = event?.description ?? '';
  let dateVal     = initD;
  let timeVal     = initT;
  let location    = event?.location ?? '';
  let color       = event?.color ?? COLORS[0];
  let isPrivate   = event?.private ?? false;
  let saving      = false;
  let error       = '';

  // display date in header
  $: headerDate = dateVal ? new Date(dateVal + 'T00:00').toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }) : '';

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = ''; saving = true;
    try {
      const isoDate = dateVal && timeVal ? `${dateVal}T${timeVal}` : dateVal ? `${dateVal}T00:00` : '';
      const body = { title, description: description || null, date: isoDate, location: location || null, color, private: isPrivate };
      if (event) {
        await apiFetch(`/api/events/${event.id}`, { method: 'PUT', body: JSON.stringify(body) });
      } else {
        await apiFetch('/api/events', { method: 'POST', body: JSON.stringify(body) });
      }
      dispatch('saved');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save';
    } finally { saving = false; }
  }

  const INPUT = 'h-9 w-full rounded-lg border border-input bg-muted/30 px-3 text-sm outline-none focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20 transition placeholder:text-muted-foreground/60';
</script>

<div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
  on:click|self={() => dispatch('cancel')} role="dialog">
  <div class="w-full max-w-md bg-card border border-border rounded-2xl shadow-2xl shadow-black/40 flex flex-col overflow-hidden">
    <!-- Header -->
    <div class="flex items-center gap-2 px-5 pt-5 pb-4 border-b border-border">
      <span class="text-sm text-muted-foreground">{event ? '✏️' : '+'}</span>
      <h2 class="font-semibold text-sm flex-1">
        {event ? 'Edit Event' : 'New Event'}
        {#if headerDate}<span class="text-muted-foreground font-normal"> — {headerDate}</span>{/if}
      </h2>
      <button on:click={() => dispatch('cancel')} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground">
        <X class="w-4 h-4" />
      </button>
    </div>

    <form on:submit={handleSubmit} class="flex flex-col gap-4 p-5">
      <!-- Title -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Title <span class="text-red-400">*</span></label>
        <input class={INPUT} bind:value={title} required placeholder="e.g. Team meeting" />
      </div>

      <!-- Date + Time -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Date</label>
          <input class={INPUT} type="date" bind:value={dateVal} required />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Time <span class="text-muted-foreground/60 normal-case">(optional)</span></label>
          <input class={INPUT} type="time" bind:value={timeVal} />
        </div>
      </div>

      <!-- Description -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Description</label>
        <textarea
          bind:value={description} rows="3" placeholder="Add details..."
          class="w-full rounded-lg border border-input bg-muted/30 px-3 py-2 text-sm outline-none
                 focus:border-ring focus:bg-transparent focus:ring-2 focus:ring-ring/20 transition
                 resize-none placeholder:text-muted-foreground/60"
        ></textarea>
      </div>

      <!-- Location -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Location</label>
        <input class={INPUT} bind:value={location} placeholder="e.g. Office or https://meet.google.com/..." />
      </div>

      <!-- Color + Private -->
      <div class="flex items-center justify-between">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Color</label>
          <div class="flex items-center gap-1.5">
            {#each COLORS as c}
              <button
                type="button"
                on:click={() => (color = c)}
                class="w-6 h-6 rounded-full transition-all
                       {color === c ? 'ring-2 ring-offset-2 ring-offset-card scale-110' : 'opacity-70 hover:opacity-100 hover:scale-105'}"
                style="background:{c}; ring-color:{c};"
                aria-label={c}
              ></button>
            {/each}
          </div>
        </div>

        <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
          <div class="relative">
            <input type="checkbox" bind:checked={isPrivate} class="sr-only peer" />
            <div class="w-9 h-5 rounded-full bg-muted peer-checked:bg-primary transition-colors"></div>
            <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform peer-checked:translate-x-4 shadow"></div>
          </div>
          <span class="text-sm text-muted-foreground">Private</span>
        </label>
      </div>

      {#if error}
        <div class="px-3 py-2.5 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg">{error}</div>
      {/if}

      <div class="flex gap-2 pt-1">
        <button type="button" on:click={() => dispatch('cancel')}
          class="flex-1 h-9 rounded-lg border border-border text-sm hover:bg-muted transition">Cancel</button>
        <button type="submit" disabled={saving}
          class="flex-1 h-9 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                 hover:bg-primary/90 disabled:opacity-50 transition active:scale-[0.98]">
          {saving ? 'Saving…' : event ? 'Update' : 'Create'}
        </button>
      </div>
    </form>
  </div>
</div>
