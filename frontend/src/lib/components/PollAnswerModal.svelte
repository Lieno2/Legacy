<!--
  PollAnswerModal — shown after the user clicks Going / Late.
  Supports single and multiple choice depending on poll.allow_multiple.

  Events:
    confirm({ choiceIds })  — user confirmed selection
    skip                    — user dismissed without answering
-->
<script lang="ts">
  import { HelpCircle, X, Loader2 } from 'lucide-svelte';
  import type { Poll } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  export let poll: Poll;
  export let eventColor = '#6366f1';
  export let saving = false;

  const dispatch = createEventDispatcher<{
    confirm: { choiceIds: number[] };
    skip: void;
  }>();

  // Multi-select: Set of selected IDs
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

  $: isSelected = (id: number) => selected.has(id);
  $: canConfirm = selected.size > 0;

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

    <div class="flex items-center gap-2.5 px-5 py-4 border-b border-border">
      <div class="w-6 h-6 rounded-md flex items-center justify-center shrink-0"
        style="background:{eventColor}20; border:1px solid {eventColor}40;">
        <HelpCircle class="w-3.5 h-3.5" style="color:{eventColor};" />
      </div>
      <div class="flex-1 min-w-0">
        <h2 class="text-sm font-semibold leading-tight">Quick question</h2>
        {#if poll.allow_multiple}
          <p class="text-[11px] text-muted-foreground mt-0.5">You can select multiple options</p>
        {/if}
      </div>
      <button on:click={() => dispatch('skip')} aria-label="Skip"
        class="w-7 h-7 rounded-lg flex items-center justify-center hover:bg-muted transition text-muted-foreground hover:text-foreground">
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex flex-col gap-4 p-5">
      <p class="text-sm font-medium leading-snug">{poll.question}</p>

      <div class="flex flex-col gap-2">
        {#each poll.choices as choice}
          {@const active = isSelected(choice.id)}
          <button
            type="button"
            on:click={() => toggle(choice.id)}
            class="flex items-center gap-3 px-4 py-3 rounded-xl border text-sm font-medium transition-all text-left
                   {active ? 'border-[--c]/50 bg-[--c]/10 text-foreground' : 'border-border text-muted-foreground hover:bg-muted hover:text-foreground'}"
            style="--c:{eventColor};"
          >
            <!-- Radio for single, checkbox square for multi -->
            {#if poll.allow_multiple}
              <span
                class="w-4 h-4 rounded-md border-2 shrink-0 flex items-center justify-center transition-colors"
                style="border-color:{active ? eventColor : 'var(--border)'}; background:{active ? eventColor : 'transparent'};"
              >
                {#if active}<svg class="w-2.5 h-2.5 text-white" viewBox="0 0 10 10" fill="none"><path d="M1.5 5l2.5 2.5 4.5-4.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>{/if}
              </span>
            {:else}
              <span
                class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center transition-colors"
                style="border-color:{active ? eventColor : 'var(--border)'};"
              >
                {#if active}<span class="w-2 h-2 rounded-full" style="background:{eventColor};"></span>{/if}
              </span>
            {/if}
            {choice.label}
          </button>
        {/each}
      </div>

      <div class="flex gap-2 pt-1">
        <button type="button" on:click={() => dispatch('skip')}
          class="flex-1 h-9 rounded-xl border border-border text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition">
          Skip
        </button>
        <button
          type="button"
          on:click={() => canConfirm && dispatch('confirm', { choiceIds: [...selected] })}
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
