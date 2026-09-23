<script lang="ts">
  import { Loader2 } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";

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

<div
  class="dialog-scrim"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="rename-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell">
    <DialogHeader
      {title}
      titleId="rename-dialog-title"
      {onClose}
      closeLabel="Close"
      closeTestId="rename-close"
    />

    <div class="ka-dialog-body">
      <div class="ka-field od-field">
        <label for="rename-input">Name</label>
        <input
          id="rename-input"
          bind:this={inputRef}
          bind:value={newName}
          type="text"
          placeholder="Enter name…"
          disabled={saving}
          aria-invalid={error ? true : undefined}
          aria-describedby={error ? "rename-error" : undefined}
        />
        {#if error}
          <p id="rename-error" class="ka-error" role="alert">{error}</p>
        {/if}
      </div>
    </div>

    <footer class="ka-dialog-footer">
      <button
        type="button"
        onclick={onClose}
        class="ka-button ka-button--secondary"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        data-testid="rename-save"
        type="button"
        onclick={handleSave}
        class="ka-button"
        disabled={saving || !newName.trim()}
        aria-busy={saving || undefined}
      >
        {#if saving}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Saving…
        {:else}
          Save
        {/if}
      </button>
    </footer>
  </div>
</div>
