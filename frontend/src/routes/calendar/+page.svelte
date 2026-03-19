<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { getDaysInMonth, getFirstDayOfMonth, isSameDay, formatDate, cn } from '$lib/utils';
  import type { Event, EventMember } from '$lib/types';
  import EventModal from '$lib/components/EventModal.svelte';
  import EventDetail from '$lib/components/EventDetail.svelte';
  import { goto } from '$app/navigation';
  import { Calendar, User, ChevronLeft, ChevronRight, Plus } from 'lucide-svelte';

  let events: Event[] = [];
  let loading = true;
  let today   = new Date();
  let viewYear  = today.getFullYear();
  let viewMonth = today.getMonth();

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
    return events.filter(e => isSameDay(new Date(e.date), day));
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
    // only open if the click wasn't on an event button
    if ((e.target as HTMLElement).closest('[data-event]')) return;
    openCreateOnDay(day);
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
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col">
  <!-- Header -->
  <header class="border-b border-border/50 px-6 py-4 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <Calendar class="w-5 h-5 text-primary" />
      <span class="font-semibold text-base">Legacy</span>
    </div>
    <a href="/account" class="inline-flex items-center justify-center w-8 h-8 rounded-md hover:bg-muted transition">
      <User class="w-4 h-4" />
    </a>
  </header>

  <!-- Month nav -->
  <div class="flex items-center justify-between px-6 py-4">
    <button on:click={prevMonth} class="p-1 rounded hover:bg-muted transition">
      <ChevronLeft class="w-5 h-5" />
    </button>
    <h2 class="text-lg font-semibold">{MONTHS[viewMonth]} {viewYear}</h2>
    <button on:click={nextMonth} class="p-1 rounded hover:bg-muted transition">
      <ChevronRight class="w-5 h-5" />
    </button>
  </div>

  <!-- Weekday headers -->
  <div class="grid grid-cols-7 px-6 mb-1">
    {#each ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'] as d}
      <div class="text-xs text-muted-foreground text-center py-1">{d}</div>
    {/each}
  </div>

  <!-- Calendar grid -->
  {#if loading}
    <div class="flex-1 flex items-center justify-center text-muted-foreground text-sm">Loading…</div>
  {:else}
    <div class="grid grid-cols-7 px-6 flex-1 auto-rows-fr border-t border-border/50">
      {#each calendarDays as day, i}
        {@const dayEvents = eventsOnDay(day)}
        {@const isToday = day ? isSameDay(day, today) : false}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          on:click={(e) => day && handleDayClick(e, day)}
          class={cn(
            'group min-h-[80px] border-b border-r border-border/30 p-1.5 flex flex-col gap-0.5 relative',
            !day && 'bg-muted/20',
            day && 'cursor-pointer hover:bg-muted/30 transition-colors',
            i % 7 === 0 && 'border-l'
          )}
        >
          {#if day}
            <!-- plus icon, visible on group hover -->
            <button
              data-event
              on:click|stopPropagation={() => openCreateOnDay(day)}
              class="absolute top-1 right-1 w-5 h-5 flex items-center justify-center rounded
                     opacity-0 group-hover:opacity-100 transition-opacity hover:bg-muted"
              tabindex="-1"
              aria-label="Add event"
            >
              <Plus class="w-3 h-3 text-muted-foreground" />
            </button>

            <span class={cn(
              'text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full self-end',
              isToday ? 'bg-primary text-primary-foreground' : 'text-muted-foreground'
            )}>{day.getDate()}</span>

            {#each dayEvents.slice(0, 3) as event}
              <button
                data-event
                on:click|stopPropagation={() => openDetail(event)}
                class="w-full text-left text-xs px-1.5 py-0.5 rounded truncate"
                style="background: {event.color ?? 'hsl(var(--primary) / 0.15)'}; color: inherit;"
              >{event.title}</button>
            {/each}
            {#if dayEvents.length > 3}
              <span class="text-[10px] text-muted-foreground pl-1">+{dayEvents.length - 3} more</span>
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
