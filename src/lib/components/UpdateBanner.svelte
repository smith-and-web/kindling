<script lang="ts">
  import { updateState, installAndRelaunch, dismissUpdate, type UpdateState } from "../updater";
  import { X } from "lucide-svelte";

  let state = $state<UpdateState | null>(null);

  $effect(() => {
    const unsub = updateState.subscribe((s) => {
      state = s;
    });
    return unsub;
  });
</script>

{#if state}
  <div
    class="fixed left-0 right-0 top-0 z-press-toast flex items-center justify-between gap-4 border-b border-press-border bg-press-accent px-4 py-2 text-press-ui text-press-on-accent"
  >
    <span>
      Kindling v{state.version} is ready — Restart to update
    </span>
    <div class="flex items-center gap-2">
      <button
        onclick={() => state && installAndRelaunch(state)}
        class="rounded px-3 py-1 font-medium border border-press-on-accent hover:bg-press-accent-text transition-colors"
      >
        Restart
      </button>
      <button
        onclick={dismissUpdate}
        class="p-1 rounded hover:bg-press-accent-text transition-colors"
        aria-label="Dismiss"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
{/if}
