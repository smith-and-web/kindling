<script lang="ts">
  interface Props {
    partTitle: string;
    childChapterCount: number;
    onDeletePartOnly: () => void;
    onDeletePartAndChapters: () => void;
    onCancel: () => void;
  }

  let { partTitle, childChapterCount, onDeletePartOnly, onDeletePartAndChapters, onCancel }: Props =
    $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  data-testid="part-delete-dialog"
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  role="dialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  tabindex="-1"
>
  <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
    <h3 id="dialog-title" class="text-lg font-heading font-medium text-text-primary mb-4">
      Delete Part
    </h3>
    <p class="text-text-secondary text-sm mb-2">
      "{partTitle}" contains {childChapterCount} chapter{childChapterCount !== 1 ? "s" : ""}.
    </p>
    <p class="text-text-secondary text-sm mb-6">What would you like to do?</p>

    <div class="space-y-3 mb-6">
      <button
        data-testid="delete-part-only"
        onclick={onDeletePartOnly}
        class="w-full text-left px-4 py-3 bg-bg-card rounded-lg hover:bg-beat-header transition-colors border border-transparent hover:border-accent"
      >
        <div class="font-medium text-text-primary">Delete Part only</div>
        <div class="text-xs text-text-secondary mt-1">
          The {childChapterCount} chapter{childChapterCount !== 1 ? "s" : ""} will remain in the project
        </div>
      </button>

      <button
        data-testid="delete-part-and-chapters"
        onclick={onDeletePartAndChapters}
        class="w-full text-left px-4 py-3 bg-red-600/10 rounded-lg hover:bg-red-600/20 transition-colors border border-red-600/30"
      >
        <div class="font-medium text-red-400">Delete Part and all chapters</div>
        <div class="text-xs text-red-300/70 mt-1">
          This will permanently delete the Part and its {childChapterCount} chapter{childChapterCount !==
          1
            ? "s"
            : ""}
        </div>
      </button>
    </div>

    <div class="flex justify-end">
      <button
        data-testid="dialog-cancel"
        onclick={onCancel}
        class="px-4 py-2 bg-bg-card rounded hover:bg-beat-header transition-colors text-text-primary"
      >
        Cancel
      </button>
    </div>
  </div>
</div>
