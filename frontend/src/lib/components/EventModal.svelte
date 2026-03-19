<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { apiFetch } from '$lib/api';
  import type { Event } from '$lib/types';
  import { X, MapPin, AlignLeft, Lock, Calendar, Clock, Edit3, Plus, AlertCircle } from 'lucide-svelte';

  export let event: Event | null = null;
  export let defaultDate: string = '';

  const dispatch = createEventDispatcher();

  const COLORS = [
    { hex: '#6366f1', name: 'Indigo'  },
    { hex: '#a855f7', name: 'Purple'  },
    { hex: '#22c55e', name: 'Green'   },
    { hex: '#eab308', name: 'Yellow'  },
    { hex: '#14b8a6', name: 'Teal'    },
    { hex: '#ec4899', name: 'Pink'    },
    { hex: '#f97316', name: 'Orange'  },
    { hex: '#ef4444', name: 'Red'     },
    { hex: '#3b82f6', name: 'Blue'    },
  ];

  function parseDefault() {
    const src = event ? event.date.slice(0, 16) : defaultDate;
    if (!src) return { d: '', t: '09:00' };
    const [d, t] = src.split('T');
    return { d, t: t?.slice(0, 5) ?? '09:00' };
  }

  const { d: initD, t: initT } = parseDefault();

  let title       = event?.title ?? '';
  let description = event?.description ?? '';
  let dateVal     = initD;
  let timeVal     = initT || '09:00';
  let location    = event?.location ?? '';
  let color       = event?.color ?? COLORS[0].hex;
  let isPrivate   = event?.private ?? false;
  let saving      = false;
  let fieldErrors: Record<string, string> = {};

  $: headerDate = dateVal
    ? new Date(dateVal + 'T00:00').toLocaleDateString('en-US', { weekday: 'short', month: 'long', day: 'numeric' })
    : '';

  function validate(): boolean {
    fieldErrors = {};
    if (!title.trim()) fieldErrors.title = 'Title is required';
    if (!dateVal)       fieldErrors.date  = 'Date is required';
    return Object.keys(fieldErrors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) return;
    saving = true;
    try {
      const iso = `${dateVal}T${timeVal}:00Z`;
      const body = {
        title:       title.trim(),
        description: description.trim() || null,
        date:        iso,
        location:    location.trim() || null,
        color,
        private:     isPrivate
      };
      if (event) {
        await apiFetch(`/api/events/${event.id}`, { method: 'PUT',  body: JSON.stringify(body) });
      } else {
        await apiFetch('/api/events',              { method: 'POST', body: JSON.stringify(body) });
      }
      dispatch('saved');
    } catch (err: unknown) {
      fieldErrors.global = err instanceof Error ? err.message : 'Failed to save event';
    } finally { saving = false; }
  }

  const BASE_INPUT = [
    'w-full rounded-lg border bg-muted/20 px-3 text-sm outline-none transition',
    'focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20',
    'placeholder:text-muted-foreground/50'
  ].join(' ');

  function inputCls(field: string) {
    return `${BASE_INPUT} h-9 ${fieldErrors[field] ? 'border-red-500/60 bg-red-500/5' : 'border-input'}`;
  }
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center p-4"
  style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);"
  on:click|self={() => dispatch('cancel')}
  role="dialog" aria-modal="true"
>
  <div
    class="w-full max-w-md bg-card border border-border rounded-2xl shadow-2xl shadow-black/50 flex flex-col overflow-hidden"
    style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;"
  >
    <div class="h-0.5 w-full" style="background: linear-gradient(90deg, {color}, {color}55);"></div>

    <div class="flex items-center gap-2.5 px-5 py-4 border-b border-border">
      <div class="w-6 h-6 rounded-md flex items-center justify-center shrink-0"
        style="background:{color}20; border:1px solid {color}40;">
        {#if event}
          <Edit3 class="w-3.5 h-3.5" style="color:{color};" />
        {:else}
          <Plus class="w-3.5 h-3.5" style="color:{color};" />
        {/if}
      </div>
      <h2 class="text-sm font-semibold flex-1 leading-tight">
        {event ? 'Edit event' : 'New event'}{#if headerDate}<span class="text-muted-foreground font-normal"> — {headerDate}</span>{/if}
      </h2>
      <button
        on:click={() => dispatch('cancel')}
        class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex flex-col gap-4 p-5 overflow-y-auto max-h-[80vh]">

      <!-- Title -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Title</label>
        <input
          bind:value={title}
          placeholder="e.g. Team meeting"
          class={inputCls('title')}
          on:input={() => { delete fieldErrors.title; fieldErrors = fieldErrors; }}
        />
        {#if fieldErrors.title}
          <p class="flex items-center gap-1 text-xs text-red-400">
            <AlertCircle class="w-3 h-3 shrink-0" /> {fieldErrors.title}
          </p>
        {/if}
      </div>

      <!-- Date + Time -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
            <Calendar class="w-3 h-3" /> Date
          </label>
          <input
            type="date"
            bind:value={dateVal}
            class={inputCls('date')}
            on:change={() => { delete fieldErrors.date; fieldErrors = fieldErrors; }}
          />
          {#if fieldErrors.date}
            <p class="flex items-center gap-1 text-xs text-red-400">
              <AlertCircle class="w-3 h-3 shrink-0" /> {fieldErrors.date}
            </p>
          {/if}
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
            <Clock class="w-3 h-3" /> Time
          </label>
          <input type="time" bind:value={timeVal} class="{BASE_INPUT} h-9 border-input" />
        </div>
      </div>

      <!-- Description -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
          <AlignLeft class="w-3 h-3" /> Description
          <span class="normal-case font-normal text-muted-foreground/50 ml-0.5">(optional)</span>
        </label>
        <textarea
          bind:value={description}
          rows="3"
          placeholder="Add more details about this event..."
          class="{BASE_INPUT} py-2 resize-none border-input"
        ></textarea>
      </div>

      <!-- Location -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
          <MapPin class="w-3 h-3" /> Location
          <span class="normal-case font-normal text-muted-foreground/50 ml-0.5">(optional)</span>
        </label>
        <input
          bind:value={location}
          placeholder="Office, Zoom link, address..."
          class="{BASE_INPUT} h-9 border-input"
        />
      </div>

      <!-- Color + Private -->
      <div class="flex items-end justify-between gap-4 pt-1 border-t border-border">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Color</label>
          <!-- padding gives room for the scale(1.25) selected swatch without clipping -->
          <div class="flex items-center gap-2 py-1 px-0.5">
            {#each COLORS as c}
              <button
                type="button"
                on:click={() => (color = c.hex)}
                title={c.name}
                class="w-4 h-4 rounded-full transition-all duration-150 shrink-0
                       {color === c.hex
                         ? 'scale-125 ring-2 ring-offset-2 ring-offset-card'
                         : 'opacity-50 hover:opacity-100 hover:scale-110'}"
                style="background:{c.hex}; --tw-ring-color:{c.hex};"
              ></button>
            {/each}
          </div>
        </div>

        <label class="flex items-center gap-2 cursor-pointer select-none shrink-0 pb-1">
          <span class="text-sm text-muted-foreground flex items-center gap-1.5">
            <Lock class="w-3.5 h-3.5" /> Private
          </span>
          <div class="relative">
            <input type="checkbox" bind:checked={isPrivate} class="sr-only peer" />
            <div class="w-9 h-5 rounded-full border transition-colors duration-200
                        bg-muted border-border peer-checked:bg-primary peer-checked:border-primary"></div>
            <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow-sm
                        transition-transform duration-200 peer-checked:translate-x-4"></div>
          </div>
        </label>
      </div>

      <!-- Global error -->
      {#if fieldErrors.global}
        <div class="flex items-start gap-2.5 px-3 py-3 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-xl">
          <AlertCircle class="w-4 h-4 mt-0.5 shrink-0 text-red-400" />
          <span>{fieldErrors.global}</span>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-2 pt-1">
        <button
          type="button"
          on:click={() => dispatch('cancel')}
          class="flex-1 h-9 rounded-xl border border-border text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition"
        >Cancel</button>
        <button
          type="button"
          on:click={handleSubmit}
          disabled={saving}
          class="flex-1 h-9 rounded-xl text-sm font-semibold transition active:scale-[0.97] disabled:opacity-50 text-white"
          style="background:{color};"
        >
          {saving ? 'Saving…' : event ? 'Update event' : 'Create event'}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes modal-in {
    from { opacity: 0; transform: scale(0.94) translateY(8px); }
    to   { opacity: 1; transform: scale(1)    translateY(0);   }
  }
</style>
