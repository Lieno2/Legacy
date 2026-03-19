<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { apiFetch } from '$lib/api';
  import type { Event } from '$lib/types';
  import { X } from 'lucide-svelte';

  export let event: Event | null = null;
  export let defaultDate: string = '';

  const dispatch = createEventDispatcher();

  let title       = event?.title ?? '';
  let description = event?.description ?? '';
  let date        = event ? event.date.slice(0, 16) : defaultDate;
  let location    = event?.location ?? '';
  let color       = event?.color ?? '#6366f1';
  let isPrivate   = event?.private ?? false;
  let saving      = false;
  let error       = '';

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error  = '';
    saving = true;
    try {
      const body = { title, description: description || null, date, location: location || null, color, private: isPrivate };
      if (event) {
        await apiFetch(`/api/events/${event.id}`, { method: 'PUT', body: JSON.stringify(body) });
      } else {
        await apiFetch('/api/events', { method: 'POST', body: JSON.stringify(body) });
      }
      dispatch('saved');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save';
    } finally {
      saving = false;
    }
  }

  const INPUT = 'h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm outline-none focus:border-ring focus:ring-2 focus:ring-ring/30 transition';
</script>

<div class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" on:click|self={() => dispatch('cancel')} role="dialog">
  <div class="w-full max-w-md bg-card rounded-xl ring-1 ring-foreground/10 shadow-xl p-6 flex flex-col gap-5">
    <div class="flex items-center justify-between">
      <h2 class="font-semibold">{event ? 'Edit Event' : 'New Event'}</h2>
      <button on:click={() => dispatch('cancel')} class="p-1 rounded hover:bg-muted transition">
        <X class="w-4 h-4" />
      </button>
    </div>

    <form on:submit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium">Title *</label>
        <input class={INPUT} bind:value={title} required placeholder="Event title" />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium">Date & Time *</label>
        <input class={INPUT} type="datetime-local" bind:value={date} required />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium">Location</label>
        <input class={INPUT} bind:value={location} placeholder="Optional location" />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium">Description</label>
        <textarea
          bind:value={description}
          rows="3"
          placeholder="Optional description"
          class="w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none
                 focus:border-ring focus:ring-2 focus:ring-ring/30 transition resize-none"
        ></textarea>
      </div>

      <div class="flex items-center gap-4">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">Color</label>
          <input type="color" bind:value={color} class="h-9 w-16 rounded-md border border-input cursor-pointer bg-transparent" />
        </div>
        <label class="flex items-center gap-2 text-sm cursor-pointer mt-5">
          <input type="checkbox" bind:checked={isPrivate} class="rounded" />
          Private event
        </label>
      </div>

      {#if error}
        <div class="p-3 text-sm text-red-400 bg-red-500/10 border border-red-500/20 rounded-md">{error}</div>
      {/if}

      <div class="flex gap-2 justify-end">
        <button type="button" on:click={() => dispatch('cancel')}
          class="h-9 px-4 rounded-md border border-border text-sm hover:bg-muted transition">
          Cancel
        </button>
        <button type="submit" disabled={saving}
          class="h-9 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/80 disabled:opacity-50 transition">
          {saving ? 'Saving…' : event ? 'Update' : 'Create'}
        </button>
      </div>
    </form>
  </div>
</div>
