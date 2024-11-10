<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from "mode-watcher";
  import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  onMount(async () => {
    await onOpenUrl((urls) => {
      console.log('deep link:', urls);
      goto('/');
    });
  });
	let { children } = $props();
</script>
<ModeWatcher />
{@render children()}
