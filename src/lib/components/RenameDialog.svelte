<script lang="ts">
  /* eslint-disable no-undef, svelte/prefer-writable-derived */
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

  let newName = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);
  let inputRef: HTMLInputElement | null = $state(null);

  // Initialize newName from currentName
  $effect(() => {
    newName = currentName;
  });

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

<svelte:window on:keydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="rename-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-md mx-4 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <h2 id="rename-dialog-title" class="text-lg font-medium text-text-primary">
        {title}
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="p-4">
      <label for="rename-input" class="block text-sm font-medium text-text-secondary mb-2">
        Name
      </label>
      <input
        id="rename-input"
        bind:this={inputRef}
        bind:value={newName}
        type="text"
        class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent"
        placeholder="Enter name..."
        disabled={saving}
      />
      {#if error}
        <p class="mt-2 text-sm text-red-400">{error}</p>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSave}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50"
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
