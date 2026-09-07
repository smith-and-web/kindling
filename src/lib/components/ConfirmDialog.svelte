<script lang="ts">
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  data-testid="confirm-dialog"
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal"
  role={embedded ? undefined : "dialog"}
  aria-modal={embedded ? undefined : true}
  aria-labelledby={embedded ? undefined : titleId}
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-lg p-6 max-w-md w-full mx-4 shadow-press-overlay"
  >
    <h3 id={titleId} class="text-press-body-lg font-heading font-medium text-press-text mb-4">
      {title}
    </h3>
    <p
      data-testid="dialog-message"
      class="font-prose text-press-text text-press-body mb-6 max-w-press-measure"
    >
      {message}
    </p>
    <div class="flex gap-3 justify-end">
      <button
        data-testid="dialog-cancel"
        onclick={onCancel}
        class="px-4 py-2 bg-press-sunken rounded hover:bg-press-sunken transition-colors text-press-text"
      >
        {cancelLabel}
      </button>
      <button
        data-testid="dialog-confirm"
        onclick={onConfirm}
        class="px-4 py-2 bg-press-error text-press-on-accent rounded hover:bg-press-error transition-colors"
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
