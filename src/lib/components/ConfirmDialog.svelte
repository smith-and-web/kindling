<script lang="ts">
  interface Props {
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    message,
    confirmLabel = "Delete",
    cancelLabel = "Cancel",
    onConfirm,
    onCancel,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  data-testid="confirm-dialog"
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  role="dialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  tabindex="-1"
>
  <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
    <h3 id="dialog-title" class="text-lg font-heading font-medium text-text-primary mb-4">
      {title}
    </h3>
    <p data-testid="dialog-message" class="text-text-secondary text-sm mb-6">
      {message}
    </p>
    <div class="flex gap-3 justify-end">
      <button
        data-testid="dialog-cancel"
        onclick={onCancel}
        class="px-4 py-2 bg-bg-card rounded hover:bg-beat-header transition-colors text-text-primary"
      >
        {cancelLabel}
      </button>
      <button
        data-testid="dialog-confirm"
        onclick={onConfirm}
        class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
