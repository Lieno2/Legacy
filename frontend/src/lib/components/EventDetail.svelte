<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { formatDate, formatTime } from '$lib/utils';
  import type { Event, EventMember, RsvpStatus, Poll } from '$lib/types';
  import { X, MapPin, Clock, Lock, Pencil, Trash2, Users, Check, Timer, XCircle, AlertCircle, Loader2, Minus, Plus, Link, CheckCheck } from 'lucide-svelte';
  import PollAnswerModal from './PollAnswerModal.svelte';
  import PollResults from './PollResults.svelte';
  import Avatar from './Avatar.svelte';

  export let event: Event;
  export let currentUserId: string | null;

  const dispatch = createEventDispatcher();

  let members: EventMember[] = [];
  let myStatus: RsvpStatus | null = null;
  let lateMinutes = 5;
  let rsvpLoading = false;
  let membersLoading = true;
  let membersError = '';
  let rsvpError = '';
  let linkCopied = false;

  // ── Poll state ────────────────────────────────────────────────────────────
  let poll: Poll | null = null;
  let pendingRsvpStatus: RsvpStatus | null = null;
  let pollAnswerSaving = false;
  $: showPollModal = pendingRsvpStatus !== null && poll !== null && poll.my_choice_ids.length === 0;

  onMount(async () => {
    await Promise.all([loadMembers(), loadPoll()]);
  });

  async function loadPoll() {
    try {
      poll = await apiFetch<Poll | null>(`/api/polls?event_id=${event.id}`);
    } catch { poll = null; }
  }

  async function loadMembers() {
    membersLoading = true;
    membersError = '';
    try {
      members = await apiFetch<EventMember[]>(`/api/rsvp?event_id=${event.id}`);
      const me = members.find(m => m.user_id === currentUserId);
      myStatus    = (me?.status as RsvpStatus) ?? null;
      lateMinutes = me?.late_minutes ?? 5;
    } catch (err: unknown) {
      membersError = err instanceof Error ? err.message : 'Failed to load attendees';
    } finally {
      membersLoading = false;
    }
  }

  async function rsvp(status: RsvpStatus) {
    if (poll && poll.my_choice_ids.length === 0 && (status === 'going' || status === 'late')) {
      pendingRsvpStatus = status;
      return;
    }
    await submitRsvp(status);
  }

  async function submitRsvp(status: RsvpStatus) {
    rsvpLoading = true;
    rsvpError = '';
    try {
      await apiFetch('/api/rsvp', {
        method: 'POST',
        body: JSON.stringify({
          event_id:    event.id,
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

  async function onPollConfirm(e: CustomEvent<{ choiceIds: number[] }>) {
    if (!poll || !pendingRsvpStatus) return;
    pollAnswerSaving = true;
    try {
      await apiFetch('/api/polls/answer', {
        method: 'POST',
        body: JSON.stringify({ poll_id: poll.id, choice_ids: e.detail.choiceIds })
      });
      poll = { ...poll, my_choice_ids: e.detail.choiceIds };
      await submitRsvp(pendingRsvpStatus);
      await loadPoll();
    } catch (err: unknown) {
      rsvpError = err instanceof Error ? err.message : 'Failed to save poll answer';
    } finally {
      pollAnswerSaving = false;
      pendingRsvpStatus = null;
    }
  }

  async function onPollSkip() {
    const status = pendingRsvpStatus;
    pendingRsvpStatus = null;
    if (status) await submitRsvp(status);
  }

  function handleBackdropKey(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('close');
  }

  function clampMinutes(v: number) {
    lateMinutes = Math.min(120, Math.max(1, Math.round(v)));
  }

  function onMinutesInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value);
    if (!isNaN(v)) clampMinutes(v);
  }

  async function copyShareLink() {
    if (!event.share_token) return;
    const url = `${window.location.origin}/event/${event.share_token}`;
    await navigator.clipboard.writeText(url);
    linkCopied = true;
    setTimeout(() => (linkCopied = false), 2000);
  }

  $: isOwner = currentUserId != null && currentUserId === event.created_by;
  $: eventColor = event.color ?? '#6366f1';
  $: canShare = !event.private && !!event.share_token;

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
  const STATUS_LABEL: Record<RsvpStatus, string> = { going: 'Going', late: 'Late', not_going: 'Not going' };
</script>

{#if showPollModal && poll}
  <PollAnswerModal
    {poll}
    {eventColor}
    saving={pollAnswerSaving}
    on:confirm={onPollConfirm}
    on:skip={onPollSkip}
  />
{/if}

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="fixed inset-0 z-50 flex items-end sm:items-center justify-center p-4"
  style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  on:click|self={() => dispatch('close')}
  on:keydown={handleBackdropKey}
>
  <div
    class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl shadow-black/40 overflow-hidden"
    style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;"
  >
    <div class="h-0.5 w-full" style="background:{eventColor};"></div>

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
          <!-- Share button — only for public events with a token -->
          {#if canShare}
            <button
              on:click={copyShareLink}
              aria-label="Copy share link"
              title={linkCopied ? 'Link copied!' : 'Copy share link'}
              class="w-7 h-7 rounded-lg flex items-center justify-center transition
                     {linkCopied
                       ? 'text-emerald-400 bg-emerald-500/10'
                       : 'hover:bg-muted text-muted-foreground hover:text-foreground'}"
            >
              {#if linkCopied}
                <CheckCheck class="w-3.5 h-3.5" />
              {:else}
                <Link class="w-3.5 h-3.5" />
              {/if}
            </button>
          {/if}
          {#if isOwner}
            <button on:click={() => dispatch('edit')} aria-label="Edit event"
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
              <Pencil class="w-3.5 h-3.5" />
            </button>
            <button on:click={() => dispatch('delete')} aria-label="Delete event"
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-red-500/10 transition text-muted-foreground hover:text-red-400">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          {/if}
          <button on:click={() => dispatch('close')} aria-label="Close"
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
            <span class="flex-1">{event.location}</span>
            <a
              href={event.location.startsWith('http') ? event.location : `https://maps.google.com/?q=${encodeURIComponent(event.location)}`}
              target="_blank" rel="noopener noreferrer"
              class="text-xs underline underline-offset-3 hover:no-underline shrink-0"
              style="color:{eventColor};"
            >Maps</a>
          </div>
        {/if}
      </div>

      {#if event.description}
        <p class="text-sm text-muted-foreground leading-relaxed">{event.description}</p>
      {/if}

      <!-- RSVP -->
      <div class="border-t border-border pt-4 flex flex-col gap-2.5">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Will you attend?</p>
        <div class="grid grid-cols-3 gap-1.5">
          {#each RSVP_BTNS as btn}
            <button
              on:click={() => rsvp(btn.status)}
              disabled={rsvpLoading}
              aria-pressed={myStatus === btn.status}
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
          <div class="flex items-center gap-2 bg-amber-500/5 border border-amber-500/20 rounded-xl px-3 py-2.5">
            <Timer class="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span class="text-xs text-muted-foreground flex-1">Minutes late</span>
            <div class="flex items-center gap-1">
              <button type="button" on:click={() => clampMinutes(lateMinutes - 5)} disabled={lateMinutes <= 1}
                aria-label="Decrease"
                class="w-6 h-6 rounded-md flex items-center justify-center bg-muted hover:bg-muted/80
                       text-muted-foreground hover:text-foreground transition active:scale-90 disabled:opacity-30"
              ><Minus class="w-3 h-3" /></button>
              <input type="number" min="1" max="120" value={lateMinutes}
                on:change={onMinutesInput} on:blur={onMinutesInput}
                class="w-10 text-center text-sm font-semibold tabular-nums text-amber-400
                       bg-transparent border border-amber-500/25 rounded-md outline-none
                       focus:border-amber-400/60 transition
                       [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
              />
              <button type="button" on:click={() => clampMinutes(lateMinutes + 5)} disabled={lateMinutes >= 120}
                aria-label="Increase"
                class="w-6 h-6 rounded-md flex items-center justify-center bg-muted hover:bg-muted/80
                       text-muted-foreground hover:text-foreground transition active:scale-90 disabled:opacity-30"
              ><Plus class="w-3 h-3" /></button>
            </div>
            <button on:click={() => submitRsvp('late')} disabled={rsvpLoading}
              class="h-6 px-2.5 rounded-md bg-amber-500/15 border border-amber-500/25
                     text-xs font-medium text-amber-400 hover:bg-amber-500/25
                     transition active:scale-95 disabled:opacity-40 shrink-0"
            >{rsvpLoading ? '…' : 'Save'}</button>
          </div>
        {/if}
      </div>

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
                  <Avatar username={m.username ?? '?'} avatarUrl={m.avatar_url ?? null} size={20} />
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

      <!-- Poll results -->
      {#if poll}
        <div class="border-t border-border pt-4">
          <PollResults {poll} {eventColor} />
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  @keyframes modal-in {
    from { opacity: 0; transform: scale(0.94) translateY(8px); }
    to   { opacity: 1; transform: scale(1)    translateY(0);   }
  }
</style>