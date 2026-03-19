<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { formatDate, formatTime } from '$lib/utils';
  import type { Event, EventMember, RsvpStatus } from '$lib/types';
  import { X, MapPin, Clock, Lock, Pencil, Trash2, Users, Check, Timer, XCircle, AlertCircle, Loader2 } from 'lucide-svelte';

  export let event: Event;
  export let currentUserId: string | null;

  const dispatch = createEventDispatcher();

  let members: EventMember[] = [];
  let myStatus: RsvpStatus | null = null;
  let lateMinutes = 0;
  let rsvpLoading = false;
  let membersLoading = true;
  let membersError = '';
  let rsvpError = '';

  onMount(async () => {
    await loadMembers();
  });

  async function loadMembers() {
    membersLoading = true;
    membersError = '';
    try {
      members = await apiFetch<EventMember[]>(`/api/rsvp?event_id=${event.id}`);
      const me = members.find(m => m.user_id === currentUserId);
      myStatus    = (me?.status as RsvpStatus) ?? null;
      lateMinutes = me?.late_minutes ?? 0;
    } catch (err: unknown) {
      membersError = err instanceof Error ? err.message : 'Failed to load attendees';
    } finally {
      membersLoading = false;
    }
  }

  async function rsvp(status: RsvpStatus) {
    rsvpLoading = true;
    rsvpError = '';
    try {
      await apiFetch('/api/rsvp', {
        method: 'POST',
        body: JSON.stringify({
          event_id: event.id,
          status,
          late_minutes: status === 'late' ? (lateMinutes || null) : null
        })
      });
      myStatus = status;
      await loadMembers();
    } catch (err: unknown) {
      rsvpError = err instanceof Error ? err.message : 'Failed to save RSVP';
    } finally {
      rsvpLoading = false;
    }
  }

  $: isOwner = currentUserId != null && currentUserId === event.created_by;

  const RSVP_BTNS: { status: RsvpStatus; label: string; icon: any; active: string }[] = [
    { status: 'going',     label: 'Going',       icon: Check,   active: 'border-emerald-500/50 bg-emerald-500/10 text-emerald-400' },
    { status: 'late',      label: 'Coming late', icon: Timer,   active: 'border-amber-500/50  bg-amber-500/10  text-amber-400'  },
    { status: 'not_going', label: 'Not going',   icon: XCircle, active: 'border-red-500/50    bg-red-500/10    text-red-400'    },
  ];

  const STATUS_BADGE: Record<RsvpStatus, string> = {
    going:     'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
    late:      'bg-amber-500/15  text-amber-400  border-amber-500/30',
    not_going: 'bg-red-500/15    text-red-400    border-red-500/30',
  };
  const STATUS_LABEL: Record<RsvpStatus, string> = {
    going: 'Going', late: 'Late', not_going: 'Not going'
  };
</script>

<div
  class="fixed inset-0 z-50 flex items-end sm:items-center justify-center p-4"
  style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);"
  on:click|self={() => dispatch('close')}
  role="dialog" aria-modal="true"
>
  <div
    class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl shadow-black/40 overflow-hidden"
    style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;"
  >
    <div class="h-0.5 w-full" style="background:{event.color ?? '#6366f1'};"></div>

    <div class="p-5 flex flex-col gap-4">
      <!-- Header -->
      <div class="flex items-start justify-between gap-3">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <h2 class="font-semibold text-base leading-tight">{event.title}</h2>
            {#if event.private}
              <span class="inline-flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-full border border-border bg-muted text-muted-foreground">
                <Lock class="w-2.5 h-2.5" /> Private
              </span>
            {/if}
          </div>
          <p class="text-xs text-muted-foreground mt-0.5">by {event.creator_name ?? 'Unknown'}</p>
        </div>
        <div class="flex items-center gap-0.5 shrink-0">
          {#if isOwner}
            <button on:click={() => dispatch('edit')}
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
              <Pencil class="w-3.5 h-3.5" />
            </button>
            <button on:click={() => dispatch('delete')}
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-red-500/10 transition text-muted-foreground hover:text-red-400">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          {/if}
          <button on:click={() => dispatch('close')}
            class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Meta -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <Clock class="w-3.5 h-3.5 shrink-0" />
          <span>{formatDate(event.date)} · {formatTime(event.date)}</span>
        </div>
        {#if event.location}
          <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <MapPin class="w-3.5 h-3.5 shrink-0" />
            <span>{event.location}</span>
          </div>
        {/if}
      </div>

      {#if event.description}
        <p class="text-sm text-muted-foreground leading-relaxed">{event.description}</p>
      {/if}

      <!-- RSVP section (public events only) -->
      {#if !event.private}
        <div class="border-t border-border pt-4 flex flex-col gap-2.5">
          <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Will you attend?</p>
          <div class="grid grid-cols-3 gap-1.5">
            {#each RSVP_BTNS as btn}
              <button
                on:click={() => rsvp(btn.status)}
                disabled={rsvpLoading}
                class="h-8 rounded-lg border text-xs font-medium transition-all active:scale-95
                       flex items-center justify-center gap-1
                       {myStatus === btn.status ? btn.active : 'border-border text-muted-foreground hover:bg-muted hover:text-foreground'}
                       disabled:opacity-40 disabled:cursor-not-allowed"
              >
                {#if rsvpLoading && myStatus === btn.status}
                  <Loader2 class="w-3 h-3 animate-spin" />
                {:else}
                  <svelte:component this={btn.icon} class="w-3 h-3" />
                {/if}
                {btn.label}
              </button>
            {/each}
          </div>

          {#if rsvpError}
            <div class="flex items-center gap-2 text-xs text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
              <AlertCircle class="w-3.5 h-3.5 shrink-0" /> {rsvpError}
            </div>
          {/if}

          {#if myStatus === 'late'}
            <div class="flex items-center gap-2">
              <label class="text-xs text-muted-foreground shrink-0">Minutes late</label>
              <input
                type="number" min="1" max="120"
                bind:value={lateMinutes}
                class="h-7 w-20 rounded-md border border-input bg-transparent px-2 text-sm outline-none focus:border-ring transition"
              />
              <button
                on:click={() => rsvp('late')}
                disabled={rsvpLoading}
                class="text-xs text-primary hover:underline disabled:opacity-40"
              >Update</button>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Attendees -->
      <div class="border-t border-border pt-4 flex flex-col gap-2">
        <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
          <Users class="w-3.5 h-3.5" />
          Attendees {#if !membersLoading}({members.length}){/if}
        </div>

        {#if membersLoading}
          <div class="flex items-center gap-2 text-xs text-muted-foreground py-1">
            <Loader2 class="w-3.5 h-3.5 animate-spin" /> Loading...
          </div>
        {:else if membersError}
          <div class="flex items-center gap-2 text-xs text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
            <AlertCircle class="w-3.5 h-3.5 shrink-0" /> {membersError}
          </div>
        {:else if members.length === 0}
          <p class="text-xs text-muted-foreground">No RSVPs yet — be the first!</p>
        {:else}
          <div class="flex flex-col gap-1.5">
            {#each members as m}
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div
                    class="w-5 h-5 rounded-full border border-border flex items-center justify-center text-[10px] font-bold uppercase"
                    style="background:hsl({((m.username ?? '?').charCodeAt(0)*47)%360},40%,25%); color:hsl({((m.username ?? '?').charCodeAt(0)*47)%360},70%,70%);"
                  >
                    {(m.username ?? '?')[0]}
                  </div>
                  <span class="text-sm">{m.username ?? 'Unknown'}</span>
                </div>
                <span class="text-[11px] px-1.5 py-0.5 rounded-full border {STATUS_BADGE[m.status as RsvpStatus]}">
                  {STATUS_LABEL[m.status as RsvpStatus]}{m.status === 'late' && m.late_minutes ? ` +${m.late_minutes}m` : ''}
                </span>
              </div>
            {/each}
          </div>
        {/if}
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
