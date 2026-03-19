<script lang="ts">
  import { Hash, Webhook, Save, Loader2, CheckCircle2 } from 'lucide-svelte';

  export let INPUT: string;
  export let TEXTAREA: string;

  interface DiscordConfig {
    webhook_url: string; enabled: boolean; format: string;
    msg_created: string; msg_updated: string; msg_deleted: string;
  }

  export let discord: DiscordConfig;
  export let discordLoading = false;
  export let discordSaving = false;
  export let discordSaved = false;

  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher<{ save: void }>();
</script>

{#if discordLoading}
  <div class="flex items-center justify-center gap-2 py-16 text-muted-foreground">
    <Loader2 class="w-5 h-5 animate-spin" /> Loading Discord settings…
  </div>
{:else}
  <div class="flex flex-col gap-4">
    <div class="bg-card border border-border rounded-2xl p-5 flex items-center gap-4">
      <div class="w-11 h-11 rounded-2xl bg-[#5865f2]/10 border border-[#5865f2]/20 flex items-center justify-center shrink-0">
        <Hash class="w-5 h-5 text-[#5865f2]" />
      </div>
      <div class="flex-1 min-w-0">
        <h2 class="font-semibold text-sm">Discord Webhook</h2>
        <p class="text-xs text-muted-foreground mt-0.5">Send notifications to a Discord channel when events are created, updated or deleted.</p>
      </div>
      <label class="flex items-center gap-2 cursor-pointer shrink-0">
        <span class="text-xs text-muted-foreground">{discord.enabled ? 'On' : 'Off'}</span>
        <div class="relative">
          <input type="checkbox" bind:checked={discord.enabled} class="sr-only peer" />
          <div class="w-9 h-5 rounded-full border transition-colors bg-muted border-border peer-checked:bg-[#5865f2] peer-checked:border-[#5865f2]"></div>
          <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-transform peer-checked:translate-x-4"></div>
        </div>
      </label>
    </div>

    <div class="bg-card border border-border rounded-2xl p-5 flex flex-col gap-3">
      <label class="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
        <Webhook class="w-3.5 h-3.5" /> Webhook URL
      </label>
      <input class={INPUT} bind:value={discord.webhook_url} placeholder="https://discord.com/api/webhooks/..." />
      <div class="flex flex-col gap-1.5 pt-1">
        <span class="text-xs font-medium text-muted-foreground uppercase tracking-wider">Message Format</span>
        <div class="grid grid-cols-2 gap-2">
          <button type="button" on:click={() => (discord.format = 'embed')}
            class="h-9 rounded-xl border text-sm font-medium transition
                   {discord.format === 'embed' ? 'border-[#5865f2]/50 bg-[#5865f2]/10 text-[#5865f2]' : 'border-border text-muted-foreground hover:bg-muted'}">
            Embed
          </button>
          <button type="button" on:click={() => (discord.format = 'plain')}
            class="h-9 rounded-xl border text-sm font-medium transition
                   {discord.format === 'plain' ? 'border-[#5865f2]/50 bg-[#5865f2]/10 text-[#5865f2]' : 'border-border text-muted-foreground hover:bg-muted'}">
            Plain Text
          </button>
        </div>
      </div>
    </div>

    <div class="bg-card border border-border rounded-2xl p-5 flex flex-col gap-4">
      <div>
        <h3 class="text-sm font-semibold">Message Templates</h3>
        <p class="text-xs text-muted-foreground mt-0.5">
          Available placeholders:
          <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.title&#125;</code>
          <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.date&#125;</code>
          <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.location&#125;</code>
          <code class="bg-muted px-1 py-0.5 rounded text-[11px]">&#123;event.creator&#125;</code>
        </p>
      </div>
      {#each [
        { key: 'msg_created' as const, label: '✅ Event Created',  emoji: '📅' },
        { key: 'msg_updated' as const, label: '✏️ Event Updated',  emoji: '✏️' },
        { key: 'msg_deleted' as const, label: '🗑️ Event Deleted',  emoji: '🗑️' },
      ] as tpl}
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground">{tpl.label}</label>
          <textarea class="{TEXTAREA} h-20" bind:value={discord[tpl.key]}
            placeholder="{tpl.emoji} Message for {tpl.label.toLowerCase()}..."
          ></textarea>
        </div>
      {/each}
    </div>

    <button on:click={() => dispatch('save')} disabled={discordSaving}
      class="h-10 rounded-xl font-semibold text-sm transition active:scale-[0.97] disabled:opacity-50 flex items-center justify-center gap-2
             {discordSaved ? 'bg-emerald-500/10 border border-emerald-500/20 text-emerald-400' : 'bg-[#5865f2] text-white hover:bg-[#4752c4]'}">
      {#if discordSaving}
        <Loader2 class="w-4 h-4 animate-spin" /> Saving…
      {:else if discordSaved}
        <CheckCircle2 class="w-4 h-4" /> Saved!
      {:else}
        <Save class="w-4 h-4" /> Save Discord Settings
      {/if}
    </button>
  </div>
{/if}
