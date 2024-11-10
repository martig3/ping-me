<script lang="ts">
  import type { PageData } from './$types';
  import * as Avatar from '$lib/components/ui/avatar/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import { LoaderCircleIcon, PlayIcon } from 'lucide-svelte';
  import Button from '$lib/components/ui/button/button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { NotifyState } from '$lib/types/notify-state';
  import { listen } from '@tauri-apps/api/event';
  import { differenceInMinutes } from 'date-fns';

  const { data }: { data: PageData } = $props();
  const user = $derived(data.user);
  let notifying = $state(false);
  let lastNotified = $state<Date | null>(null);
  let lastNotifiedDisplay = $derived.by(() => {
    if (!lastNotified) return 'n/a';
    if (differenceInMinutes(new Date(), lastNotified) < 1) return 'Just now';
    if (differenceInMinutes(new Date(), lastNotified) < 60)
      return `${differenceInMinutes(new Date(), lastNotified)} minutes ago`;

    return `Over ${Math.floor(differenceInMinutes(new Date(), lastNotified) / 60)} hours ago`;
  });
  async function start() {
    const state: NotifyState = await invoke('start_notifying');
    notifying = state.isRunning;
  }
  async function stop() {
    const state: NotifyState = await invoke('stop_notifying');
    notifying = state.isRunning;
  }
  listen('notified', () => {
    lastNotified = new Date();
  });
</script>

<main class="flex flex-col items-center justify-center h-[100vh]">
  <Card.Root class="border-none !bg-transparent">
    <Card.Header>
      <Card.Title class="flex flex-row gap-2 items-center">
        <Avatar.Root>
          <Avatar.Image src={user.discordAvatar} alt="{user.name} avatar" />
          <Avatar.Fallback><LoaderCircleIcon /></Avatar.Fallback>
        </Avatar.Root>
        <span class="text-3xl font-bold">{user.name}</span>
      </Card.Title>
    </Card.Header>
    <Card.Content>
      <div class="flex flex-row justify-between w-full">
        <div>Last Notified</div>
        <span class="text-gray-400">{lastNotifiedDisplay}</span>
      </div>
    </Card.Content>
    <Card.Footer class="flex flex-row justify-between">
      {#if notifying}
        <Button class="w-full" onclick={stop}>
          <LoaderCircleIcon class="size-4 animate-spin" /> Pause Notifying
        </Button>
      {:else}
        <Button class="w-full" onclick={start}>
          <PlayIcon class="size-4" /> Start Notifying
        </Button>
      {/if}
    </Card.Footer>
  </Card.Root>
</main>
