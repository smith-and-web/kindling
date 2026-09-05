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
      message: "Edit beats and scenes here. Add discovery notes with ⌘D.",
      position: "center",
    },
    references: {
      message:
        "Link characters, locations, and items to scenes. Use the + button to search and add references, or accept auto-detected ⚡ suggestions.",
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

  const positionClasses = $derived(
    config?.position === "left"
      ? "left-[min(2rem,5%)] top-1/2 -translate-y-1/2"
      : config?.position === "right"
        ? "right-[min(2rem,5%)] top-1/2 -translate-y-1/2"
        : "left-1/2 top-[40%] -translate-x-1/2 -translate-y-1/2"
  );

  function dismiss() {
    if (currentArea) ui.markTooltipSeen(currentArea);
  }

  function disableTips() {
    ui.setGuidanceEnabled(false);
    if (currentArea) ui.markTooltipSeen(currentArea);
  }
</script>

{#if currentArea && config}
  <!-- Backdrop: subtle dim, modal-style -->
  <div
    class="fixed inset-0 z-press-guidance-backdrop bg-press-overlay backdrop-blur-[2px]"
    role="presentation"
    aria-hidden="true"
  ></div>

  <!-- Tooltip card: modal-style, floats above -->
  <div
    class="fixed z-press-guidance {positionClasses} w-[min(22rem,90vw)] max-w-md"
    role="dialog"
    aria-modal="true"
    aria-labelledby="guidance-title"
    aria-describedby="guidance-message"
  >
    <div
      class="guidance-card rounded-xl border border-press-border bg-press-surface p-4 text-press-ui text-press-text shadow-press-overlay"
    >
      <div class="flex gap-3 mb-3">
        <div
          class="shrink-0 w-8 h-8 rounded-lg bg-press-accent-wash flex items-center justify-center"
          aria-hidden="true"
        >
          <Info class="w-4 h-4 text-press-accent-text" />
        </div>
        <div>
          <h3 id="guidance-title" class="font-medium text-press-text mb-0.5">Tip</h3>
          <p
            id="guidance-message"
            class="font-prose text-press-text text-press-body leading-relaxed"
          >
            {config.message}
          </p>
        </div>
      </div>
      <div class="flex items-center justify-between gap-3 pt-2 border-t border-press-border">
        <button
          type="button"
          onclick={disableTips}
          class="flex items-center gap-1.5 text-press-eyebrow text-press-muted hover:text-press-text transition-colors"
          title="Don't show tips again"
        >
          <EyeOff class="w-3.5 h-3.5" />
          Disable tips
        </button>
        <button
          type="button"
          onclick={dismiss}
          class="flex items-center gap-1.5 px-3 py-1.5 text-press-eyebrow font-medium bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors"
        >
          <Check class="w-3.5 h-3.5" />
          Got it
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .guidance-card {
    animation: guidance-appear 0.2s ease-out forwards;
  }
  @keyframes guidance-appear {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
