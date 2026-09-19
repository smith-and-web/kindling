<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  interface Props {
    partTitle: string;
    childChapterCount: number;
    partLabel?: string;
    chapterLabel?: string;
    onDeletePartOnly: () => void;
    onDeletePartAndChapters: () => void;
    onCancel: () => void;
  }

  let {
    partTitle,
    childChapterCount,
    partLabel = "Part",
    chapterLabel = "Chapter",
    onDeletePartOnly,
    onDeletePartAndChapters,
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
  data-testid="part-delete-dialog"
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal"
  role="dialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-lg overflow-hidden max-w-md w-full mx-4 shadow-press-overlay"
  >
    <DialogHeader title={`Delete ${partLabel}`} titleId="dialog-title" onClose={onCancel} />
    <div class="p-6">
      <p class="text-press-muted text-press-ui mb-2">
        "{partTitle}" contains {childChapterCount}
        {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""}.
      </p>
      <p class="text-press-muted text-press-ui mb-6">What would you like to do?</p>

      <div class="space-y-3 mb-6">
        <button
          data-testid="delete-part-only"
          onclick={onDeletePartOnly}
          class="w-full text-left px-4 py-3 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors border border-transparent hover:border-press-accent"
        >
          <div class="font-medium text-press-text">Delete {partLabel} only</div>
          <div class="text-press-eyebrow text-press-muted mt-1">
            The {childChapterCount}
            {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""} will remain in the project
          </div>
        </button>

        <button
          data-testid="delete-part-and-chapters"
          onclick={onDeletePartAndChapters}
          class="w-full text-left px-4 py-3 bg-press-error-wash rounded-lg hover:bg-press-error-wash transition-colors border border-press-error"
        >
          <div class="font-medium text-press-error">
            Delete {partLabel} and all {chapterLabel.toLowerCase()}s
          </div>
          <div class="text-press-eyebrow text-press-error mt-1">
            This will permanently delete the {partLabel} and its {childChapterCount}
            {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""}
          </div>
        </button>
      </div>

      <div class="flex justify-end">
        <button
          data-testid="dialog-cancel"
          onclick={onCancel}
          class="px-4 py-2 bg-press-sunken rounded hover:bg-press-sunken transition-colors text-press-text"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>
