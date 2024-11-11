<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index.js";
  import type { Trigger } from "$lib/types/trigger";
  import {
    ArrowLeftIcon,
    PlusIcon,
    SaveIcon,
    SettingsIcon,
    TrashIcon,
  } from "lucide-svelte";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { goto } from "$app/navigation";
  import type { NotifyState } from "$lib/types/notify-state";
  import { invoke } from "@tauri-apps/api/core";

  let triggers = $state<Trigger[]>(
    JSON.parse(localStorage.getItem("triggers") || "[]"),
  );
  let newTrigger = $state<Trigger>({ phrase: "", isActive: true });
  function addTrigger() {
    if (triggers.find((trigger) => trigger.phrase === newTrigger.phrase)) {
      return;
    }
    triggers.push(newTrigger);
    newTrigger = { phrase: "", isActive: true };
  }
  async function save() {
    localStorage.setItem("triggers", JSON.stringify(triggers));
    const state: NotifyState = await invoke("is_notifying");
    if (state.isRunning) {
      await invoke("stop_notifying");
      await invoke("start_notifying", {
        phrases: triggers.filter((t) => t.isActive).map((t) => t.phrase),
      });
    }
    goto("/");
  }
  function remove(removed: Trigger) {
    triggers = triggers.filter((trigger) => trigger.phrase !== removed.phrase);
    localStorage.setItem("triggers", JSON.stringify(triggers));
  }
</script>

<Card.Root class="border-none !bg-transparent w-full">
  <Card.Header>
    <Card.Title class="flex flex-row gap-2 items-center">
      <SettingsIcon class="size-4" /> Config
    </Card.Title>
    <Card.Description>
      Phrases on your screen(s) that trigger a notification.
    </Card.Description>
  </Card.Header>
  <Card.Content>
    <div class="flex flex-col gap-4">
      {#each triggers as trigger}
        <div class="flex flex-row gap-2 items-center">
          <Input
            type="text"
            class={!trigger.isActive ? "text-gray-500" : ""}
            placeholder="Phrase to trigger notification"
            bind:value={trigger.phrase}
          />
          <Switch bind:checked={trigger.isActive} />
          <Button
            variant="destructive"
            size="icon"
            class="w-12  h-8"
            onclick={() => remove(trigger)}
          >
            <TrashIcon />
          </Button>
        </div>
      {/each}
      <div class="flex flex-row gap-2">
        <Input
          type="text"
          placeholder="New phrase (case sensitive)"
          bind:value={newTrigger.phrase}
        />
        <Button
          variant="ghost"
          size="icon"
          disabled={!newTrigger.phrase ||
            !!triggers.find((trigger) => trigger.phrase === newTrigger.phrase)}
          onclick={addTrigger}><PlusIcon /></Button
        >
      </div>
    </div>
  </Card.Content>
  <Card.Footer class="flex flex-row gap-2">
    <Button variant="outline" href="/">
      <ArrowLeftIcon /> Back
    </Button>
    <Button class="w-full" onclick={save}
      ><SaveIcon class="size-4" />Save</Button
    >
  </Card.Footer>
</Card.Root>
