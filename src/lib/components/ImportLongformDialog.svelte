<script lang="ts">
  import { FileText, FolderOpen } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  interface Props {
    onSelectIndex: () => void;
    onSelectVault: () => void;
    onClose: () => void;
  }

  let { onSelectIndex, onSelectVault, onClose }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="dialog-scrim"
  role="dialog"
  aria-modal="true"
  aria-labelledby="longform-import-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader title="Import Longform project" titleId="longform-import-title" {onClose} />
    <div class="ka-dialog-body longform">
      <p class="longform-lede">
        Choose how to import your Longform project. Use the index file if you already have a
        Longform project note, or select a vault folder to auto-detect the project.
      </p>
      <button
        type="button"
        onclick={onSelectIndex}
        class="ka-button ka-button--secondary longform-choice"
      >
        <FileText class="w-5 h-5" aria-hidden="true" />
        <span class="longform-choice-text">
          <span>Choose Longform index file</span>
          <small>Select the .md index note with Longform frontmatter</small>
        </span>
      </button>
      <button
        type="button"
        onclick={onSelectVault}
        class="ka-button ka-button--secondary longform-choice"
      >
        <FolderOpen class="w-5 h-5" aria-hidden="true" />
        <span class="longform-choice-text">
          <span>Choose Obsidian vault folder</span>
          <small>Scan the vault to find Longform projects</small>
        </span>
      </button>
    </div>
    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button ka-button--secondary">
        Cancel
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
