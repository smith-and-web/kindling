<!--
  GuidanceOverlay.svelte - Centralized first-visit guidance (Phase C)

  Shows ONE tooltip at a time as a modal-style overlay:
  - Fixed positioning, floats above the UI
  - Subtle backdrop dims the rest of the interface
  - Strong shadow and clear visual separation
  - Order: sidebar → scene panel → references (based on visibility)
  - "Got it" dismisses current and advances; "Disable tips" turns off all
-->
<script lang="ts">
  import { ui } from "../stores/ui.svelte";
  import { currentProject } from "../stores/project.svelte";
  import type { GuidanceArea } from "../stores/ui.svelte";
  import { Check, Info, EyeOff } from "lucide-svelte";

  const TOOLTIP_CONFIG: Record<
    GuidanceArea,
    { message: string; position: "left" | "center" | "right" }
  > = {
    sidebar: {
      message: "Your outline lives here. Click chapters to expand, drag to reorder.",
      position: "left",
    },
    scenePanel: {
      message: "Edit beats and scenes here. Capture ideas in discovery notes.",
      position: "center",
    },
    references: {
      message:
        "Link characters, locations, and items to scenes. Use the + button to search and add references, or accept the suggested references kindling finds in your prose.",
      position: "right",
    },
    sync: {
      message:
        "Sync from source reimports your outline file when you've made changes elsewhere. Use it to pull in edits from Plottr, yWriter, Longform, Scrivener, or Markdown.",
      position: "left",
    },
    planningStatus: {
      message:
        "Use planning status (Fixed, Flexible, Undefined) to control how much structure each scene has. Start loose and tighten as you go.",
      position: "center",
    },
    screenplay: {
      message:
        "This is a screenplay project. Acts and Sequences replace Chapters. Use sluglines for scene headings — the page count estimate updates as you write.",
      position: "left",
    },
  };

  // planningStatus is shown as an inline banner in ScenePanel, not in this overlay sequence

  const ORDER: GuidanceArea[] = ["screenplay", "sidebar", "scenePanel", "references", "sync"];

  const currentArea = $derived.by(() => {
    if (!ui.guidanceEnabled || !currentProject.value || ui.showOnboarding) return null;

    const sidebarVisible = !ui.sidebarCollapsed;
    const sceneVisible = !!currentProject.currentScene;
    const referencesVisible = !ui.referencesPanelCollapsed;

    const hasSourcePath = !!currentProject.value?.source_path;
    const isScreenplay = currentProject.value?.project_type === "screenplay";
    const visibility: Record<GuidanceArea, boolean> = {
      sidebar: sidebarVisible,
      scenePanel: sceneVisible,
      references: referencesVisible,
      sync: sidebarVisible && hasSourcePath,
      planningStatus: sceneVisible,
      screenplay: sidebarVisible && isScreenplay,
    };

    for (const area of ORDER) {
      if (visibility[area] && !ui.hasSeenTooltip(area)) return area;
    }
    return null;
  });

  const config = $derived(currentArea ? TOOLTIP_CONFIG[currentArea] : null);

  /** The panel each tip describes; the coach mark sits beside it, pointing in. */
  const TARGETS: Record<"left" | "center" | "right", string> = {
    left: '[data-testid="sidebar"]',
    center: '[data-testid="scene-panel"]',
    right: 'aside[aria-label="References"]',
  };
  const GAP = 16;

  let anchor = $state<{ left?: number; right?: number; top: number } | null>(null);
  // A tip never competes with a dialog: it waits until every modal has closed.
  let modalOpen = $state(false);

  function place() {
    if (!config) return;
    const target = document.querySelector(TARGETS[config.position]);
    const rect = target?.getBoundingClientRect();
    if (!rect || rect.width === 0) {
      anchor = null;
      return;
    }
    // Point the arrow (28px down the card) at what the tip is about: the first
    // chapter row for the outline, otherwise a little way into the panel.
    const row = target?.querySelector('[data-testid="chapter-item"]')?.getBoundingClientRect();
    const aim = row && row.height > 0 ? row.top + row.height / 2 - 34 : rect.top + 96;
    const top = Math.max(GAP, Math.min(aim, window.innerHeight - 280));
    anchor =
      config.position === "left"
        ? { left: rect.right + GAP, top }
        : config.position === "right"
          ? { right: window.innerWidth - rect.left + GAP, top }
          : { left: rect.left + rect.width / 2 - 180, top: rect.top + 160 };
  }

  function checkModals() {
    modalOpen = !!document.querySelector("dialog[open], .dialog-scrim, [data-testid='onboarding']");
  }

  $effect(() => {
    if (!currentArea) return;
    checkModals();
    place();
    const observer = new MutationObserver(() => {
      checkModals();
      place();
    });
    observer.observe(document.body, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["open", "class"],
    });
    window.addEventListener("resize", place);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", place);
    };
  });

  function dismiss() {
    if (currentArea) ui.markTooltipSeen(currentArea);
  }

  function disableTips() {
    ui.setGuidanceEnabled(false);
    if (currentArea) ui.markTooltipSeen(currentArea);
  }
  function focusOnMount(node: HTMLElement) {
    queueMicrotask(() => node.focus({ preventScroll: true }));
  }
</script>

{#if currentArea && config && !modalOpen}
  <!-- The shared modal scrim: no blur, one treatment for every overlay. -->
  <div class="guidance-scrim" role="presentation" aria-hidden="true"></div>

  <div
    class="guidance guidance--{config.position}"
    class:is-anchored={!!anchor}
    style:left={anchor?.left !== undefined ? `${anchor.left}px` : null}
    style:right={anchor?.right !== undefined ? `${anchor.right}px` : null}
    style:top={anchor ? `${anchor.top}px` : null}
    role="dialog"
    aria-modal="true"
    aria-labelledby="guidance-title"
    aria-describedby="guidance-message"
  >
    <div class="ka-coach guidance-card">
      <div class="guidance-body">
        <h4 id="guidance-title">
          <Info class="w-5 h-5 ka-icon" aria-hidden="true" />
          Tip
        </h4>
        <p id="guidance-message">{config.message}</p>
      </div>
      <footer class="guidance-actions">
        <button
          type="button"
          onclick={disableTips}
          class="ka-button ka-button--ghost"
          title="Don’t show tips again"
        >
          <EyeOff class="w-5 h-5" aria-hidden="true" />
          Disable tips
        </button>
        <button type="button" onclick={dismiss} class="ka-button" use:focusOnMount>
          <Check class="w-5 h-5" aria-hidden="true" />
          Got it
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .guidance-scrim {
    position: fixed;
    inset: 0;
    z-index: var(--z-guidance-backdrop);
    background: var(--color-overlay-scrim);
  }
  .guidance {
    position: fixed;
    z-index: var(--z-guidance);
    width: min(380px, 90vw);
  }
  /* Unanchored fallback (target not measurable): centred in the window. */
  .guidance:not(.is-anchored) {
    left: 50%;
    top: 40%;
    transform: translate(-50%, -50%);
  }
  /* The arrow points at the panel the tip describes. */
  .guidance.is-anchored:not(.guidance--center) .guidance-card::before {
    content: "";
    position: absolute;
    top: 28px;
    width: 12px;
    height: 12px;
    background: var(--color-surface);
    border-left: var(--border-hair);
    border-bottom: var(--border-hair);
  }
  .guidance--left .guidance-card::before {
    left: -7px;
    transform: rotate(45deg);
  }
  .guidance--right .guidance-card::before {
    right: -7px;
    transform: rotate(-135deg);
  }
  .guidance-card {
    position: relative;
    width: auto;
    display: grid;
    gap: var(--space-s);
    padding: 20px var(--space-m) var(--space-s);
  }
  .guidance-card > .guidance-body {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-2xs);
  }
  .guidance-card h4 :global(.ka-icon) {
    color: var(--color-info);
  }
  .guidance-card h4 {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin: 0;
    font: 550 var(--text-h3) / 1.3 var(--font-display);
    color: var(--color-text);
  }
  .guidance-card p {
    margin: 0;
    font: var(--text-ui) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
  .guidance-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2xs);
    padding-top: var(--space-xs);
    border-top: var(--border-hair);
  }
  @media (prefers-reduced-motion: no-preference) {
    .guidance-card {
      animation: ka-enter var(--ka-motion) ease-out;
    }
  }
</style>
