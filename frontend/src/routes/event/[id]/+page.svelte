<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { formatDate, formatTime } from '$lib/utils';
  import { CalendarDays, Clock, MapPin, Users, Lock, Loader2, AlertCircle } from 'lucide-svelte';

  const BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3001';

  interface PublicEvent {
    id: number;
    title: string;
    description: string | null;
    date: string;
    location: string | null;
    color: string | null;
    creator_name: string | null;
    rsvp_counts: { going: number; late: number; not_going: number };
  }

  let event: PublicEvent | null = null;
  let loading = true;
  let error = '';

  onMount(async () => {
    const token = $page.params.id;
    try {
      const res = await fetch(`${BASE}/api/events/public/${token}`);
      if (res.status === 404 || res.status === 403) {
        error = 'This event link is invalid or the event is private.';
      } else if (!res.ok) {
        error = 'Failed to load event.';
      } else {
        event = await res.json();
      }
    } catch {
      error = 'Could not connect to the server.';
    } finally {
      loading = false;
    }
  });

  $: color = event?.color ?? '#6366f1';
  $: totalRsvps = event ? event.rsvp_counts.going + event.rsvp_counts.late + event.rsvp_counts.not_going : 0;
</script>

<svelte:head>
  <title>{event ? event.title : 'Event'} — Legacy Calendar</title>
</svelte:head>

<div class="min-h-screen bg-background text-foreground flex flex-col items-center justify-center p-4">
  <div class="w-full max-w-md flex flex-col gap-5">

    <!-- Brand -->
    <div class="flex items-center justify-center gap-2 text-muted-foreground">
      <CalendarDays class="w-4 h-4" />
      <span class="text-sm font-medium">Legacy Calendar</span>
    </div>

    {#if loading}
      <div class="flex items-center justify-center gap-2 py-16 text-muted-foreground">
        <Loader2 class="w-5 h-5 animate-spin" />
        <span class="text-sm">Loading event…</span>
      </div>

    {:else if error}
      <div class="bg-card border border-border rounded-2xl p-8 flex flex-col items-center gap-3 text-center">
        <div class="w-10 h-10 rounded-xl bg-red-500/10 flex items-center justify-center">
          <AlertCircle class="w-5 h-5 text-red-400" />
        </div>
        <p class="text-sm text-muted-foreground">{error}</p>
        <a href="/login" class="text-sm text-primary underline underline-offset-3 hover:no-underline">
          Sign in to view your calendar
        </a>
      </div>

    {:else if event}
      <div class="bg-card border border-border rounded-2xl overflow-hidden shadow-xl shadow-black/20">
        <!-- Color banner -->
        <div class="h-1 w-full" style="background:{color};"></div>

        <div class="p-6 flex flex-col gap-5">
          <!-- Header -->
          <div>
            <h1 class="text-xl font-semibold leading-tight">{event.title}</h1>
            <p class="text-sm text-muted-foreground mt-1">by {event.creator_name ?? 'Unknown'}</p>
          </div>

          <!-- Meta -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center gap-2.5 text-sm text-muted-foreground">
              <Clock class="w-4 h-4 shrink-0" style="color:{color};" />
              <span>{formatDate(event.date)} · {formatTime(event.date)}</span>
            </div>
            {#if event.location}
              <div class="flex items-center gap-2.5 text-sm text-muted-foreground">
                <MapPin class="w-4 h-4 shrink-0" style="color:{color};" />
                <span class="flex-1">{event.location}</span>
                <a
                  href={event.location.startsWith('http') ? event.location : `https://maps.google.com/?q=${encodeURIComponent(event.location)}`}
                  target="_blank" rel="noopener noreferrer"
                  class="text-xs underline underline-offset-3 hover:no-underline shrink-0"
                  style="color:{color};"
                >Open in Maps</a>
              </div>
            {/if}
          </div>

          <!-- Description -->
          {#if event.description}
            <p class="text-sm text-muted-foreground leading-relaxed border-t border-border pt-4">
              {event.description}
            </p>
          {/if}

          <!-- RSVP summary -->
          {#if totalRsvps > 0}
            <div class="border-t border-border pt-4 flex flex-col gap-2">
              <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                <Users class="w-3.5 h-3.5" />
                {totalRsvps} response{totalRsvps !== 1 ? 's' : ''}
              </div>
              <div class="flex gap-3">
                {#if event.rsvp_counts.going > 0}
                  <span class="text-sm font-medium text-emerald-400">✓ {event.rsvp_counts.going} going</span>
                {/if}
                {#if event.rsvp_counts.late > 0}
                  <span class="text-sm font-medium text-amber-400">⏱ {event.rsvp_counts.late} late</span>
                {/if}
                {#if event.rsvp_counts.not_going > 0}
                  <span class="text-sm font-medium text-red-400">✗ {event.rsvp_counts.not_going} not going</span>
                {/if}
              </div>
            </div>
          {/if}

          <!-- CTA -->
          <div class="border-t border-border pt-4 flex flex-col items-center gap-2">
            <p class="text-xs text-muted-foreground">Want to RSVP or see full details?</p>
            <a
              href="/login"
              class="h-9 px-6 rounded-xl text-sm font-semibold text-white transition hover:opacity-90 active:scale-95"
              style="background:{color};"
            >Sign in to respond</a>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>