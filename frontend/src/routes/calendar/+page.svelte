<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { getDaysInMonth, getFirstDayOfMonth, isSameDay, cn } from '$lib/utils';
  import type { Event } from '$lib/types';
  import EventModal from '$lib/components/EventModal.svelte';
  import EventDetail from '$lib/components/EventDetail.svelte';
  import { goto } from '$app/navigation';
  import { ChevronLeft, ChevronRight, Search, User } from 'lucide-svelte';

  let events: Event[] = [];
  let loading = true;
  let today   = new Date();
  let viewYear  = today.getFullYear();
  let viewMonth = today.getMonth();
  let search = '';

  let showCreate    = false;
  let createDate    = '';
  let editingEvent: Event | null = null;
  let detailEvent:  Event | null = null;

  $: user = $auth.user;

  onMount(async () => {
    if (!localStorage.getItem('access_token')) { goto('/login'); return; }
    await loadEvents();
  });

  async function loadEvents() {
    loading = true;
    try { events = await apiFetch<Event[]>('/api/events'); }
    finally { loading = false; }
  }

  $: daysInMonth  = getDaysInMonth(viewYear, viewMonth);
  $: firstDay     = getFirstDayOfMonth(viewYear, viewMonth);
  $: calendarDays = buildCalendar(viewYear, viewMonth, firstDay, daysInMonth);

  function buildCalendar(year: number, month: number, first: number, days: number) {
    const cells: (Date | null)[] = Array(first).fill(null);
    for (let d = 1; d <= days; d++) cells.push(new Date(year, month, d));
    while (cells.length % 7 !== 0) cells.push(null);
    return cells;
  }

  function eventsOnDay(day: Date | null): Event[] {
    if (!day) return [];
    let filtered = events.filter(e => isSameDay(new Date(e.date), day));
    if (search.trim()) {
      const q = search.trim().toLowerCase();
      filtered = filtered.filter(e => e.title.toLowerCase().includes(q));
    }
    return filtered;
  }

  function toLocalDatetimeString(d: Date): string {
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T09:00`;
  }

  function openCreateOnDay(day: Date) {
    createDate = toLocalDatetimeString(day);
    showCreate = true;
  }

  function handleDayClick(e: MouseEvent, day: Date) {
    if ((e.target as HTMLElement).closest('[data-event]')) return;
    openCreateOnDay(day);
  }

  function goToday() {
    viewYear  = today.getFullYear();
    viewMonth = today.getMonth();
  }

  const MONTHS = ['January','February','March','April','May','June',
                  'July','August','September','October','November','December'];

  function prevMonth() {
    if (viewMonth === 0) { viewMonth = 11; viewYear--; }
    else viewMonth--;
  }
  function nextMonth() {
    if (viewMonth === 11) { viewMonth = 0; viewYear++; }
    else viewMonth++;
  }

  function openDetail(event: Event) { detailEvent = event; }
  function openEdit(event: Event)   { editingEvent = event; detailEvent = null; }

  async function deleteEvent(id: number) {
    await apiFetch(`/api/events/${id}`, { method: 'DELETE' });
    await loadEvents();
    detailEvent = null;
  }

  // Is event happening soon (within 2 hours)?
  function isSoon(event: Event): boolean {
    const diff = new Date(event.date).getTime() - Date.now();
    return diff > 0 && diff < 2 * 60 * 60 * 1000;
  }
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col">
  <!-- Header -->
  <header class="px-6 py-3 flex items-center gap-4">
    <!-- Month nav -->
    <div class="flex items-center gap-1">
      <h2 class="text-base font-semibold w-36">{MONTHS[viewMonth]} {viewYear}</h2>
      <button on:click={prevMonth} class="p-1 rounded hover:bg-muted transition">
        <ChevronLeft class="w-4 h-4" />
      </button>
      <button
        on:click={goToday}
        class="px-3 h-7 rounded-md border border-border text-xs font-medium hover:bg-muted transition"
      >Today</button>
      <button on:click={nextMonth} class="p-1 rounded hover:bg-muted transition">
        <ChevronRight class="w-4 h-4" />
      </button>
    </div>

    <div class="flex-1"></div>

    <!-- Search -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
      <input
        bind:value={search}
        placeholder="Search events..."
        class="h-8 w-44 rounded-md border border-border bg-transparent pl-8 pr-3 text-sm outline-none
               focus:border-ring focus:ring-2 focus:ring-ring/30 transition placeholder:text-muted-foreground"
      />
    </div>

    <!-- Avatar -->
    <a href="/account" class="w-7 h-7 rounded-full bg-muted flex items-center justify-center text-xs font-semibold hover:ring-2 hover:ring-ring transition">
      {user?.username?.[0]?.toUpperCase() ?? '?'}
    </a>
  </header>

  <!-- Weekday headers -->
  <div class="grid grid-cols-7 border-b border-border/50">
    {#each ['SUN','MON','TUE','WED','THU','FRI','SAT'] as d}
      <div class="text-[11px] font-medium text-muted-foreground text-center py-2 tracking-wide">{d}</div>
    {/each}
  </div>

  <!-- Calendar grid -->
  {#if loading}
    <div class="flex-1 flex items-center justify-center text-muted-foreground text-sm">Loading…</div>
  {:else}
    <div class="grid grid-cols-7 flex-1" style="grid-auto-rows: minmax(100px, 1fr);">
      {#each calendarDays as day, i}
        {@const dayEvents = eventsOnDay(day)}
        {@const isToday = day ? isSameDay(day, today) : false}
        {@const isCurrentMonth = day !== null}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          on:click={(e) => day && handleDayClick(e, day)}
          class={cn(
            'group relative border-b border-r border-border/30 p-1 flex flex-col gap-0.5 overflow-hidden',
            !day && 'bg-muted/10',
            day && 'cursor-pointer hover:bg-muted/20 transition-colors',
            i % 7 === 0 && 'border-l'
          )}
        >
          {#if day}
            <!-- Day number top-left -->
            <div class="flex items-center justify-between px-0.5 mb-0.5">
              <span class={cn(
                'text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full',
                isToday
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground'
              )}>{day.getDate()}</span>

              <!-- Plus on hover -->
              <button
                data-event
                on:click|stopPropagation={() => openCreateOnDay(day)}
                class="w-5 h-5 flex items-center justify-center rounded opacity-0 group-hover:opacity-100
                       transition-opacity hover:bg-muted text-muted-foreground"
                tabindex="-1"
                aria-label="Add event"
              >+</button>
            </div>

            {#each dayEvents.slice(0, 3) as event}
              <button
                data-event
                on:click|stopPropagation={() => openDetail(event)}
                class="w-full text-left text-[11px] px-1.5 py-[3px] rounded-sm truncate flex items-center gap-1 font-medium"
                style="background-color: {event.color ?? '#6366f1'}22; color: {event.color ?? '#6366f1'}; border-left: 2.5px solid {event.color ?? '#6366f1'};"
              >
                {#if event.private}<span class="shrink-0">🔒</span>{/if}
                {#if isSoon(event)}<span class="shrink-0">⏰</span>{/if}
                <span class="truncate">{event.title}</span>
              </button>
            {/each}
            {#if dayEvents.length > 3}
              <span class="text-[10px] text-muted-foreground pl-1.5">+{dayEvents.length - 3} more</span>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreate || editingEvent}
  <EventModal
    event={editingEvent}
    defaultDate={createDate}
    on:saved={() => { showCreate = false; editingEvent = null; createDate = ''; loadEvents(); }}
    on:cancel={() => { showCreate = false; editingEvent = null; createDate = ''; }}
  />
{/if}

{#if detailEvent}
  <EventDetail
    event={detailEvent}
    currentUserId={user?.id ?? null}
    on:edit={() => openEdit(detailEvent!)}
    on:delete={() => deleteEvent(detailEvent!.id)}
    on:close={() => (detailEvent = null)}
  />
{/if}
