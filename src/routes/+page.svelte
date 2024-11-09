<script lang="ts">
  import DiscordIcon from '$lib/img/discord.svg';
  import Button from '$lib/components/ui/button/button.svelte';
  import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  onMount(async () => {
    await onOpenUrl((urls) => {
      console.log('deep link:', urls);
      goto('/authenticated');
    });
  });
  function onClick() {
    window.location.href = 'http://localhost:3000/api/auth/login';
  }
</script>

<main class="flex flex-row gap-2 w-full items-center justify-center h-[100vh]">
  <Button onclick={onClick}
    ><img src={DiscordIcon} alt="discord logo" class="size-6" /> Continue with Discord</Button
  >
</main>
