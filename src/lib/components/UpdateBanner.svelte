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
    class="fixed left-0 right-0 top-0 z-[90] flex items-center justify-between gap-4 bg-accent px-4 py-2 text-sm text-white shadow-md"
  >
    <span>
      Kindling v{state.version} is ready — Restart to update
    </span>
    <div class="flex items-center gap-2">
      <button
        onclick={() => state && installAndRelaunch(state)}
        class="rounded px-3 py-1 font-medium bg-white/20 hover:bg-white/30 transition-colors"
      >
        Restart
      </button>
      <button
        onclick={dismissUpdate}
        class="p-1 rounded hover:bg-white/20 transition-colors"
        aria-label="Dismiss"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
{/if}
