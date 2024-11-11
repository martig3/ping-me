<script lang="ts">
  import DiscordIcon from "$lib/img/discord.svg";
  import Button from "$lib/components/ui/button/button.svelte";
  import { PUBLIC_BASE_API } from "$env/static/public";

  async function onClick() {
    window.location.href = `${PUBLIC_BASE_API}/auth/login`;
  }
</script>

<main class="flex flex-row gap-2 w-full items-center justify-center h-[100vh]">
  <Button onclick={onClick}>
    <img src={DiscordIcon} alt="discord logo" class="size-6" />
    Continue with Discord
  </Button>
</main>
