<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { formatDate, formatTime } from '$lib/utils';
  import type { Event, EventMember, RsvpStatus } from '$lib/types';
  import { X, MapPin, Clock, Lock, Pencil, Trash2, Users } from 'lucide-svelte';

  export let event: Event;
  export let currentUserId: string | null;

  const dispatch = createEventDispatcher();

  let members: EventMember[] = [];
  let myStatus: RsvpStatus | null = null;
  let lateMinutes = 0;
  let rsvpLoading = false;

  onMount(async () => {
    try {
      members = await apiFetch<EventMember[]>(`/api/rsvp?event_id=${event.id}`);
      const me = members.find(m => m.user_id === currentUserId);
      myStatus    = me?.status ?? null;
      lateMinutes = me?.late_minutes ?? 0;
    } catch {}
  });

  async function rsvp(status: RsvpStatus) {
    rsvpLoading = true;
    try {
      await apiFetch('/api/rsvp', {
        method: 'POST',
        body: JSON.stringify({ event_id: event.id, status, late_minutes: status === 'late' ? lateMinutes : null })
      });
      myStatus = status;
      members = await apiFetch<EventMember[]>(`/api/rsvp?event_id=${event.id}`);
    } catch {} finally {
      rsvpLoading = false;
    }
  }

  $: isOwner = currentUserId === event.created_by;

  const STATUS_LABELS: Record<RsvpStatus, string> = {
    going:     'Going',
    late:      'Late',
    not_going: 'Not going'
  };

  const STATUS_COLORS: Record<RsvpStatus, string> = {
    going:     'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
    late:      'bg-amber-500/15 text-amber-400 border-amber-500/30',
    not_going: 'bg-red-500/15 text-red-400 border-red-500/30'
  };
</script>

<div class="fixed inset-0 z-50 bg-black/40 flex items-end sm:items-center justify-center p-4" on:click|self={() => dispatch('close')} role="dialog">
  <div class="w-full max-w-md bg-card rounded-xl ring-1 ring-foreground/10 shadow-xl flex flex-col gap-0 overflow-hidden">
    <!-- Color bar -->
    <div class="h-1.5 w-full" style="background:{event.color ?? '#6366f1'}"></div>

    <div class="p-6 flex flex-col gap-4">
      <!-- Header -->
      <div class="flex items-start justify-between gap-3">
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-2">
            <h2 class="font-semibold text-base">{event.title}</h2>
            {#if event.private}
              <Lock class="w-3.5 h-3.5 text-muted-foreground" />
            {/if}
          </div>
          <p class="text-xs text-muted-foreground">by {event.creator_name ?? 'Unknown'}</p>
        </div>
        <div class="flex items-center gap-1">
          {#if isOwner}
            <button on:click={() => dispatch('edit')} class="p-1.5 rounded hover:bg-muted transition">
              <Pencil class="w-3.5 h-3.5" />
            </button>
            <button on:click={() => dispatch('delete')} class="p-1.5 rounded hover:bg-muted text-destructive transition">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          {/if}
          <button on:click={() => dispatch('close')} class="p-1.5 rounded hover:bg-muted transition">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Meta -->
      <div class="flex flex-col gap-1.5 text-sm">
        <div class="flex items-center gap-2 text-muted-foreground">
          <Clock class="w-3.5 h-3.5" />
          <span>{formatDate(event.date)} at {formatTime(event.date)}</span>
        </div>
        {#if event.location}
          <div class="flex items-center gap-2 text-muted-foreground">
            <MapPin class="w-3.5 h-3.5" />
            <span>{event.location}</span>
          </div>
        {/if}
      </div>

      {#if event.description}
        <p class="text-sm text-muted-foreground">{event.description}</p>
      {/if}

      <!-- RSVP buttons -->
      {#if !event.private}
        <div class="border-t border-border/50 pt-4 flex flex-col gap-3">
          <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Your RSVP</p>
          <div class="flex gap-2">
            {#each (['going', 'late', 'not_going'] as RsvpStatus[]) as s}
              <button
                on:click={() => rsvp(s)}
                disabled={rsvpLoading}
                class="flex-1 h-8 rounded-md border text-xs font-medium transition
                       {myStatus === s ? STATUS_COLORS[s] : 'border-border text-muted-foreground hover:bg-muted'}"
              >
                {STATUS_LABELS[s]}
              </button>
            {/each}
          </div>
          {#if myStatus === 'late'}
            <div class="flex items-center gap-2">
              <label class="text-xs text-muted-foreground">Minutes late</label>
              <input
                type="number" min="1" max="120" bind:value={lateMinutes}
                class="h-7 w-20 rounded border border-input bg-transparent px-2 text-sm outline-none focus:border-ring transition"
              />
              <button on:click={() => rsvp('late')} class="text-xs text-primary hover:underline">Update</button>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Members -->
      <div class="border-t border-border/50 pt-4 flex flex-col gap-2">
        <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wide">
          <Users class="w-3.5 h-3.5" />
          Attendees ({members.length})
        </div>
        {#if members.length === 0}
          <p class="text-xs text-muted-foreground">No RSVPs yet</p>
        {:else}
          <div class="flex flex-col gap-1">
            {#each members as m}
              <div class="flex items-center justify-between text-sm">
                <span>{m.username ?? 'Unknown'}</span>
                <span class="text-xs px-1.5 py-0.5 rounded-full border {STATUS_COLORS[m.status]}">
                  {STATUS_LABELS[m.status]}{m.status === 'late' && m.late_minutes ? ` (+${m.late_minutes}m)` : ''}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
