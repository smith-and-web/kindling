<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { countLabel } from "../utils/plural";
  import { t, tn } from "../i18n";
  interface Props {
    partTitle: string;
    childChapterCount: number;
    partLabel?: string;
    /** What the project calls the part's children: screenplays use sequences. */
    chapterNoun?: "chapter" | "sequence";
    onDeletePartOnly: () => void;
    onDeletePartAndChapters: () => void;
    onCancel: () => void;
  }

  let {
    partTitle,
    childChapterCount,
    partLabel = t("sidebar.partDeleteDialog.partLabel"),
    chapterNoun = "chapter",
    onDeletePartOnly,
    onDeletePartAndChapters,
    onCancel,
  }: Props = $props();

  const chapters = $derived(countLabel(childChapterCount, chapterNoun));
  const allChapters = $derived(
    t(
      chapterNoun === "sequence"
        ? "sidebar.partDeleteDialog.allSequences"
        : "sidebar.partDeleteDialog.allChapters"
    )
  );
</script>

<div
  data-testid="part-delete-dialog"
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onCancel, initialFocus: '[data-testid="dialog-cancel"]' }}
  role="alertdialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title={t("sidebar.partDeleteDialog.title", { label: partLabel })}
      titleId="dialog-title"
      onClose={onCancel}
    />
    <div class="ka-dialog-body part-delete">
      <p class="part-delete-lede">
        {tn("sidebar.partDeleteDialog.lede", childChapterCount, { title: partTitle, chapters })}
      </p>

      <button
        type="button"
        data-testid="delete-part-only"
        onclick={onDeletePartOnly}
        class="ka-button ka-button--secondary part-delete-choice"
      >
        <span>{t("sidebar.partDeleteDialog.deleteOnly", { label: partLabel.toLowerCase() })}</span>
        <small>
          {tn("sidebar.partDeleteDialog.deleteOnlyHint", childChapterCount, { chapters })}
        </small>
      </button>

      <button
        type="button"
        data-testid="delete-part-and-chapters"
        onclick={onDeletePartAndChapters}
        class="ka-button ka-button--danger part-delete-choice"
      >
        <span
          >{t("sidebar.partDeleteDialog.deleteAll", {
            part: partLabel.toLowerCase(),
            chapters: allChapters,
          })}</span
        >
        <small>
          {tn("sidebar.partDeleteDialog.deleteAllHint", childChapterCount, {
            part: partLabel.toLowerCase(),
            chapters,
          })}
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
        {t("common.cancel")}
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
