<!--
  PollAnswerModal — shown after the user clicks Going / Late, before the RSVP
  is confirmed, when the event has a poll the user hasn't answered yet.

  Events emitted:
    confirm({ choiceId })  — user picked an answer, proceed with RSVP
    skip                   — user dismissed without answering
-->
<script lang="ts">
  import { HelpCircle, X, Loader2 } from 'lucide-svelte';
  import type { Poll } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  export let poll: Poll;
  export let eventColor = '#6366f1';
  export let saving = false;

  const dispatch = createEventDispatcher<{
    confirm: { choiceId: number };
    skip: void;
  }>();

  let selected: number | null = poll.my_choice_id ?? null;

  function handleBackdropKey(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('skip');
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="fixed inset-0 z-[60] flex items-center justify-center p-4"
  style="background:rgba(0,0,0,0.65); backdrop-filter:blur(4px);"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  on:click|self={() => dispatch('skip')}
  on:keydown={handleBackdropKey}
>
  <div
    class="w-full max-w-sm bg-card border border-border rounded-2xl shadow-2xl shadow-black/50 overflow-hidden"
    style="animation: modal-in 0.18s cubic-bezier(0.34,1.56,0.64,1) both;"
  >
    <div class="h-0.5 w-full" style="background: linear-gradient(90deg, {eventColor}, {eventColor}55);"></div>

    <!-- Header -->
    <div class="flex items-center gap-2.5 px-5 py-4 border-b border-border">
      <div class="w-6 h-6 rounded-md flex items-center justify-center shrink-0"
        style="background:{eventColor}20; border:1px solid {eventColor}40;">
        <HelpCircle class="w-3.5 h-3.5" style="color:{eventColor};" />
      </div>
      <h2 class="text-sm font-semibold flex-1 leading-tight">Quick question</h2>
      <button on:click={() => dispatch('skip')} aria-label="Skip"
        class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Body -->
    <div class="flex flex-col gap-4 p-5">
      <p class="text-sm font-medium leading-snug">{poll.question}</p>

      <div class="flex flex-col gap-2">
        {#each poll.choices as choice}
          <button
            type="button"
            on:click={() => (selected = choice.id)}
            class="flex items-center gap-3 px-4 py-3 rounded-xl border text-sm font-medium transition-all text-left
                   {selected === choice.id
                     ? 'border-[--c]/50 bg-[--c]/10 text-foreground'
                     : 'border-border text-muted-foreground hover:bg-muted hover:text-foreground'}"
            style="--c:{eventColor};"
          >
            <span
              class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center transition-colors"
              style="border-color:{selected === choice.id ? eventColor : 'var(--border)'};"
            >
              {#if selected === choice.id}
                <span class="w-2 h-2 rounded-full" style="background:{eventColor};"></span>
              {/if}
            </span>
            {choice.label}
          </button>
        {/each}
      </div>

      <!-- Actions -->
      <div class="flex gap-2 pt-1">
        <button type="button" on:click={() => dispatch('skip')}
          class="flex-1 h-9 rounded-xl border border-border text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition">
          Skip
        </button>
        <button
          type="button"
          on:click={() => selected !== null && dispatch('confirm', { choiceId: selected })}
          disabled={selected === null || saving}
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
