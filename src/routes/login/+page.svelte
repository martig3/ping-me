<script lang="ts">
  import DiscordIcon from '$lib/img/discord.svg';
  import Button from '$lib/components/ui/button/button.svelte';

  function onClick() {}
</script>

<main class="flex flex-row gap-2 w-full items-center justify-center h-[100vh]">
  <Button onclick={onClick}
    ><img src={DiscordIcon} alt="discord logo" class="size-6" /> Continue with Discord</Button
  >
</main>
