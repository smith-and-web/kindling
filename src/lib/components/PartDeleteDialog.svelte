<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { t } from "../i18n";
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
    partLabel = t("sidebar.partDeleteDialog.partLabel"),
    chapterLabel = t("sidebar.partDeleteDialog.chapterLabel"),
    onDeletePartOnly,
    onDeletePartAndChapters,
    onCancel,
  }: Props = $props();
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
        {#if childChapterCount !== 1}
          {t("sidebar.partDeleteDialog.lede.many", {
            title: partTitle,
            count: childChapterCount,
            label: chapterLabel.toLowerCase(),
          })}
        {:else}
          {t("sidebar.partDeleteDialog.lede.one", {
            title: partTitle,
            count: childChapterCount,
            label: chapterLabel.toLowerCase(),
          })}
        {/if}
      </p>

      <button
        type="button"
        data-testid="delete-part-only"
        onclick={onDeletePartOnly}
        class="ka-button ka-button--secondary part-delete-choice"
      >
        <span>{t("sidebar.partDeleteDialog.deleteOnly", { label: partLabel.toLowerCase() })}</span>
        <small>
          {#if childChapterCount !== 1}
            {t("sidebar.partDeleteDialog.deleteOnlyHint.many", {
              count: childChapterCount,
              label: chapterLabel.toLowerCase(),
            })}
          {:else}
            {t("sidebar.partDeleteDialog.deleteOnlyHint.one", {
              count: childChapterCount,
              label: chapterLabel.toLowerCase(),
            })}
          {/if}
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
            chapter: chapterLabel.toLowerCase(),
          })}</span
        >
        <small>
          {#if childChapterCount !== 1}
            {t("sidebar.partDeleteDialog.deleteAllHint.many", {
              part: partLabel.toLowerCase(),
              count: childChapterCount,
              chapter: chapterLabel.toLowerCase(),
            })}
          {:else}
            {t("sidebar.partDeleteDialog.deleteAllHint.one", {
              part: partLabel.toLowerCase(),
              count: childChapterCount,
              chapter: chapterLabel.toLowerCase(),
            })}
          {/if}
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
