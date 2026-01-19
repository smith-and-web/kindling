<!--
  Tooltip.svelte - Branded tooltip component

  Wraps any element to show a styled tooltip on hover.
  Supports positioning: top, bottom, left, right
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    text: string;
    position?: "top" | "bottom" | "left" | "right";
    children: Snippet;
    delay?: number;
  }

  let { text, position = "top", children, delay = 300 }: Props = $props();

  let visible = $state(false);
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  function showTooltip() {
    timeoutId = setTimeout(() => {
      visible = true;
    }, delay);
  }

  function hideTooltip() {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    visible = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="relative inline-flex"
  onmouseenter={showTooltip}
  onmouseleave={hideTooltip}
  onfocus={showTooltip}
  onblur={hideTooltip}
>
  {@render children()}

  {#if visible && text}
    <div
      class="tooltip absolute z-50 px-3 py-1.5 text-xs font-medium whitespace-nowrap rounded-lg shadow-lg pointer-events-none bg-bg-card text-text-primary"
      class:tooltip-top={position === "top"}
      class:tooltip-bottom={position === "bottom"}
      class:tooltip-left={position === "left"}
      class:tooltip-right={position === "right"}
      role="tooltip"
    >
      {text}
    </div>
  {/if}
</div>

<style>
  .tooltip {
    animation: tooltip-fade-in 0.15s ease-out forwards;
  }

  @keyframes tooltip-fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  /* Position: Top */
  .tooltip-top {
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  /* Position: Bottom */
  .tooltip-bottom {
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  /* Position: Left */
  .tooltip-left {
    right: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }

  /* Position: Right */
  .tooltip-right {
    left: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }
</style>
