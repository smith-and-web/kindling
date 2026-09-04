<script lang="ts">
  import { X, Loader2 } from "lucide-svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    title,
    currentName,
    onSave,
    onClose,
  }: {
    title: string;
    currentName: string;
    onSave: (newName: string) => Promise<void>;
    onClose: () => void;
  } = $props();

  let newName = $state(currentName);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let inputRef: HTMLInputElement | null = $state(null);

  // Focus input on mount
  $effect(() => {
    if (inputRef) {
      inputRef.focus();
      inputRef.select();
    }
  });

  async function handleSave() {
    const trimmedName = newName.trim();
    if (!trimmedName) {
      error = "Name cannot be empty";
      return;
    }

    saving = true;
    error = null;

    try {
      await onSave(trimmedName);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to rename";
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && !saving) {
      handleSave();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="rename-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div
    class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
      <h2 id="rename-dialog-title" class="text-press-body-lg font-medium text-press-text">
        {title}
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="p-4">
      <label for="rename-input" class="block text-press-ui font-medium text-press-muted mb-2">
        Name
      </label>
      <input
        id="rename-input"
        bind:this={inputRef}
        bind:value={newName}
        type="text"
        class="w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 focus:outline-none focus:border-press-accent"
        placeholder="Enter name..."
        disabled={saving}
      />
      {#if error}
        <p class="mt-2 text-press-ui text-press-error">{error}</p>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSave}
        class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors"
        disabled={saving || !newName.trim()}
      >
        {#if saving}
          <Loader2 class="w-4 h-4 animate-spin" />
        {:else}
          Save
        {/if}
      </button>
    </div>
  </div>
</div>
