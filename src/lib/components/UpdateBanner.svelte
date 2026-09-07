<script lang="ts">
  import { updateState, installAndRelaunch, dismissUpdate, type UpdateState } from "../updater";
  import { X } from "lucide-svelte";
  import { ui } from "../stores/ui.svelte";

  let {
    disabled = false,
    restarting = $bindable(false),
    prepare,
  }: { disabled?: boolean; restarting?: boolean; prepare?: () => Promise<void> } = $props();

  let state = $state<UpdateState | null>(null);

  $effect(() => {
    const unsub = updateState.subscribe((s) => {
      state = s;
    });
    return unsub;
  });

  async function restart() {
    if (!state || disabled || restarting) return;
    restarting = true;
    try {
      await prepare?.();
      await installAndRelaunch(state);
    } catch (error) {
      ui.showError(`Could not restart to update: ${String(error)}`);
    } finally {
      restarting = false;
    }
  }
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
        onclick={restart}
        disabled={disabled || restarting}
        class="rounded px-3 py-1 font-medium border border-press-on-accent hover:bg-press-accent-text transition-colors disabled:bg-press-disabled-bg disabled:text-press-disabled-text disabled:border-press-disabled-border"
      >
        Restart
      </button>
      <button
        onclick={dismissUpdate}
        disabled={disabled || restarting}
        class="p-1 rounded hover:bg-press-accent-text transition-colors disabled:bg-press-disabled-bg disabled:text-press-disabled-text"
        aria-label="Dismiss"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
{/if}
