<script lang="ts">
  import { Check, Timer, XCircle, Loader2, Minus, Plus, AlertCircle } from 'lucide-svelte';
  import type { RsvpStatus } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  export let myStatus: RsvpStatus | null = null;
  export let lateMinutes = 5;
  export let rsvpLoading = false;
  export let rsvpError = '';

  const dispatch = createEventDispatcher<{
    rsvp: { status: RsvpStatus; lateMinutes?: number };
  }>();

  const RSVP_BTNS: { status: RsvpStatus; label: string; icon: any; active: string }[] = [
    { status: 'going',     label: 'Going',       icon: Check,   active: 'border-emerald-500/50 bg-emerald-500/10 text-emerald-400' },
    { status: 'late',      label: 'Coming late', icon: Timer,   active: 'border-amber-500/50  bg-amber-500/10  text-amber-400'  },
    { status: 'not_going', label: 'Not going',   icon: XCircle, active: 'border-red-500/50    bg-red-500/10    text-red-400'    },
  ];

  function clampMinutes(v: number) {
    lateMinutes = Math.min(120, Math.max(1, Math.round(v)));
  }

  function onMinutesInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value);
    if (!isNaN(v)) clampMinutes(v);
  }
</script>

<div class="flex flex-col gap-2.5">
  <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Will you attend?</p>

  <div class="grid grid-cols-3 gap-1.5">
    {#each RSVP_BTNS as btn}
      <button
        on:click={() => dispatch('rsvp', { status: btn.status, lateMinutes })}
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
                 text-muted-foreground hover:text-foreground transition active:scale-90 disabled:opacity-30">
          <Minus class="w-3 h-3" />
        </button>
        <input type="number" min="1" max="120" value={lateMinutes}
          on:change={onMinutesInput} on:blur={onMinutesInput}
          class="w-10 text-center text-sm font-semibold tabular-nums text-amber-400
                 bg-transparent border border-amber-500/25 rounded-md
                 outline-none focus:border-amber-400/60 transition
                 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
        />
        <button type="button" on:click={() => clampMinutes(lateMinutes + 5)} disabled={lateMinutes >= 120}
          aria-label="Increase"
          class="w-6 h-6 rounded-md flex items-center justify-center bg-muted hover:bg-muted/80
                 text-muted-foreground hover:text-foreground transition active:scale-90 disabled:opacity-30">
          <Plus class="w-3 h-3" />
        </button>
      </div>
      <button on:click={() => dispatch('rsvp', { status: 'late', lateMinutes })} disabled={rsvpLoading}
        class="h-6 px-2.5 rounded-md bg-amber-500/15 border border-amber-500/25
               text-xs font-medium text-amber-400 hover:bg-amber-500/25
               transition active:scale-95 disabled:opacity-40 shrink-0">
        {rsvpLoading ? '…' : 'Save'}
      </button>
    </div>
  {/if}
</div>
