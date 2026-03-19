<!--
  PollResults — shown inside EventDetail below the attendees section.
  Displays the poll question, each choice with a progress bar and count,
  and highlights the current user's pick.
-->
<script lang="ts">
  import { HelpCircle, CheckCircle2 } from 'lucide-svelte';
  import type { Poll } from '$lib/types';

  export let poll: Poll;
  export let eventColor = '#6366f1';

  $: total = poll.choices.reduce((s, c) => s + c.answer_count, 0);

  function pct(count: number) {
    return total === 0 ? 0 : Math.round((count / total) * 100);
  }
</script>

<div class="flex flex-col gap-2.5">
  <!-- Section header -->
  <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
    <HelpCircle class="w-3.5 h-3.5" />
    Poll · {total} response{total !== 1 ? 's' : ''}
  </div>

  <p class="text-sm font-medium leading-snug">{poll.question}</p>

  <div class="flex flex-col gap-2">
    {#each poll.choices as choice}
      {@const isMyPick = poll.my_choice_id === choice.id}
      {@const barPct  = pct(choice.answer_count)}
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between text-xs">
          <span class="flex items-center gap-1.5 {isMyPick ? 'text-foreground font-medium' : 'text-muted-foreground'}">
            {#if isMyPick}
              <CheckCircle2 class="w-3 h-3 shrink-0" style="color:{eventColor};" />
            {:else}
              <span class="w-3 h-3 shrink-0"></span>
            {/if}
            {choice.label}
          </span>
          <span class="tabular-nums font-medium {isMyPick ? 'text-foreground' : 'text-muted-foreground'}">
            {choice.answer_count} <span class="text-muted-foreground/50">({barPct}%)</span>
          </span>
        </div>
        <div class="h-1.5 bg-muted rounded-full overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-500"
            style="width:{barPct}%; background:{isMyPick ? eventColor : 'color-mix(in srgb, {eventColor} 40%, transparent)'};"
          ></div>
        </div>
      </div>
    {/each}
  </div>
</div>
