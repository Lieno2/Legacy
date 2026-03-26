<!--
  PollEditor — inside EventModal.
  A standard poll form (question + type + choices).
  At the top there is an optional "Use a template" dropdown (admin-created templates only).
  When a template is selected the manual type/choices section is hidden
  because the template already defines everything; the user just needs to
  confirm (or clear the template to go back to manual).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Trash2, HelpCircle, ChevronDown, Star, X } from 'lucide-svelte';
  import { apiFetch } from '$lib/api';
  import type { PollType, PollTemplate } from '$lib/types';

  export let enabled        = false;
  export let question       = '';
  export let pollType: PollType = 'choice';
  export let choices: string[]  = ['', ''];
  export let allowMultiple  = false;

  const TYPE_LABELS: Record<PollType, { label: string; desc: string }> = {
    choice:  { label: 'Multiple choice', desc: 'Users pick from a list of options' },
    yesno:   { label: 'Yes / No',        desc: 'Simple binary question' },
    text:    { label: 'Free text',       desc: 'Users type a free-form answer' },
    rating:  { label: 'Rating',          desc: 'Users rate from 1 to 5 stars' },
    date:    { label: 'Date slots',      desc: 'Users pick from date/time options' },
  };

  // ── Templates (admin-created / global only) ───────────────────────────────
  let templates: PollTemplate[] = [];
  let templatesLoading = false;
  let selectedTemplate: PollTemplate | null = null;
  let templateDropdownOpen = false;

  onMount(async () => {
    if (enabled) await loadTemplates();
  });

  $: if (enabled && templates.length === 0 && !templatesLoading) {
    loadTemplates();
  }

  async function loadTemplates() {
    templatesLoading = true;
    try {
      const all = await apiFetch<PollTemplate[]>('/api/polls/templates');
      // Only show global (admin-created) templates
      templates = all.filter(t => t.global);
    } catch {
      templates = [];
    } finally {
      templatesLoading = false;
    }
  }

  function selectTemplate(t: PollTemplate) {
    selectedTemplate = t;
    // Pre-fill the exported props so the parent EventModal gets the values
    pollType      = t.poll_type;
    question      = t.question ?? '';
    allowMultiple = t.allow_multiple;
    choices       = t.choices ? t.choices.map(c => c.label) : ['', ''];
    templateDropdownOpen = false;
  }

  function clearTemplate() {
    selectedTemplate = null;
    // Reset to sensible defaults so the user can configure manually
    pollType      = 'choice';
    question      = '';
    allowMultiple = false;
    choices       = ['', ''];
  }

  // ── Choices helpers ───────────────────────────────────────────────────────
  function needsChoices(t: PollType) { return t === 'choice' || t === 'date'; }

  function addChoice() {
    if (choices.length < 10) choices = [...choices, ''];
  }
  function removeChoice(i: number) {
    if (choices.length > 2) choices = choices.filter((_, idx) => idx !== i);
  }
  function updateChoice(i: number, val: string) {
    choices = choices.map((c, idx) => (idx === i ? val : c));
  }

  function onTypeChange(t: PollType) {
    pollType = t;
    if (needsChoices(t) && choices.filter(c => c.trim()).length < 2) {
      choices = ['', ''];
    }
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

      <!-- ── Template dropdown ─────────────────────────────────────────── -->
      {#if templates.length > 0 || templatesLoading}
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
            Use a template <span class="normal-case font-normal text-muted-foreground/50">(optional)</span>
          </label>

          {#if selectedTemplate}
            <!-- Selected state: show chip + clear button -->
            <div class="flex items-center justify-between px-3 h-9 rounded-lg border border-primary/40
                        bg-primary/10 text-sm text-primary">
              <span class="font-medium">{selectedTemplate.name}</span>
              <button
                type="button"
                on:click={clearTemplate}
                class="ml-2 w-5 h-5 rounded-md flex items-center justify-center
                       hover:bg-primary/20 transition text-primary/70 hover:text-primary"
                aria-label="Clear template"
              >
                <X class="w-3.5 h-3.5" />
              </button>
            </div>
          {:else}
            <!-- Closed / open dropdown -->
            <div class="relative">
              <button
                type="button"
                on:click={() => (templateDropdownOpen = !templateDropdownOpen)}
                class="flex items-center justify-between w-full px-3 h-9 rounded-lg border border-input
                       bg-muted/20 text-sm text-muted-foreground hover:text-foreground hover:bg-muted/40
                       transition outline-none focus:border-ring focus:ring-2 focus:ring-ring/20"
              >
                <span>{templatesLoading ? 'Loading templates…' : 'Select a template…'}</span>
                <ChevronDown class="w-3.5 h-3.5 transition-transform {templateDropdownOpen ? 'rotate-180' : ''}" />
              </button>

              {#if templateDropdownOpen && !templatesLoading}
                <div class="absolute left-0 top-10 z-20 w-full bg-card border border-border
                            rounded-xl shadow-xl overflow-hidden">
                  {#each templates as t}
                    <button
                      type="button"
                      on:click={() => selectTemplate(t)}
                      class="flex items-center justify-between w-full px-3 py-2.5 text-sm
                             hover:bg-muted transition text-left"
                    >
                      <span>{t.name}</span>
                      <span class="text-[11px] text-muted-foreground">{TYPE_LABELS[t.poll_type].label}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      <!-- ── Manual poll config (hidden when a template is active) ──────── -->
      {#if !selectedTemplate}

        <!-- Poll type selector -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Type</label>
          <div class="grid grid-cols-3 gap-1.5 sm:grid-cols-5">
            {#each Object.entries(TYPE_LABELS) as [t, meta]}
              <button
                type="button"
                on:click={() => onTypeChange(t as PollType)}
                title={meta.desc}
                class="h-8 rounded-lg border text-xs font-medium transition
                       {pollType === t
                         ? 'border-primary/50 bg-primary/10 text-primary'
                         : 'border-border text-muted-foreground hover:bg-muted hover:text-foreground'}"
              >
                {meta.label}
              </button>
            {/each}
          </div>
          <p class="text-[11px] text-muted-foreground/60">{TYPE_LABELS[pollType].desc}</p>
        </div>

        <!-- Question -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Question</label>
          <input
            bind:value={question}
            placeholder={
              pollType === 'yesno'  ? 'e.g. Are you coming?' :
              pollType === 'rating' ? 'e.g. Rate the venue idea' :
              pollType === 'text'   ? 'e.g. Any comments?' :
              'e.g. Which option do you prefer?'
            }
            class="w-full rounded-lg border border-input bg-muted/20 px-3 h-9 text-sm outline-none transition
                   focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                   placeholder:text-muted-foreground/50"
          />
        </div>

        <!-- Type-specific config -->
        {#if pollType === 'yesno'}
          <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted/30 border border-border text-xs text-muted-foreground">
            Auto-generates <span class="text-foreground font-medium">Yes</span> and <span class="text-foreground font-medium">No</span> options.
          </div>

        {:else if pollType === 'text'}
          <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted/30 border border-border text-xs text-muted-foreground">
            Each attendee can type a free-form answer.
          </div>

        {:else if pollType === 'rating'}
          <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted/30 border border-border text-xs text-muted-foreground">
            <div class="flex gap-0.5">
              {#each [1, 2, 3, 4, 5] as _}
                <Star class="w-3.5 h-3.5 text-amber-400 fill-amber-400" />
              {/each}
            </div>
            Attendees rate from 1 to 5 stars.
          </div>

        {:else if pollType === 'choice' || pollType === 'date'}
          <!-- Choices -->
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                {pollType === 'date' ? 'Date / Time options' : 'Choices'}
              </label>
              <span class="text-[11px] text-muted-foreground/50">{choices.length}/10</span>
            </div>
            {#each choices as choice, i}
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground/40 w-4 text-right shrink-0">{i + 1}</span>
                <input
                  value={choice}
                  on:input={(e) => updateChoice(i, (e.target as HTMLInputElement).value)}
                  placeholder={pollType === 'date' ? 'e.g. Saturday 5pm' : `Choice ${i + 1}`}
                  class="flex-1 rounded-lg border border-input bg-muted/20 px-3 h-8 text-sm outline-none transition
                         focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                         placeholder:text-muted-foreground/50"
                />
                <button
                  type="button"
                  on:click={() => removeChoice(i)}
                  disabled={choices.length <= 2}
                  aria-label="Remove"
                  class="w-7 h-7 rounded-lg flex items-center justify-center
                         text-muted-foreground hover:text-red-400 hover:bg-red-500/10
                         transition disabled:opacity-25 disabled:cursor-not-allowed shrink-0"
                ><Trash2 class="w-3.5 h-3.5" /></button>
              </div>
            {/each}

            {#if choices.length < 10}
              <button
                type="button"
                on:click={addChoice}
                class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition w-fit mt-0.5"
              >
                <Plus class="w-3.5 h-3.5" /> Add {pollType === 'date' ? 'date option' : 'choice'}
              </button>
            {/if}
          </div>

          <!-- Allow multiple -->
          <label class="flex items-center justify-between cursor-pointer select-none">
            <span class="text-xs text-muted-foreground">Allow multiple selections</span>
            <div class="relative">
              <input type="checkbox" bind:checked={allowMultiple} class="sr-only peer" />
              <div class="w-9 h-5 rounded-full border transition-colors duration-200
                          bg-muted border-border peer-checked:bg-primary peer-checked:border-primary"></div>
              <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow-sm
                          transition-transform duration-200 peer-checked:translate-x-4"></div>
            </div>
          </label>
        {/if}

      {:else}
        <!-- Template selected: show a read-only summary -->
        <div class="flex flex-col gap-1.5 px-3 py-2.5 rounded-lg bg-muted/30 border border-border text-xs text-muted-foreground">
          <div class="flex items-center justify-between">
            <span class="font-medium text-foreground">{question || '—'}</span>
            <span class="text-[11px] bg-muted border border-border px-1.5 py-0.5 rounded-full">
              {TYPE_LABELS[pollType].label}
            </span>
          </div>
          {#if (pollType === 'choice' || pollType === 'date') && choices.filter(c => c.trim()).length > 0}
            <ul class="mt-1 flex flex-wrap gap-1">
              {#each choices.filter(c => c.trim()) as c}
                <li class="px-2 py-0.5 rounded-full bg-muted border border-border">{c}</li>
              {/each}
            </ul>
          {:else if pollType === 'rating'}
            <div class="flex gap-0.5 mt-1">
              {#each [1, 2, 3, 4, 5] as _}
                <Star class="w-3 h-3 text-amber-400 fill-amber-400" />
              {/each}
            </div>
          {:else if pollType === 'yesno'}
            <span>Yes / No</span>
          {:else if pollType === 'text'}
            <span>Free text answer</span>
          {/if}
          <p class="mt-1 text-[11px] text-muted-foreground/50">
            Clear the template above to configure manually.
          </p>
        </div>
      {/if}

    </div>
  {/if}
</div>
