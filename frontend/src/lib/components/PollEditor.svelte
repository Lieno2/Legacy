<!--
  PollEditor — inside EventModal.
  Supports 5 poll types + template loading.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Trash2, HelpCircle, ChevronDown, Star, BookOpen, Save, X } from 'lucide-svelte';
  import { apiFetch } from '$lib/api';
  import type { PollType, PollTemplate } from '$lib/types';

  export let enabled        = false;
  export let question       = '';
  export let pollType: PollType = 'choice';
  export let choices: string[]  = ['', ''];
  export let allowMultiple  = false;

  // ── Built-in templates ────────────────────────────────────────────────────
  const BUILTIN_TEMPLATES: Omit<PollTemplate, 'id' | 'created_by' | 'global'>[] = [
    { name: '🍕 Food',          poll_type: 'choice', question: 'What do you want to eat?',    choices: [{label:'Pizza'},{label:'Sushi'},{label:'Burger'},{label:'Pasta'},{label:'Tacos'}], allow_multiple: false },
    { name: '🍺 Drinks',        poll_type: 'choice', question: 'What are you drinking?',       choices: [{label:'Beer'},{label:'Wine'},{label:'Cocktails'},{label:'Non-alcoholic'}],       allow_multiple: true  },
    { name: '🌿 Vibes',         poll_type: 'choice', question: 'What are we doing?',           choices: [{label:'Smoking'},{label:'Drinking'},{label:'Both'},{label:'Neither'}],           allow_multiple: false },
    { name: '✅ Yes / No',      poll_type: 'yesno',  question: 'Are you in?',                  choices: null,                                                                              allow_multiple: false },
    { name: '⭐ Rating',        poll_type: 'rating', question: 'How would you rate this?',     choices: null,                                                                              allow_multiple: false },
    { name: '💬 Open question', poll_type: 'text',   question: 'Any comments or suggestions?', choices: null,                                                                              allow_multiple: false },
    { name: '📅 Date picker',   poll_type: 'date',   question: 'Which date works for you?',    choices: [{label:'Option 1'},{label:'Option 2'},{label:'Option 3'}],                       allow_multiple: false },
  ];

  const TYPE_LABELS: Record<PollType, { label: string; desc: string }> = {
    choice:  { label: 'Multiple choice', desc: 'Users pick from a list of options' },
    yesno:   { label: 'Yes / No',        desc: 'Simple binary question' },
    text:    { label: 'Free text',       desc: 'Users type a free-form answer' },
    rating:  { label: 'Rating',          desc: 'Users rate from 1 to 5 stars' },
    date:    { label: 'Date slots',      desc: 'Users pick from date/time options' },
  };

  // ── Saved templates ───────────────────────────────────────────────────────
  let savedTemplates: PollTemplate[] = [];
  let templatesLoading = false;
  let showTemplates = false;
  let savingTemplate = false;
  let newTemplateName = '';
  let showSaveTemplate = false;

  onMount(async () => {
    if (!enabled) return;
    await loadTemplates();
  });

  async function loadTemplates() {
    templatesLoading = true;
    try {
      savedTemplates = await apiFetch<PollTemplate[]>('/api/polls/templates');
    } catch { savedTemplates = []; }
    finally { templatesLoading = false; }
  }

  $: if (enabled && savedTemplates.length === 0 && !templatesLoading) {
    loadTemplates();
  }

  // ── Apply template ────────────────────────────────────────────────────────
  function applyBuiltin(t: typeof BUILTIN_TEMPLATES[0]) {
    pollType      = t.poll_type;
    question      = t.question ?? '';
    allowMultiple = t.allow_multiple;
    choices       = t.choices ? t.choices.map(c => c.label) : ['', ''];
    showTemplates = false;
  }

  function applySaved(t: PollTemplate) {
    pollType      = t.poll_type;
    question      = t.question ?? '';
    allowMultiple = t.allow_multiple;
    choices       = t.choices ? t.choices.map(c => c.label) : ['', ''];
    showTemplates = false;
  }

  async function deleteSavedTemplate(id: number) {
    try {
      await apiFetch(`/api/polls/templates?id=${id}`, { method: 'DELETE' });
      savedTemplates = savedTemplates.filter(t => t.id !== id);
    } catch { /* ignore */ }
  }

  async function saveAsTemplate() {
    if (!newTemplateName.trim()) return;
    savingTemplate = true;
    try {
      const t = await apiFetch<PollTemplate>('/api/polls/templates', {
        method: 'POST',
        body: JSON.stringify({
          name:           newTemplateName.trim(),
          poll_type:      pollType,
          question:       question || null,
          choices:        needsChoices(pollType) ? choices.filter(c => c.trim()) : null,
          allow_multiple: allowMultiple,
          global:         false,
        }),
      });
      savedTemplates = [...savedTemplates, t];
      newTemplateName  = '';
      showSaveTemplate = false;
    } catch { /* ignore */ }
    finally { savingTemplate = false; }
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
    choices = choices.map((c, idx) => idx === i ? val : c);
  }

  // Reset choices when type changes
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

      <!-- Template picker button -->
      <div class="relative">
        <button
                type="button"
                on:click={() => { showTemplates = !showTemplates; showSaveTemplate = false; }}
                class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition w-fit"
        >
          <BookOpen class="w-3.5 h-3.5" />
          Use a template
          <ChevronDown class="w-3 h-3 transition-transform {showTemplates ? 'rotate-180' : ''}" />
        </button>

        {#if showTemplates}
          <div class="absolute left-0 top-6 z-20 w-72 bg-card border border-border rounded-xl shadow-xl overflow-hidden">
            <div class="px-3 py-2 border-b border-border text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">
              Built-in
            </div>
            {#each BUILTIN_TEMPLATES as t}
              <button
                      type="button"
                      on:click={() => applyBuiltin(t)}
                      class="flex items-center justify-between w-full px-3 py-2 text-sm hover:bg-muted transition text-left"
              >
                <span>{t.name}</span>
                <span class="text-[11px] text-muted-foreground">{TYPE_LABELS[t.poll_type].label}</span>
              </button>
            {/each}

            {#if savedTemplates.length > 0}
              <div class="px-3 py-2 border-t border-b border-border text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">
                Saved
              </div>
              {#each savedTemplates as t}
                <div class="flex items-center justify-between px-3 py-2 hover:bg-muted transition group">
                  <button
                          type="button"
                          on:click={() => applySaved(t)}
                          class="flex-1 text-sm text-left flex items-center gap-2"
                  >
                    {#if t.global}<span class="text-[10px] text-amber-400 bg-amber-500/10 border border-amber-500/20 px-1.5 py-0.5 rounded-full">Global</span>{/if}
                    {t.name}
                  </button>
                  <button
                          type="button"
                          on:click={() => deleteSavedTemplate(t.id)}
                          class="w-6 h-6 rounded-md flex items-center justify-center opacity-0 group-hover:opacity-100
                           text-muted-foreground hover:text-red-400 hover:bg-red-500/10 transition"
                  >
                    <X class="w-3 h-3" />
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>

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
                placeholder={pollType === 'yesno' ? 'e.g. Are you coming?' : pollType === 'rating' ? 'e.g. Rate the venue idea' : pollType === 'text' ? 'e.g. What do you want to eat?' : 'e.g. Which option do you prefer?'}
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
            {#each [1,2,3,4,5] as s}
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

        <!-- Allow multiple (only for choice/date) -->
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

      <!-- Save as template -->
      <div class="flex flex-col gap-2 pt-1 border-t border-border/50">
        {#if showSaveTemplate}
          <div class="flex items-center gap-2">
            <input
                    bind:value={newTemplateName}
                    placeholder="Template name…"
                    class="flex-1 rounded-lg border border-input bg-muted/20 px-3 h-8 text-sm outline-none transition
                     focus:border-ring focus:bg-card focus:ring-2 focus:ring-ring/20
                     placeholder:text-muted-foreground/50"
            />
            <button
                    type="button"
                    on:click={saveAsTemplate}
                    disabled={savingTemplate || !newTemplateName.trim()}
                    class="h-8 px-3 rounded-lg bg-muted border border-border text-xs font-medium
                     hover:bg-muted/80 transition disabled:opacity-40"
            >{savingTemplate ? 'Saving…' : 'Save'}</button>
            <button
                    type="button"
                    on:click={() => { showSaveTemplate = false; newTemplateName = ''; }}
                    class="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-muted transition"
            ><X class="w-3.5 h-3.5" /></button>
          </div>
        {:else}
          <button
                  type="button"
                  on:click={() => { showSaveTemplate = true; showTemplates = false; }}
                  class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition w-fit"
          >
            <Save class="w-3 h-3" /> Save as template
          </button>
        {/if}
      </div>

    </div>
  {/if}
</div>