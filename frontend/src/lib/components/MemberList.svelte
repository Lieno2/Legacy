<script lang="ts">
  import { Users, AlertCircle, Loader2 } from 'lucide-svelte';
  import type { EventMember, RsvpStatus } from '$lib/types';

  export let members: EventMember[] = [];
  export let membersLoading = false;
  export let membersError = '';

  const STATUS_BADGE: Record<RsvpStatus, string> = {
    going:     'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
    late:      'bg-amber-500/15  text-amber-400  border-amber-500/30',
    not_going: 'bg-red-500/15    text-red-400    border-red-500/30',
  };
  const STATUS_LABEL: Record<RsvpStatus, string> = {
    going: 'Going', late: 'Late', not_going: 'Not going',
  };
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center gap-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider">
    <Users class="w-3.5 h-3.5" />
    Attendees {#if !membersLoading}({members.length}){/if}
  </div>

  {#if membersLoading}
    <div class="flex items-center gap-2 text-xs text-muted-foreground py-1">
      <Loader2 class="w-3.5 h-3.5 animate-spin" /> Loading...
    </div>
  {:else if membersError}
    <div class="flex items-center gap-2 text-xs text-red-400 bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2">
      <AlertCircle class="w-3.5 h-3.5 shrink-0" /> {membersError}
    </div>
  {:else if members.length === 0}
    <p class="text-xs text-muted-foreground">No RSVPs yet — be the first!</p>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each members as m}
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <div class="w-5 h-5 rounded-full border border-border flex items-center justify-center text-[10px] font-bold uppercase"
              style="background:hsl({((m.username ?? '?').charCodeAt(0)*47)%360},40%,25%); color:hsl({((m.username ?? '?').charCodeAt(0)*47)%360},70%,70%);"
              aria-hidden="true"
            >{(m.username ?? '?')[0]}</div>
            <span class="text-sm">{m.username ?? 'Unknown'}</span>
          </div>
          <span class="text-[11px] px-1.5 py-0.5 rounded-full border {STATUS_BADGE[m.status as RsvpStatus]}">
            {STATUS_LABEL[m.status as RsvpStatus]}{m.status === 'late' && m.late_minutes ? ` +${m.late_minutes}m` : ''}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>
