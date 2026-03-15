<!--
  GuidanceTooltip.svelte - Contextual first-visit tooltip (Phase C)

  Shows a dismissible callout when:
  - ui.guidanceEnabled is true
  - User has not yet seen the tooltip for this area

  Used in Sidebar, ScenePanel, ReferencesPanel.
-->
<script lang="ts">
  import { ui } from "../stores/ui.svelte";
  import type { GuidanceArea } from "../stores/ui.svelte";
  import type { Snippet } from "svelte";
  import { Check } from "lucide-svelte";

  interface Props {
    area: GuidanceArea;
    message: string;
    position?: "top" | "bottom" | "left" | "right";
    /** When false, tooltip is hidden (e.g. panel collapsed). Does not mark as seen. */
    visible?: boolean;
    children: Snippet;
  }

  let { area, message, position = "bottom", visible = true, children }: Props = $props();

  const show = $derived(visible && ui.guidanceEnabled && !ui.hasSeenTooltip(area));

  function dismiss() {
    ui.markTooltipSeen(area);
  }
</script>

<div class="relative">
  {@render children()}

  {#if show}
    {@const posClasses =
      position === "bottom"
        ? "top-full left-0 mt-2"
        : position === "top"
          ? "bottom-full left-0 mb-2"
          : position === "left"
            ? "right-full top-1/2 -translate-y-1/2 mr-2"
            : "left-full top-1/2 -translate-y-1/2 ml-2"}
    <div
      class="guidance-tooltip absolute z-50 rounded-lg shadow-lg border border-bg-card bg-bg-panel p-3 max-w-xs text-sm text-text-primary {posClasses}"
      role="status"
      aria-live="polite"
    >
      <p class="text-text-secondary mb-2">{message}</p>
      <button
        type="button"
        onclick={dismiss}
        class="flex items-center gap-1.5 px-2 py-1 text-xs font-medium bg-accent/20 text-accent rounded hover:bg-accent/30 transition-colors"
      >
        <Check class="w-3.5 h-3.5" />
        Got it
      </button>
    </div>
  {/if}
</div>

<style>
  .guidance-tooltip {
    animation: guidance-fade-in 0.2s ease-out forwards;
  }
  @keyframes guidance-fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
