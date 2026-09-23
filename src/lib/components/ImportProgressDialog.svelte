<!--
  ImportProgressDialog.svelte - App-wide progress for imports and sample creation.

  A native modal (top layer, focus contained). It has no close control and
  swallows Escape because an import cannot be cancelled once started.
-->
<script lang="ts">
  import { ui } from "../stores/ui.svelte";

  function modal(dialog: HTMLDialogElement) {
    const restore = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.showModal();
    return {
      destroy() {
        if (dialog.open) dialog.close();
        if (restore?.isConnected) restore.focus();
      },
    };
  }
</script>

{#if ui.isImporting}
  <dialog
    use:modal
    data-testid="import-progress"
    class="ka-dialog press-app import-dialog"
    aria-labelledby="import-progress-title"
    oncancel={(event) => event.preventDefault()}
  >
    <header class="ka-dialog-header">
      <h3 id="import-progress-title">{ui.importTitle}</h3>
    </header>
    <div class="ka-dialog-body ka-progress od-field">
      <div class="ka-between">
        <span id="import-progress-status" role="status" aria-live="polite">
          {ui.importStatus || "Reading your outline…"}
        </span>
        <span class="od-nowrap">{Math.round(ui.importProgress)}%</span>
      </div>
      <progress max="100" value={ui.importProgress} aria-labelledby="import-progress-status"
      ></progress>
    </div>
  </dialog>
{/if}

<style>
  .import-dialog {
    width: min(520px, calc(100% - 32px));
  }
  .import-dialog .ka-dialog-body {
    display: grid;
    gap: var(--space-2xs);
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
</style>
