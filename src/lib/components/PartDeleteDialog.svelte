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
  class="dialog-scrim"
  role="alertdialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader title={`Delete ${partLabel}`} titleId="dialog-title" onClose={onCancel} />
    <div class="ka-dialog-body part-delete">
      <p class="part-delete-lede">
        “{partTitle}” contains {childChapterCount}
        {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""}. What would you like to do?
      </p>

      <button
        type="button"
        data-testid="delete-part-only"
        onclick={onDeletePartOnly}
        class="ka-button ka-button--secondary part-delete-choice"
      >
        <span>Delete {partLabel.toLowerCase()} only</span>
        <small>
          The {childChapterCount}
          {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""} will remain in the project.
        </small>
      </button>

      <button
        type="button"
        data-testid="delete-part-and-chapters"
        onclick={onDeletePartAndChapters}
        class="ka-button ka-button--danger part-delete-choice"
      >
        <span>Delete {partLabel.toLowerCase()} and all {chapterLabel.toLowerCase()}s</span>
        <small>
          This permanently deletes the {partLabel.toLowerCase()} and its {childChapterCount}
          {chapterLabel.toLowerCase()}{childChapterCount !== 1 ? "s" : ""}.
        </small>
      </button>
    </div>
    <footer class="ka-dialog-footer">
      <button
        type="button"
        data-testid="dialog-cancel"
        onclick={onCancel}
        class="ka-button ka-button--secondary"
      >
        Cancel
      </button>
    </footer>
  </div>
</div>

<style>
  .part-delete {
    display: grid;
    gap: var(--space-2xs);
  }
  .part-delete-lede {
    margin: 0 0 var(--space-2xs);
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .part-delete-choice {
    display: grid;
    justify-items: start;
    gap: var(--space-3xs);
    width: 100%;
    padding: var(--space-xs) var(--space-s);
    text-align: left;
  }
  .part-delete-choice small {
    font: 400 var(--text-small) / 1.4 var(--font-ui);
  }
  .ka-button--secondary.part-delete-choice small {
    color: var(--color-text-muted);
  }
</style>
