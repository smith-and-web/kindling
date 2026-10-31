<script lang="ts">
  import { Loader2 } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus, ownsEnterKey } from "../utils/modalFocus";
  import { t } from "../i18n";

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
      error = t("sidebar.renameDialog.nameCannotBeEmpty");
      return;
    }

    saving = true;
    error = null;

    try {
      await onSave(trimmedName);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e || t("sidebar.renameDialog.failed"));
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Enter saves from the name field; on Cancel or Close it activates that button.
    if (event.key === "Enter" && !event.isComposing && !ownsEnterKey(event.target) && !saving) {
      event.preventDefault();
      handleSave();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

<!-- Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, onKeydown: handleKeydown, initialFocus: "#rename-input" }}
  onclick={handleBackdropClick}
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
      closeLabel={t("common.close")}
      closeTestId="rename-close"
    />

    <div class="ka-dialog-body">
      <div class="ka-field od-field">
        <label for="rename-input">{t("sidebar.renameDialog.name")}</label>
        <input
          id="rename-input"
          bind:this={inputRef}
          bind:value={newName}
          type="text"
          placeholder={t("sidebar.renameDialog.enterName")}
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
        {t("common.cancel")}
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
          {t("common.saving")}
        {:else}
          {t("common.save")}
        {/if}
      </button>
    </footer>
  </div>
</div>
