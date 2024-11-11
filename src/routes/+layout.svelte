<script lang="ts">
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { onNavigate } from "$app/navigation";

  onNavigate((navigation) => {
    if (!document.startViewTransition) return;

    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });
    });
  });
  onMount(async () => {
    await onOpenUrl((urls) => {
      console.log("deep link:", urls);
      goto("/");
    });
  });
  let { children } = $props();
</script>

<ModeWatcher />
<main class="flex flex-col items-center justify-center h-[100vh]">
  {@render children()}
</main>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }

  @keyframes fade-out {
    to {
      opacity: 0;
    }
  }

  @keyframes slide-from-right {
    from {
      transform: translateX(30px);
    }
  }

  @keyframes slide-to-left {
    to {
      transform: translateX(-30px);
    }
  }

  :root::view-transition-old(root) {
    animation:
      90ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
      300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-left;
  }

  :root::view-transition-new(root) {
    animation:
      210ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in,
      300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
  }
</style>
