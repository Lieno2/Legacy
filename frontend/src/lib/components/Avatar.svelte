<!--
  Avatar component — shows profile picture or initial fallback.
  Props:
    user     — { username, avatar_url } — the user to display
    size     — number, px size (default 40)
    editable — if true, shows a camera overlay on hover to trigger upload
  Events:
    change(dataUrl) — emitted when a new image is selected and resized
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Camera } from 'lucide-svelte';

  export let username: string = '?';
  export let avatarUrl: string | null = null;
  export let size: number = 40;
  export let editable: boolean = false;

  const dispatch = createEventDispatcher<{ change: string }>();

  let fileInput: HTMLInputElement;

  $: hue = (username.charCodeAt(0) * 47) % 360;
  $: initials = username[0]?.toUpperCase() ?? '?';
  $: fontSize = Math.round(size * 0.38);

  function openPicker() {
    if (editable) fileInput.click();
  }

  function handleFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    if (!file.type.startsWith('image/')) return;

    const reader = new FileReader();
    reader.onload = (ev) => {
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        canvas.width  = 128;
        canvas.height = 128;
        const ctx = canvas.getContext('2d')!;
        const min = Math.min(img.width, img.height);
        const sx  = (img.width  - min) / 2;
        const sy  = (img.height - min) / 2;
        ctx.drawImage(img, sx, sy, min, min, 0, 0, 128, 128);
        const dataUrl = canvas.toDataURL('image/jpeg', 0.85);
        dispatch('change', dataUrl);
      };
      img.src = ev.target?.result as string;
    };
    reader.readAsDataURL(file);
    (e.target as HTMLInputElement).value = '';
  }
</script>

<div
  class="relative shrink-0 rounded-full overflow-hidden {editable ? 'cursor-pointer group' : ''}"
  style="width:{size}px; height:{size}px;"
  on:click={openPicker}
  role={editable ? 'button' : 'img'}
  tabindex={editable ? 0 : -1}
  on:keydown={e => e.key === 'Enter' && openPicker()}
  aria-label={editable ? 'Change profile picture' : username}
>
  {#if avatarUrl}
    <img
      src={avatarUrl}
      alt={username}
      class="w-full h-full object-cover"
    />
  {:else}
    <div
      class="w-full h-full flex items-center justify-center font-bold uppercase select-none"
      style="background:hsl({hue},40%,25%); color:hsl({hue},70%,70%); font-size:{fontSize}px;"
    >
      {initials}
    </div>
  {/if}

  {#if editable}
    <div class="absolute inset-0 bg-black/50 flex items-center justify-center
                opacity-0 group-hover:opacity-100 transition-opacity">
      <Camera class="text-white" style="width:{Math.round(size*0.35)}px; height:{Math.round(size*0.35)}px;" />
    </div>
  {/if}
</div>

{#if editable}
  <input
    bind:this={fileInput}
    type="file"
    accept="image/*"
    class="hidden"
    on:change={handleFile}
  />
{/if}
