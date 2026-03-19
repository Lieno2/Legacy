<script lang="ts">
  import { RotateCcw, Loader2 } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export interface AuditEntry {
    id: number;
    user_id: string | null;
    username: string | null;
    action: string;
    target_type: string | null;
    target_id: string | null;
    target_name: string | null;
    metadata: Record<string, unknown> | null;
    created_at: string;
  }

  export let audit: AuditEntry[] = [];
  export let auditLoading = false;
  export let reverting: number | null = null;

  const dispatch = createEventDispatcher<{
    refresh: void;
    revert: number;
  }>();

  const ACTION_STYLE: Record<string, string> = {
    create: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    update: 'bg-blue-500/10    text-blue-400    border-blue-500/20',
    delete: 'bg-red-500/10     text-red-400     border-red-500/20',
  };

  function timeAgo(iso: string) {
    const diff = Date.now() - new Date(iso).getTime();
    const m = Math.floor(diff / 60000);
    if (m < 1)  return 'just now';
    if (m < 60) return `${m}m ago`;
    const h = Math.floor(m / 60);
    if (h < 24) return `${h}h ago`;
    return `${Math.floor(h / 24)}d ago`;
  }
</script>

<div class="bg-card border border-border rounded-2xl overflow-hidden">
  <div class="p-5 border-b border-border flex items-center justify-between">
    <div>
      <h2 class="font-semibold text-sm">Audit Log</h2>
      <p class="text-xs text-muted-foreground mt-0.5">Last 200 actions. Deleted events can be reverted.</p>
    </div>
    <button on:click={() => dispatch('refresh')} disabled={auditLoading}
      class="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition disabled:opacity-40">
      <RotateCcw class="w-3.5 h-3.5 {auditLoading ? 'animate-spin' : ''}" />
    </button>
  </div>

  <div class="divide-y divide-border/50">
    {#if auditLoading}
      <div class="flex items-center justify-center gap-2 py-10 text-muted-foreground text-sm">
        <Loader2 class="w-4 h-4 animate-spin" /> Loading…
      </div>
    {:else if audit.length === 0}
      <div class="py-10 text-center text-sm text-muted-foreground">No audit entries yet.</div>
    {:else}
      {#each audit as entry}
        {@const isReverted = entry.metadata?.reverted === true}
        <div class="flex items-start gap-3 px-5 py-3 hover:bg-muted/20 transition">
          <span class="shrink-0 mt-0.5 inline-flex items-center text-[10px] font-semibold px-1.5 py-0.5 rounded-full border uppercase tracking-wide
                       {ACTION_STYLE[entry.action] ?? 'bg-muted text-muted-foreground border-border'}">
            {entry.action}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-sm">
              <span class="font-medium">{entry.username ?? 'System'}</span>
              <span class="text-muted-foreground"> {entry.action}d </span>
              <span class="font-medium">{entry.target_name ?? entry.target_id ?? '—'}</span>
              {#if isReverted}
                <span class="ml-1 text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-1.5 py-0.5 rounded-full">reverted</span>
              {/if}
            </p>
            <p class="text-[11px] text-muted-foreground/60 mt-0.5">{timeAgo(entry.created_at)}</p>
          </div>
          {#if entry.action === 'delete' && entry.target_type === 'event' && !isReverted && entry.metadata}
            <button
              on:click={() => dispatch('revert', entry.id)}
              disabled={reverting === entry.id}
              title="Restore this event"
              class="shrink-0 h-7 px-2.5 rounded-lg border border-border text-xs text-muted-foreground
                     hover:bg-emerald-500/10 hover:text-emerald-400 hover:border-emerald-500/20
                     transition disabled:opacity-40 flex items-center gap-1"
            >
              {#if reverting === entry.id}
                <Loader2 class="w-3 h-3 animate-spin" />
              {:else}
                <RotateCcw class="w-3 h-3" />
              {/if}
              Revert
            </button>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
