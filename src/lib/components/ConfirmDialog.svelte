<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  interface Props {
    title: string;
    titleId?: string;
    embedded?: boolean;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    titleId = "dialog-title",
    embedded = false,
    message,
    confirmLabel = "Delete",
    cancelLabel = "Cancel",
    onConfirm,
    onCancel,
  }: Props = $props();

  // Destructive confirms start on the safe choice.
  function focusOnMount(node: HTMLElement) {
    queueMicrotask(() => node.focus());
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  data-testid="confirm-dialog"
  class="dialog-scrim"
  role={embedded ? undefined : "alertdialog"}
  aria-modal={embedded ? undefined : true}
  aria-labelledby={embedded ? undefined : titleId}
  aria-describedby={embedded ? undefined : `${titleId}-message`}
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow confirm">
    <DialogHeader {title} {titleId} onClose={onCancel} closeLabel="Close confirmation" />
    <div class="ka-dialog-body">
      <p id={`${titleId}-message`} data-testid="dialog-message" class="confirm-message">
        {message}
      </p>
    </div>
    <footer class="ka-dialog-footer">
      <button
        type="button"
        data-testid="dialog-cancel"
        onclick={onCancel}
        use:focusOnMount
        class="ka-button ka-button--secondary"
      >
        {cancelLabel}
      </button>
      <button
        type="button"
        data-testid="dialog-confirm"
        onclick={onConfirm}
        class="ka-button ka-button--danger"
      >
        {confirmLabel}
      </button>
    </footer>
  </div>
</div>

<style>
  .confirm {
    display: flex;
    flex-direction: column;
    max-height: calc(100dvh - 48px);
  }
  .confirm-message {
    margin: 0;
    font: var(--text-base) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
</style>
