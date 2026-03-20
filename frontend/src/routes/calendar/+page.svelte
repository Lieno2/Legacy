<script lang="ts">
  import { onMount } from 'svelte';
  import { apiFetch } from '$lib/api';
  import { auth } from '$lib/stores';
  import { getDaysInMonth, getFirstDayOfMonth, isSameDay, cn } from '$lib/utils';
  import type { Event } from '$lib/types';
  import EventModal from '$lib/components/EventModal.svelte';
  import EventDetail from '$lib/components/EventDetail.svelte';
  import HappeningSoon from '$lib/components/HappeningSoon.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import { goto } from '$app/navigation';
  import { toggleMode } from 'mode-watcher';

  let isDark = true;
  onMount(() => {
    isDark = document.documentElement.classList.contains('dark');
    const obs = new MutationObserver(() => {
      isDark = document.documentElement.classList.contains('dark');
    });
    obs.observe(document.documentElement, { attributeFilter: ['class'] });
    return () => obs.disconnect();
  });
  import {
    ChevronLeft, ChevronRight, Search, Plus, Lock, Clock,
    Sun, Moon, Shield, User
  } from 'lucide-svelte';

  let events: Event[] = [];
  let loading = true;
  let today   = new Date();
  let viewYear  = today.getFullYear();
  let viewMonth = today.getMonth();
  let search = '';

  let showCreate  = false;
  let createDate  = '';
  let editingEvent: Event | null = null;
  let detailEvent: Event | null  = null;

  $: user = $auth.user;
  $: isAdmin = (user?.perms ?? 0) >= 999;

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
    let list = events.filter(e => isSameDay(new Date(e.date), day));
    if (search.trim()) {
      const q = search.trim().toLowerCase();
      list = list.filter(e => e.title.toLowerCase().includes(q));
    }
    return list;
  }

  function toLocalDatetime(d: Date): string {
    const p = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${p(d.getMonth()+1)}-${p(d.getDate())}T09:00`;
  }

  function openCreateOnDay(day: Date) { createDate = toLocalDatetime(day); showCreate = true; }

  function handleDayClick(e: MouseEvent, day: Date) {
    if ((e.target as HTMLElement).closest('[data-event]')) return;
    openCreateOnDay(day);
  }

  function goToday() { viewYear = today.getFullYear(); viewMonth = today.getMonth(); }

  const MONTHS = ['January','February','March','April','May','June',
                  'July','August','September','October','November','December'];
  const DAYS = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];

  function prevMonth() { if (viewMonth === 0) { viewMonth = 11; viewYear--; } else viewMonth--; }
  function nextMonth() { if (viewMonth === 11) { viewMonth = 0; viewYear++; } else viewMonth++; }

  function openDetail(ev: Event) { detailEvent = ev; }
  function openEdit(ev: Event)   { editingEvent = ev; detailEvent = null; }

  async function deleteEvent(id: number) {
    await apiFetch(`/api/events/${id}`, { method: 'DELETE' });
    await loadEvents();
    detailEvent = null;
  }

  function isSoon(ev: Event): boolean {
    const diff = new Date(ev.date).getTime() - Date.now();
    return diff > 0 && diff < 48 * 60 * 60 * 1000;
  }

  $: initials = user?.username?.[0]?.toUpperCase() ?? '?';

  // ── Keyboard shortcuts ────────────────────────────────────────────────────

  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

    // Close dialogs with Escape
    if (e.key === 'Escape') {
      if (showCreate || editingEvent) { showCreate = false; editingEvent = null; createDate = ''; return; }
      if (detailEvent) { detailEvent = null; return; }
    }

    // Block other shortcuts when a modal is open
    if (showCreate || editingEvent || detailEvent) return;

    switch (e.key) {
      case 'n':
        e.preventDefault();
        createDate = toLocalDatetime(today);
        showCreate = true;
        break;
      case 't':
        e.preventDefault();
        goToday();
        break;
      case 'ArrowLeft':
        e.preventDefault();
        prevMonth();
        break;
      case 'ArrowRight':
        e.preventDefault();
        nextMonth();
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="h-screen bg-background text-foreground flex flex-col overflow-hidden">
  <!-- Top bar -->
  <header class="flex items-center gap-3 px-4 py-2.5 border-b border-border shrink-0">
    <div class="flex items-center gap-0.5">
      <button on:click={prevMonth} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
        <ChevronLeft class="w-4 h-4" />
      </button>
      <h2 class="text-sm font-semibold w-32 text-center select-none">{MONTHS[viewMonth]} {viewYear}</h2>
      <button on:click={nextMonth} class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
        <ChevronRight class="w-4 h-4" />
      </button>
    </div>

    <button on:click={goToday}
      class="h-7 px-3 rounded-lg border border-border text-xs font-medium hover:bg-muted transition">
      Today
    </button>

    <div class="flex-1"></div>

    <!-- Search -->
    <div class="relative">
      <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
      <input
        bind:value={search}
        placeholder="Search events…"
        class="h-8 w-48 rounded-lg border border-border bg-muted/40 pl-8 pr-3 text-sm outline-none
               focus:border-ring focus:ring-2 focus:ring-ring/20 focus:bg-transparent transition
               placeholder:text-muted-foreground/60"
      />
    </div>

    <!-- Theme toggle -->
    <button
      on:click={toggleMode}
      title="Toggle theme (no shortcut)"
      class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground"
    >
      {#if isDark}
        <Sun class="w-4 h-4" />
      {:else}
        <Moon class="w-4 h-4" />
      {/if}
    </button>

    <!-- Admin link (only for admins) -->
    {#if isAdmin}
      <a href="/admin"
        title="Admin panel"
        class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-amber-500/10 transition text-amber-400">
        <Shield class="w-4 h-4" />
      </a>
    {/if}

    <!-- Avatar / account -->
    <a href="/account" class="rounded-full hover:ring-2 hover:ring-ring/50 transition">
      <Avatar username={user?.username ?? '?'} avatarUrl={user?.avatar_url ?? null} size={28} />
    </a>
  </header>

  <!-- Happening Soon banner -->
  <HappeningSoon {events} on:open={e => openDetail(e.detail)} />

  <!-- Day headers -->
  <div class="grid grid-cols-7 border-b border-border shrink-0">
    {#each DAYS as d, i}
      <div class={cn(
        'py-2 text-center text-[11px] font-medium tracking-wider text-muted-foreground',
        i === 0 || i === 6 ? 'opacity-50' : ''
      )}>{d}</div>
    {/each}
  </div>

  <!-- Grid -->
  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <div class="flex flex-col items-center gap-2 text-muted-foreground">
        <div class="w-5 h-5 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin"></div>
        <span class="text-sm">Loading…</span>
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-7 flex-1 overflow-hidden" style="grid-auto-rows: minmax(0, 1fr);">
      {#each calendarDays as day, i}
        {@const dayEvs = eventsOnDay(day)}
        {@const isToday = day ? isSameDay(day, today) : false}
        {@const isWeekend = i % 7 === 0 || i % 7 === 6}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          on:click={(e) => day && handleDayClick(e, day)}
          class={cn(
            'group relative border-b border-r border-border/60 flex flex-col overflow-hidden',
            !day && 'bg-muted/10',
            day && 'cursor-pointer hover:bg-muted/20 transition-colors duration-100',
            isWeekend && day && 'bg-muted/5',
            i % 7 === 0 && 'border-l border-l-border/60'
          )}
        >
          {#if day}
            <div class="flex items-center justify-between px-2 pt-1.5 pb-0.5 shrink-0">
              <span class={cn(
                'w-6 h-6 flex items-center justify-center rounded-full text-xs font-medium select-none',
                isToday
                  ? 'bg-primary text-primary-foreground font-semibold'
                  : isWeekend ? 'text-muted-foreground/60' : 'text-muted-foreground'
              )}>{day.getDate()}</span>
              <button
                data-event
                on:click|stopPropagation={() => openCreateOnDay(day)}
                class="w-5 h-5 rounded-md flex items-center justify-center text-muted-foreground
                       opacity-0 group-hover:opacity-100 hover:bg-muted hover:text-foreground
                       transition-all"
                tabindex="-1" aria-label="Add event"
              >
                <Plus class="w-3 h-3" />
              </button>
            </div>

            <div class="flex flex-col gap-px px-1 pb-1 overflow-hidden">
              {#each dayEvs.slice(0, 3) as ev}
                <button
                  data-event
                  on:click|stopPropagation={() => openDetail(ev)}
                  class="w-full text-left text-[11px] px-1.5 py-[3px] rounded-[4px] truncate
                         font-medium transition-opacity hover:opacity-80 flex items-center gap-1"
                  style="background-color:{ev.color ?? '#6366f1'}26; color:{ev.color ?? '#a5b4fc'};
                         border-left: 2px solid {ev.color ?? '#6366f1'};"
                >
                  {#if ev.private}<Lock class="w-2.5 h-2.5 shrink-0 opacity-70" />{/if}
                  {#if isSoon(ev)}<Clock class="w-2.5 h-2.5 shrink-0 opacity-70" />{/if}
                  <span class="truncate">{ev.title}</span>
                </button>
              {/each}
              {#if dayEvs.length > 3}
                <span class="text-[10px] text-muted-foreground px-1.5 leading-tight">+{dayEvs.length - 3} more</span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- Keyboard hint -->
  <div class="fixed bottom-3 right-4 hidden md:flex items-center gap-3 text-[10px] text-muted-foreground/40 select-none pointer-events-none">
    <span><kbd class="px-1 py-0.5 rounded bg-muted/40 font-mono border border-border/30">n</kbd> new</span>
    <span><kbd class="px-1 py-0.5 rounded bg-muted/40 font-mono border border-border/30">t</kbd> today</span>
    <span><kbd class="px-1 py-0.5 rounded bg-muted/40 font-mono border border-border/30">←→</kbd> month</span>
    <span><kbd class="px-1 py-0.5 rounded bg-muted/40 font-mono border border-border/30">esc</kbd> close</span>
  </div>
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