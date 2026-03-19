<script lang="ts">
  import { BarChart2, UserCheck, CheckCircle2, Clock, XCircle, Loader2 } from 'lucide-svelte';

  interface MonthStat  { month: string; count: number }
  interface ActiveUser { username: string; rsvp_count: number }
  interface RsvpBreak  { going: number; late: number; not_going: number; invited: number }

  export interface StatsData {
    events_per_month: MonthStat[];
    most_active_users: ActiveUser[];
    rsvp_breakdown: RsvpBreak;
  }

  export let stats: StatsData | null = null;
  export let statsLoading = false;

  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{ retry: void }>();

  function barHeight(count: number, max: number) {
    return max === 0 ? 0 : Math.round((count / max) * 100);
  }

  $: maxMonth  = stats ? Math.max(...stats.events_per_month.map(m => m.count), 1) : 1;
  $: rsvpTotal = stats
    ? stats.rsvp_breakdown.going + stats.rsvp_breakdown.late + stats.rsvp_breakdown.not_going + stats.rsvp_breakdown.invited
    : 0;

  function monthLabel(ym: string) {
    const [y, m] = ym.split('-');
    return new Date(+y, +m - 1).toLocaleDateString('en-US', { month: 'short' });
  }
</script>

{#if statsLoading}
  <div class="flex items-center justify-center gap-2 py-16 text-muted-foreground">
    <Loader2 class="w-5 h-5 animate-spin" /> Loading stats…
  </div>
{:else if stats}
  <div class="flex flex-col gap-4">
    <div class="bg-card border border-border rounded-2xl p-5">
      <h2 class="font-semibold text-sm mb-4">Events per Month <span class="text-muted-foreground font-normal">(last 12 months)</span></h2>
      {#if stats.events_per_month.length === 0}
        <p class="text-sm text-muted-foreground">No events in the last 12 months.</p>
      {:else}
        <div class="flex items-end gap-1.5 h-36">
          {#each stats.events_per_month as m}
            <div class="flex-1 flex flex-col items-center gap-1 group">
              <span class="text-[10px] text-muted-foreground opacity-0 group-hover:opacity-100 transition tabular-nums">{m.count}</span>
              <div class="w-full rounded-t-md bg-primary/80 hover:bg-primary transition-all duration-300"
                style="height:{barHeight(m.count, maxMonth)}%;"></div>
              <span class="text-[10px] text-muted-foreground">{monthLabel(m.month)}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <div class="bg-card border border-border rounded-2xl p-5">
        <div class="flex items-center gap-2 mb-4">
          <UserCheck class="w-4 h-4 text-muted-foreground" />
          <h2 class="font-semibold text-sm">Most Active Users</h2>
        </div>
        {#if stats.most_active_users.length === 0}
          <p class="text-sm text-muted-foreground">No RSVP data yet.</p>
        {:else}
          <div class="flex flex-col gap-3">
            {#each stats.most_active_users as u, i}
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold text-muted-foreground/50 w-4 tabular-nums">{i + 1}</span>
                <div class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold uppercase shrink-0"
                  style="background:hsl({(u.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(u.username.charCodeAt(0)*47)%360},70%,70%);"
                >{u.username[0]}</div>
                <span class="text-sm flex-1 truncate">{u.username}</span>
                <span class="text-xs font-semibold tabular-nums text-primary">{u.rsvp_count}</span>
              </div>
              {#if i < stats.most_active_users.length - 1}
                <div class="h-px bg-border/40"></div>
              {/if}
            {/each}
          </div>
        {/if}
      </div>

      <div class="bg-card border border-border rounded-2xl p-5">
        <div class="flex items-center gap-2 mb-4">
          <BarChart2 class="w-4 h-4 text-muted-foreground" />
          <h2 class="font-semibold text-sm">RSVP Breakdown</h2>
        </div>
        {#if rsvpTotal === 0}
          <p class="text-sm text-muted-foreground">No RSVPs yet.</p>
        {:else}
          {@const b = stats.rsvp_breakdown}
          <div class="flex flex-col gap-3">
            {#each [
              { label: 'Going',     value: b.going,     color: '#22c55e', icon: CheckCircle2 },
              { label: 'Late',      value: b.late,      color: '#eab308', icon: Clock        },
              { label: 'Not going', value: b.not_going, color: '#ef4444', icon: XCircle      },
              { label: 'Invited',   value: b.invited,   color: '#6366f1', icon: UserCheck    },
            ] as row}
              <div class="flex flex-col gap-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="flex items-center gap-1.5 text-muted-foreground">
                    <svelte:component this={row.icon} class="w-3 h-3" style="color:{row.color};" />
                    {row.label}
                  </span>
                  <span class="font-medium tabular-nums">{row.value}</span>
                </div>
                <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                  <div class="h-full rounded-full transition-all duration-500"
                    style="width:{rsvpTotal ? (row.value / rsvpTotal * 100).toFixed(1) : 0}%; background:{row.color};"
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="py-16 text-center text-sm text-muted-foreground">
    Failed to load stats.
    <button on:click={() => dispatch('retry')} class="underline hover:text-foreground transition">Retry</button>
  </div>
{/if}
