<script lang="ts">
  import { X } from "lucide-svelte";
  import type { Snippet } from "svelte";

  let {
    title,
    titleId,
    onClose,
    closeLabel = `Close ${title}`,
    closeTestId,
    disabled = false,
    subtitle,
    children,
  }: {
    title: string;
    titleId?: string;
    onClose: () => void;
    closeLabel?: string;
    closeTestId?: string;
    disabled?: boolean;
    subtitle?: string;
    children?: Snippet;
  } = $props();
</script>

<header class="ka-dialog-header dialog-header">
  <div class="dialog-header-text">
    <h2 id={titleId} class="dialog-title">{title}</h2>
    {#if subtitle}<p>{subtitle}</p>{/if}
  </div>
  <div class="dialog-header-actions">
    {@render children?.()}
    <button
      type="button"
      onclick={onClose}
      {disabled}
      aria-label={closeLabel}
      title={closeLabel}
      data-testid={closeTestId}
      class="ka-button ka-button--ghost ka-icon-button"
      ><X class="w-5 h-5" aria-hidden="true" /></button
    >
  </div>
</header>

<style>
  .dialog-header {
    flex: none;
  }
  .dialog-header-text {
    min-width: 0;
  }
  .dialog-title {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .dialog-header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex: none;
  }
</style>
