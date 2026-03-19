<!--
  PollResults — shown inside EventDetail below attendees.
  Shows bar chart results + a "Show answers" toggle that reveals who voted what.
-->
<script lang="ts">
  import { HelpCircle, CheckSquare2, Users } from 'lucide-svelte';
  import type { Poll, ChoiceVoters } from '$lib/types';
  import { apiFetch } from '$lib/api';

  export let poll: Poll;
  export let eventColor = '#6366f1';

  $: total = poll.choices.reduce((s, c) => s + c.answer_count, 0);

  function pct(count: number) {
    return total === 0 ? 0 : Math.round((count / total) * 100);
  }

  // Voters breakdown
  let showVoters = false;
  let voters: ChoiceVoters[] = [];
  let votersLoading = false;
  let votersLoaded  = false;

  async function toggleVoters() {
    showVoters = !showVoters;
    if (showVoters && !votersLoaded) {
      votersLoading = true;
      try {
        voters = await apiFetch<ChoiceVoters[]>(`/api/polls/voters?event_id=${poll.event_id}`);
        votersLoaded = true;
      } catch { voters = []; }
      finally { votersLoading = false; }
    }
  }
</script>

<div class="flex flex-col gap-2.5">
  <!-- Section header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
      <HelpCircle class="w-3.5 h-3.5" />
      Poll &middot; {total} response{total !== 1 ? 's' : ''}
      {#if poll.allow_multiple}
        <span class="normal-case font-normal text-muted-foreground/50">(multi-select)</span>
      {/if}
    </div>
    {#if total > 0}
      <button
        type="button"
        on:click={toggleVoters}
        class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition"
      >
        <Users class="w-3 h-3" />
        {showVoters ? 'Hide' : 'Show'} answers
      </button>
    {/if}
  </div>

  <p class="text-sm font-medium leading-snug">{poll.question}</p>

  <!-- Results bars -->
  <div class="flex flex-col gap-2">
    {#each poll.choices as choice}
      {@const isMyPick = poll.my_choice_ids.includes(choice.id)}
      {@const barPct   = pct(choice.answer_count)}
      <div class="flex flex-col gap-1">
        <div class="flex items-center justify-between text-xs">
          <span class="flex items-center gap-1.5 {isMyPick ? 'text-foreground font-medium' : 'text-muted-foreground'}">
            {#if isMyPick}
              <CheckSquare2 class="w-3 h-3 shrink-0" style="color:{eventColor};" />
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
            style="width:{barPct}%; background:{isMyPick ? eventColor : eventColor + '55'};"
          ></div>
        </div>

        <!-- Voters list for this choice -->
        {#if showVoters}
          {#if votersLoading}
            <p class="text-[11px] text-muted-foreground pl-4">Loading...</p>
          {:else}
            {@const cv = voters.find(v => v.choice_id === choice.id)}
            {#if cv && cv.voters.length > 0}
              <div class="flex flex-wrap gap-1 pl-4 pt-0.5">
                {#each cv.voters as v}
                  <span
                    class="inline-flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-full bg-muted border border-border text-muted-foreground"
                  >
                    <span
                      class="w-3 h-3 rounded-full flex items-center justify-center text-[8px] font-bold uppercase shrink-0"
                      style="background:hsl({(v.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(v.username.charCodeAt(0)*47)%360},70%,70%);"
                    >{v.username[0]}</span>
                    {v.username}
                  </span>
                {/each}
              </div>
            {:else if cv}
              <p class="text-[11px] text-muted-foreground/50 pl-4">No votes yet</p>
            {/if}
          {/if}
        {/if}
      </div>
    {/each}
  </div>
</div>
