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

<header
  class="flex items-center justify-between gap-4 px-5 py-4 border-b border-press-border shrink-0"
>
  <div class="min-w-0">
    <h2 id={titleId} class="font-heading text-press-body-lg font-medium text-press-text">
      {title}
    </h2>
    {#if subtitle}<p class="text-press-small text-press-muted mt-1">{subtitle}</p>{/if}
  </div>
  <div class="flex items-center gap-3 shrink-0">
    {@render children?.()}
    <button
      type="button"
      onclick={onClose}
      {disabled}
      aria-label={closeLabel}
      data-testid={closeTestId}
      class="p-2 rounded-lg text-press-muted hover:text-press-text hover:bg-press-sunken transition-colors disabled:opacity-50 disabled:cursor-default"
      ><X class="w-5 h-5" /></button
    >
  </div>
</header>
