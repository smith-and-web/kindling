<script lang="ts">
  import { updateState, installAndRelaunch, dismissUpdate, type UpdateState } from "../updater";
  import { tick } from "svelte";
  import { Download, Loader2, X } from "lucide-svelte";
  import { ui } from "../stores/ui.svelte";

  let {
    disabled = false,
    restarting = $bindable(false),
    prepare,
    captureFocus,
  }: {
    disabled?: boolean;
    restarting?: boolean;
    prepare?: () => Promise<void>;
    captureFocus?: () => (restore: boolean) => void;
  } = $props();

  let state = $state<UpdateState | null>(null);

  $effect(() => {
    const unsub = updateState.subscribe((s) => {
      state = s;
    });
    return unsub;
  });

  async function restart() {
    if (!state || disabled || restarting) return;
    const finishFocus = captureFocus?.();
    let failed = false;
    restarting = true;
    try {
      await prepare?.();
      await installAndRelaunch(state);
    } catch (error) {
      failed = true;
      ui.showError(`Could not restart to update: ${String(error)}`);
    } finally {
      restarting = false;
      await tick();
      finishFocus?.(failed);
    }
  }
</script>

{#if state}
  <div class="ka-banner update-banner" role="status">
    <Download class="w-5 h-5 ka-icon" aria-hidden="true" />
    <span>kindling v{state.version} is ready — Restart to update</span>
    <div class="ka-row">
      <button
        type="button"
        onpointerdown={(event) => event.preventDefault()}
        onclick={restart}
        disabled={disabled || restarting}
        aria-busy={restarting || undefined}
        class="ka-button ka-button--secondary"
      >
        {#if restarting}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Restarting…
        {:else}
          Restart
        {/if}
      </button>
      <button
        type="button"
        onclick={dismissUpdate}
        disabled={disabled || restarting}
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label="Dismiss"
        title="Dismiss"
      >
        <X class="w-5 h-5" aria-hidden="true" />
      </button>
    </div>
  </div>
{/if}

<style>
  .update-banner {
    flex: none;
  }
</style>
