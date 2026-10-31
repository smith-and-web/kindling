<script lang="ts">
  import { FileText, FolderOpen } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { t } from "../i18n";
  interface Props {
    onSelectIndex: () => void;
    onSelectVault: () => void;
    onClose: () => void;
  }

  let { onSelectIndex, onSelectVault, onClose }: Props = $props();
</script>

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="longform-import-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      title={t("onboarding.importLongformDialog.title")}
      titleId="longform-import-title"
      {onClose}
    />
    <div class="ka-dialog-body longform">
      <p class="longform-lede">
        {t("onboarding.importLongformDialog.lede")}
      </p>
      <button
        type="button"
        onclick={onSelectIndex}
        class="ka-button ka-button--secondary longform-choice"
      >
        <FileText class="w-5 h-5" aria-hidden="true" />
        <span class="longform-choice-text">
          <span>{t("onboarding.importLongformDialog.indexTitle")}</span>
          <small>{t("onboarding.importLongformDialog.indexHint")}</small>
        </span>
      </button>
      <button
        type="button"
        onclick={onSelectVault}
        class="ka-button ka-button--secondary longform-choice"
      >
        <FolderOpen class="w-5 h-5" aria-hidden="true" />
        <span class="longform-choice-text">
          <span>{t("onboarding.importLongformDialog.vaultTitle")}</span>
          <small>{t("onboarding.importLongformDialog.vaultHint")}</small>
        </span>
      </button>
    </div>
    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button ka-button--secondary">
        {t("common.cancel")}
      </button>
    </footer>
  </div>
</div>

<style>
  .longform {
    display: grid;
    gap: var(--space-2xs);
  }
  .longform-lede {
    margin: 0 0 var(--space-2xs);
    max-width: 60ch;
    font: var(--text-ui) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
  .longform-choice {
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    gap: var(--space-xs);
    width: 100%;
    padding: var(--space-xs) var(--space-s);
    text-align: left;
  }
  .longform-choice-text {
    display: grid;
    gap: var(--space-3xs);
  }
  .longform-choice small {
    font: 400 var(--text-small) / 1.4 var(--font-ui);
    color: var(--color-text-muted);
  }
</style>
