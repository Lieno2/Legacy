<!--
  PollEditor — used inside EventModal to let the creator define a poll.
  Props:
    enabled  (bind) — whether the poll toggle is on
    question (bind) — the poll question string
    choices  (bind) — string[] of choice labels
-->
<script lang="ts">
  import { Plus, Trash2, HelpCircle } from 'lucide-svelte';

  export let enabled  = false;
  export let question = '';
  export let choices: string[] = ['', ''];

  function addChoice() {
    if (choices.length < 6) choices = [...choices, ''];
  }

  function removeChoice(i: number) {
    if (choices.length > 2) choices = choices.filter((_, idx) => idx !== i);
  }

  function updateChoice(i: number, val: string) {
    choices = choices.map((c, idx) => (idx === i ? val : c));
  }
</script>

<div class="flex flex-col gap-3 pt-1 border-t border-border">
  <!-- Toggle row -->
  <label class="flex items-center justify-between cursor-pointer select-none">
    <span class="flex items-center gap-2 text-sm text-muted-foreground">
      <HelpCircle class="w-3.5 h-3.5" />
      Add a poll
      <span class="text-[11px] text-muted-foreground/50">(optional)</span>
    </span>
    <div class="relative">
      <input type="checkbox" bind:checked={enabled} class="sr-only peer" />
      <div class="w-9 h-5 rounded-full border transition-colors duration-200
                  bg-muted border-border peer-checked:bg-primary peer-checked:border-primary"></div>
      <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow-sm
                  transition-transform duration-200 peer-checked:translate-x-4"></div>
    </div>
  </label>

  {#if enabled}
    <div class="flex flex-col gap-3 pl-1">
      <!-- Question -->
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Question</label>
        <input
          bind:value={question}
          placeholder="e.g. Are you eating with us?"
          class="w-full rounded-lg border border-input bg-muted/20 px-3 h-9 text-sm outline-none transition
                 focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                 placeholder:text-muted-foreground/50"
        />
      </div>

      <!-- Choices -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Choices</label>
          <span class="text-[11px] text-muted-foreground/50">{choices.length}/6</span>
        </div>
        {#each choices as choice, i}
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground/40 w-4 text-right shrink-0">{i + 1}</span>
            <input
              value={choice}
              on:input={(e) => updateChoice(i, (e.target as HTMLInputElement).value)}
              placeholder="Choice {i + 1}"
              class="flex-1 rounded-lg border border-input bg-muted/20 px-3 h-8 text-sm outline-none transition
                     focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                     placeholder:text-muted-foreground/50"
            />
            <button
              type="button"
              on:click={() => removeChoice(i)}
              disabled={choices.length <= 2}
              aria-label="Remove choice"
              class="w-7 h-7 rounded-lg flex items-center justify-center
                     text-muted-foreground hover:text-red-400 hover:bg-red-500/10
                     transition disabled:opacity-25 disabled:cursor-not-allowed shrink-0"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        {/each}

        {#if choices.length < 6}
          <button
            type="button"
            on:click={addChoice}
            class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground
                   transition w-fit mt-0.5"
          >
            <Plus class="w-3.5 h-3.5" /> Add choice
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
