<script lang="ts">
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
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal"
  role="dialog"
  aria-modal="true"
  aria-labelledby="longform-import-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-lg p-6 max-w-md w-full mx-4 shadow-press-overlay"
  >
    <h3
      id="longform-import-title"
      class="text-press-body-lg font-heading font-medium text-press-text mb-3"
    >
      Import Longform Project
    </h3>
    <p class="font-prose text-press-text text-press-body mb-6 max-w-press-measure">
      Choose how to import your Longform project. Use the index file if you already have a Longform
      project note, or select a vault folder to auto-detect the project.
    </p>
    <div class="space-y-3">
      <button
        onclick={onSelectIndex}
        class="w-full px-4 py-2.5 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors text-press-text text-left"
      >
        <div class="text-press-ui font-medium">Choose Longform Index File</div>
        <div class="text-press-eyebrow text-press-muted">
          Select the .md index note with longform frontmatter
        </div>
      </button>
      <button
        onclick={onSelectVault}
        class="w-full px-4 py-2.5 bg-press-sunken rounded-lg hover:bg-press-sunken transition-colors text-press-text text-left"
      >
        <div class="text-press-ui font-medium">Choose Obsidian Vault Folder</div>
        <div class="text-press-eyebrow text-press-muted">
          Scan the vault to find Longform projects
        </div>
      </button>
    </div>
    <div class="flex justify-end mt-6">
      <button
        onclick={onClose}
        class="px-4 py-2 bg-press-sunken rounded hover:bg-press-sunken transition-colors text-press-text"
      >
        Cancel
      </button>
    </div>
  </div>
</div>
