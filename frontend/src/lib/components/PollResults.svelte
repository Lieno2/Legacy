<!--
  PollResults — shown inside EventDetail.
  Renders differently based on poll_type.
-->
<script lang="ts">
  import { HelpCircle, CheckSquare2, Users, Star, MessageSquare, Loader2 } from 'lucide-svelte';
  import type { Poll, ChoiceVoters, TextAnswer } from '$lib/types';
  import { apiFetch } from '$lib/api';

  export let poll: Poll;
  export let eventColor = '#6366f1';

  // ── Choice / date / yesno ─────────────────────────────────────────────────
  $: total = poll.choices.reduce((s, c) => s + c.answer_count, 0);
  function pct(count: number) { return total === 0 ? 0 : Math.round((count / total) * 100); }

  let showVoters    = false;
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

  // ── Text answers ──────────────────────────────────────────────────────────
  let showTextAnswers   = false;
  let textAnswers: TextAnswer[] = [];
  let textLoading       = false;
  let textLoaded        = false;

  async function toggleTextAnswers() {
    showTextAnswers = !showTextAnswers;
    if (showTextAnswers && !textLoaded) {
      textLoading = true;
      try {
        textAnswers = await apiFetch<TextAnswer[]>(`/api/polls/text-answers?event_id=${poll.event_id}`);
        textLoaded  = true;
      } catch { textAnswers = []; }
      finally { textLoading = false; }
    }
  }

  // ── Rating helpers ────────────────────────────────────────────────────────
  function starFill(star: number, avg: number): 'full' | 'half' | 'empty' {
    if (avg >= star) return 'full';
    if (avg >= star - 0.5) return 'half';
    return 'empty';
  }

  // ── Type label ────────────────────────────────────────────────────────────
  const TYPE_ICON: Record<string, any> = {
    choice: HelpCircle,
    yesno:  HelpCircle,
    date:   HelpCircle,
    text:   MessageSquare,
    rating: Star,
  };
</script>

<div class="flex flex-col gap-2.5">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
      <svelte:component this={TYPE_ICON[poll.poll_type] ?? HelpCircle} class="w-3.5 h-3.5" />
      {#if poll.poll_type === 'rating'}
        Poll · {poll.rating_count} rating{poll.rating_count !== 1 ? 's' : ''}
      {:else if poll.poll_type === 'text'}
        Poll · {poll.my_text_answer ? '1 answer' : 'Open question'}
      {:else}
        Poll · {total} response{total !== 1 ? 's' : ''}
        {#if poll.allow_multiple}
          <span class="normal-case font-normal text-muted-foreground/50">(multi)</span>
        {/if}
      {/if}
    </div>

    {#if poll.poll_type === 'choice' || poll.poll_type === 'yesno' || poll.poll_type === 'date'}
      {#if total > 0}
        <button type="button" on:click={toggleVoters}
                class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition">
          <Users class="w-3 h-3" />
          {showVoters ? 'Hide' : 'Show'} votes
        </button>
      {/if}
    {:else if poll.poll_type === 'text'}
      <button type="button" on:click={toggleTextAnswers}
              class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition">
        <MessageSquare class="w-3 h-3" />
        {showTextAnswers ? 'Hide' : 'Show'} answers
      </button>
    {/if}
  </div>

  <p class="text-sm font-medium leading-snug">{poll.question}</p>

  <!-- ── CHOICE / YESNO / DATE ── -->
  {#if poll.poll_type === 'choice' || poll.poll_type === 'yesno' || poll.poll_type === 'date'}
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
            <div class="h-full rounded-full transition-all duration-500"
                 style="width:{barPct}%; background:{isMyPick ? eventColor : eventColor + '55'};"></div>
          </div>

          {#if showVoters}
            {#if votersLoading}
              <p class="text-[11px] text-muted-foreground pl-4">Loading...</p>
            {:else}
              {@const cv = voters.find(v => v.choice_id === choice.id)}
              {#if cv && cv.voters.length > 0}
                <div class="flex flex-wrap gap-1 pl-4 pt-0.5">
                  {#each cv.voters as v}
                    <span class="inline-flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-full bg-muted border border-border text-muted-foreground">
                      <span class="w-3 h-3 rounded-full flex items-center justify-center text-[8px] font-bold uppercase shrink-0"
                            style="background:hsl({(v.username.charCodeAt(0)*47)%360},40%,25%); color:hsl({(v.username.charCodeAt(0)*47)%360},70%,70%);"
                      >{v.username[0]}</span>
                      {v.username}
                    </span>
                  {/each}
                </div>
              {/if}
            {/if}
          {/if}
        </div>
      {/each}
    </div>

    <!-- ── RATING ── -->
  {:else if poll.poll_type === 'rating'}
    <div class="flex flex-col gap-3">
      <!-- Average stars -->
      {#if poll.rating_count > 0 && poll.avg_rating !== null}
        <div class="flex items-center gap-2">
          <div class="flex gap-0.5">
            {#each [1,2,3,4,5] as star}
              {@const fill = starFill(star, poll.avg_rating ?? 0)}
              <Star
                      class="w-5 h-5 transition-colors"
                      style="color:{fill !== 'empty' ? '#f59e0b' : 'var(--muted-foreground)'}; fill:{fill === 'full' ? '#f59e0b' : 'transparent'};"
              />
            {/each}
          </div>
          <span class="text-sm font-semibold tabular-nums">{(poll.avg_rating ?? 0).toFixed(1)}</span>
          <span class="text-xs text-muted-foreground">avg · {poll.rating_count} vote{poll.rating_count !== 1 ? 's' : ''}</span>
        </div>
      {:else}
        <p class="text-xs text-muted-foreground">No ratings yet.</p>
      {/if}

      <!-- My rating -->
      {#if poll.my_rating !== null}
        <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
          <CheckSquare2 class="w-3 h-3" style="color:{eventColor};" />
          You rated {poll.my_rating}/5
        </div>
      {/if}

      <!-- Rating distribution bars -->
      {#if poll.rating_count > 0}
        <div class="flex flex-col gap-1">
          {#each [5,4,3,2,1] as star}
            <!-- We don't track per-star counts in this version, just show the avg visually -->
          {/each}
        </div>
      {/if}
    </div>

    <!-- ── TEXT ── -->
  {:else if poll.poll_type === 'text'}
    {#if poll.my_text_answer}
      <div class="flex flex-col gap-1">
        <p class="text-[11px] text-muted-foreground uppercase tracking-wider font-medium">Your answer</p>
        <div class="px-3 py-2 rounded-lg bg-muted/30 border border-border text-sm" style="border-left:2px solid {eventColor};">
          {poll.my_text_answer}
        </div>
      </div>
    {:else}
      <p class="text-xs text-muted-foreground">You haven't answered yet.</p>
    {/if}

    {#if showTextAnswers}
      <div class="flex flex-col gap-2 mt-1">
        {#if textLoading}
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Loader2 class="w-3 h-3 animate-spin" /> Loading...
          </div>
        {:else if textAnswers.length === 0}
          <p class="text-xs text-muted-foreground">No answers yet.</p>
        {:else}
          {#each textAnswers as a}
            <div class="flex flex-col gap-0.5">
              <p class="text-[11px] text-muted-foreground font-medium">{a.username}</p>
              <div class="px-3 py-1.5 rounded-lg bg-muted/30 border border-border/50 text-sm">
                {a.answer}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  {/if}
</div>