<script lang="ts">
  import type { PageData } from './$types';
  import * as Avatar from '$lib/components/ui/avatar/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import {
    CornerDownLeftIcon,
    CornerDownRightIcon,
    LoaderCircleIcon,
    PauseIcon,
    PlayIcon,
  } from 'lucide-svelte';
  import Button from '$lib/components/ui/button/button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { NotifyState } from '$lib/types/notify-state';
  import { listen } from '@tauri-apps/api/event';
  import { differenceInMinutes } from 'date-fns';
  import { onDestroy, onMount } from 'svelte';

  const { data }: { data: PageData } = $props();
  const user = $derived(data.user);
  let interval = $state<number | null>(null);
  onMount(async () => {
    interval = setInterval(updateLastNotifiedDisplay, 1000);
    const state: NotifyState = await invoke('is_notifying');
    notifying = state.isRunning;
  });
  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
  let notifying = $state(false);
  let lastNotified = $state<Date | null>(null);
  let updateLastNotifiedDisplay = () => {
    if (!lastNotified) return 'n/a';
    let minAgo = differenceInMinutes(new Date(), lastNotified);
    if (minAgo < 1) {
      lastNotifiedDisplay = 'Just now';
      return;
    }
    if (minAgo < 60) {
      lastNotifiedDisplay = `${differenceInMinutes(new Date(), lastNotified)} minute${minAgo > 1 ? 's' : ''} ago`;
      return;
    }

    lastNotifiedDisplay = `Over ${Math.floor(minAgo / 60)} hours ago`;
  };
  let lastNotifiedDisplay = $state(updateLastNotifiedDisplay());
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
  <Card.Root class="border-none !bg-transparent w-full">
    <Card.Header>
      <Card.Title class="flex flex-row gap-2 items-center">
        <Avatar.Root>
          <Avatar.Image src={user.discordAvatar} alt="{user.name} avatar" />
          <Avatar.Fallback><LoaderCircleIcon /></Avatar.Fallback>
        </Avatar.Root>
        <div class="flex flex-col -mt-1">
          <span class="text-sm text-gray-400 leading-tight">
            notifying
            <CornerDownLeftIcon
              class="rotate-[270deg] inline-block size-3 translate-y-[1px]"
            />
          </span>
          <span class="text-3xl font-bold leading-6">{user.name}</span>
        </div>
      </Card.Title>
    </Card.Header>
    <Card.Content>
      <div class="flex flex-row gap-4 w-full [&>*]:text-sm">
        <div>Last Notified</div>
        <span class="text-gray-400">{lastNotifiedDisplay}</span>
      </div>
    </Card.Content>
    <Card.Footer class="flex flex-row justify-between">
      {#if notifying}
        <Button class="w-full group [&>*]:transition-all" onclick={stop}>
          <LoaderCircleIcon class="size-4 animate-spin group-hover:hidden" />
          <PauseIcon class="size-4 hidden group-hover:block" />
          Pause Notifying
        </Button>
      {:else}
        <Button class="w-full" onclick={start}>
          <PlayIcon class="size-4" /> Start Notifying
        </Button>
      {/if}
    </Card.Footer>
  </Card.Root>
</main>
