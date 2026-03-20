<!--
  PollAnswerModal — shown before RSVP.
  Renders the right UI based on poll_type.
-->
<script lang="ts">
  import { HelpCircle, X, Loader2, Star } from 'lucide-svelte';
  import type { Poll } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  export let poll: Poll;
  export let eventColor = '#6366f1';
  export let saving = false;

  const dispatch = createEventDispatcher<{
    confirm: { choiceIds?: number[]; textAnswer?: string; rating?: number };
    skip: void;
  }>();

  // ── Choice / yesno / date state ───────────────────────────────────────────
  let selected = new Set<number>(poll.my_choice_ids);

  function toggle(id: number) {
    if (poll.allow_multiple) {
      const s = new Set(selected);
      s.has(id) ? s.delete(id) : s.add(id);
      selected = s;
    } else {
      selected = new Set([id]);
    }
  }

  // ── Text state ────────────────────────────────────────────────────────────
  let textAnswer = poll.my_text_answer ?? '';

  // ── Rating state ──────────────────────────────────────────────────────────
  let rating     = poll.my_rating ?? 0;
  let hoverRating = 0;

  // ── Can confirm ───────────────────────────────────────────────────────────
  $: canConfirm = (() => {
    switch (poll.poll_type) {
      case 'choice':
      case 'yesno':
      case 'date':    return selected.size > 0;
      case 'text':    return textAnswer.trim().length > 0;
      case 'rating':  return rating > 0;
      default:        return false;
    }
  })();

  function confirm() {
    if (!canConfirm) return;
    switch (poll.poll_type) {
      case 'choice':
      case 'yesno':
      case 'date':
        dispatch('confirm', { choiceIds: [...selected] });
        break;
      case 'text':
        dispatch('confirm', { textAnswer: textAnswer.trim() });
        break;
      case 'rating':
        dispatch('confirm', { rating });
        break;
    }
  }

  function handleBackdropKey(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('skip');
  }

  const TYPE_SUBTITLE: Record<string, string> = {
    choice: 'Pick an option',
    yesno:  'Yes or no?',
    date:   'Which date works for you?',
    text:   'Type your answer',
    rating: 'Rate from 1 to 5',
  };
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
        class="fixed inset-0 z-[60] flex items-center justify-center p-4"
        style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);"
        role="dialog" aria-modal="true" tabindex="-1"
        on:click|self={() => dispatch('skip')}
        on:keydown={handleBackdropKey}
>
  <div
          class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl shadow-black/50 overflow-hidden"
          style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;"
  >
    <div class="h-0.5 w-full" style="background: linear-gradient(90deg, {eventColor}, {eventColor}55);"></div>

    <div class="flex items-center gap-2.5 px-5 py-4 border-b border-border">
      <div class="w-6 h-6 rounded-md flex items-center justify-center shrink-0"
           style="background:{eventColor}20; border:1px solid {eventColor}40;">
        <HelpCircle class="w-3.5 h-3.5" style="color:{eventColor};" />
      </div>
      <div class="flex-1 min-w-0">
        <h2 class="text-sm font-semibold leading-tight">Quick question</h2>
        <p class="text-[11px] text-muted-foreground mt-0.5">{TYPE_SUBTITLE[poll.poll_type] ?? 'Answer before RSVPing'}</p>
      </div>
      <button on:click={() => dispatch('skip')} aria-label="Skip"
              class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex flex-col gap-4 p-5">
      <p class="text-sm font-medium leading-snug">{poll.question}</p>

      <!-- ── CHOICE / YESNO / DATE ── -->
      {#if poll.poll_type === 'choice' || poll.poll_type === 'yesno' || poll.poll_type === 'date'}
        <div class="flex flex-col gap-2">
          {#each poll.choices as choice}
            {@const active = selected.has(choice.id)}
            <button
                    type="button"
                    on:click={() => toggle(choice.id)}
                    class="flex items-center gap-3 px-4 py-3 rounded-xl border text-sm font-medium transition-all text-left
                     {active ? 'text-foreground' : 'border-border text-muted-foreground hover:bg-muted hover:text-foreground'}"
                    style={active ? `border-color:${eventColor}80; background:${eventColor}15;` : ''}
            >
              {#if poll.allow_multiple}
                <span class="w-4 h-4 rounded-md border-2 shrink-0 flex items-center justify-center transition-colors"
                      style="border-color:{active ? eventColor : 'var(--border)'}; background:{active ? eventColor : 'transparent'};">
                  {#if active}
                    <svg class="w-2.5 h-2.5 text-white" viewBox="0 0 10 10" fill="none">
                      <path d="M1.5 5l2.5 2.5 4.5-4.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  {/if}
                </span>
              {:else}
                <span class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center transition-colors"
                      style="border-color:{active ? eventColor : 'var(--border)'};">
                  {#if active}<span class="w-2 h-2 rounded-full" style="background:{eventColor};"></span>{/if}
                </span>
              {/if}
              {choice.label}
            </button>
          {/each}
        </div>

        <!-- ── TEXT ── -->
      {:else if poll.poll_type === 'text'}
        <textarea
                bind:value={textAnswer}
                rows="3"
                placeholder="Type your answer…"
                class="w-full rounded-xl border border-input bg-muted/20 px-3 py-2 text-sm outline-none
                 focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                 placeholder:text-muted-foreground/50 resize-none transition"
        ></textarea>

        <!-- ── RATING ── -->
      {:else if poll.poll_type === 'rating'}
        <div class="flex flex-col items-center gap-3">
          <div class="flex gap-2">
            {#each [1,2,3,4,5] as star}
              <button
                      type="button"
                      on:click={() => { rating = star; }}
                      on:mouseenter={() => { hoverRating = star; }}
                      on:mouseleave={() => { hoverRating = 0; }}
                      aria-label="{star} star{star !== 1 ? 's' : ''}"
                      class="transition-transform hover:scale-110 active:scale-95"
              >
                <Star
                        class="w-8 h-8 transition-colors"
                        style="color:{star <= (hoverRating || rating) ? '#f59e0b' : 'var(--muted-foreground)'}; fill:{star <= (hoverRating || rating) ? '#f59e0b' : 'transparent'};"
                />
              </button>
            {/each}
          </div>
          {#if rating > 0}
            <p class="text-sm text-muted-foreground">
              {['', 'Poor', 'Fair', 'Good', 'Great', 'Excellent'][rating]}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-2 pt-1">
        <button type="button" on:click={() => dispatch('skip')}
                class="flex-1 h-9 rounded-xl border border-border text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition">
          Skip
        </button>
        <button
                type="button"
                on:click={confirm}
                disabled={!canConfirm || saving}
                class="flex-1 h-9 rounded-xl text-sm font-semibold text-white transition active:scale-[0.97] disabled:opacity-50 flex items-center justify-center gap-2"
                style="background:{eventColor};"
        >
          {#if saving}
            <Loader2 class="w-4 h-4 animate-spin" /> Saving…
          {:else}
            Confirm
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes modal-in {
    from { opacity: 0; transform: scale(0.94) translateY(8px); }
    to   { opacity: 1; transform: scale(1)    translateY(0);   }
  }
</style>