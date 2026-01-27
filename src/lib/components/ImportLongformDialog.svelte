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
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  role="dialog"
  aria-modal="true"
  aria-labelledby="longform-import-title"
  tabindex="-1"
>
  <div class="bg-bg-panel rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
    <h3 id="longform-import-title" class="text-lg font-heading font-medium text-text-primary mb-3">
      Import Longform Project
    </h3>
    <p class="text-text-secondary text-sm mb-6">
      Choose how to import your Longform project. Use the index file if you already have a Longform
      project note, or select a vault folder to auto-detect the project.
    </p>
    <div class="space-y-3">
      <button
        onclick={onSelectIndex}
        class="w-full px-4 py-2.5 bg-bg-card rounded-lg hover:bg-beat-header transition-colors text-text-primary text-left"
      >
        <div class="text-sm font-medium">Choose Longform Index File</div>
        <div class="text-xs text-text-secondary">
          Select the .md index note with longform frontmatter
        </div>
      </button>
      <button
        onclick={onSelectVault}
        class="w-full px-4 py-2.5 bg-bg-card rounded-lg hover:bg-beat-header transition-colors text-text-primary text-left"
      >
        <div class="text-sm font-medium">Choose Obsidian Vault Folder</div>
        <div class="text-xs text-text-secondary">Scan the vault to find Longform projects</div>
      </button>
    </div>
    <div class="flex justify-end mt-6">
      <button
        onclick={onClose}
        class="px-4 py-2 bg-bg-card rounded hover:bg-beat-header transition-colors text-text-primary"
      >
        Cancel
      </button>
    </div>
  </div>
</div>
