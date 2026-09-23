<!--
  QuickStartDialog.svelte - In-app Quick Start guide

  Accessible from Help → Quick Start. Covers:
  - Import formats
  - Sidebar (chapters & scenes)
  - Scene panel (synopsis, beats, discovery notes)
  - References panel
-->
<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { shortcuts } from "../stores/shortcuts.svelte";
  import {
    BookOpen,
    ChevronDown,
    FileText,
    Kanban,
    MapPin,
    PenTool,
    User,
    Users,
    Zap,
  } from "lucide-svelte";

  let { onClose }: { onClose: () => void } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="dialog-scrim"
  role="dialog"
  aria-modal="true"
  aria-labelledby="quick-start-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface ka-dialog-default dialog-shell quick-start"
    data-testid="quick-start-dialog"
  >
    <DialogHeader
      title="Quick start"
      titleId="quick-start-title"
      {onClose}
      closeLabel="Close"
      closeTestId="quick-start-close"
    />

    <div class="ka-dialog-body quick-start-body">
      <!-- Import -->
      <section class="ka-group">
        <h3 class="ka-group-title">Import your outline</h3>
        <p class="font-press-ui text-press-base text-press-text mb-3 max-w-press-measure">
          kindling works with your existing outline. Import from the Start Screen or File → Import.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <Kanban class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span><strong class="text-press-text">Plottr</strong> — .pltr files</span>
          </li>
          <li class="flex items-start gap-2">
            <PenTool class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span><strong class="text-press-text">yWriter 7</strong> — .yw7 files</span>
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Markdown</strong> — Single .md file with # Chapter, ##
              Scene, - Beat structure</span
            >
          </li>
          <li class="flex items-start gap-2">
            <BookOpen class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Longform</strong> — Obsidian vault or index file</span
            >
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Scrivener 3</strong> — .scriv project bundles</span
            >
          </li>
        </ul>
      </section>

      <!-- Sidebar -->
      <section class="ka-group">
        <h3 class="ka-group-title">Sidebar — chapters and scenes</h3>
        <p class="font-press-ui text-press-base text-press-text mb-3 max-w-press-measure">
          The left sidebar shows your project structure. Click a chapter to expand or collapse its
          scenes. Click a scene to load it in the editor.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <ChevronDown class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span>Chapters group related scenes together</span>
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span>Scenes are your primary writing units — each has a synopsis and beats</span>
          </li>
          <li class="flex items-start gap-2">
            <span class="text-press-accent-text shrink-0">•</span>
            <span>Right-click chapters or scenes for more options (reorder, archive, lock)</span>
          </li>
        </ul>
      </section>

      <!-- Scene Panel -->
      <section class="ka-group">
        <h3 class="ka-group-title">Scene panel — synopsis and beats</h3>
        <p class="font-press-ui text-press-base text-press-text mb-3 max-w-press-measure">
          When you select a scene, the main area shows its synopsis and beats. Beats are the key
          story moments — expand each to write prose.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <Zap class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Synopsis</strong> — Brief overview of the scene (from your
              outline)</span
            >
          </li>
          <li class="flex items-start gap-2">
            <Zap class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Beats</strong> — Key moments; click to expand and write
              prose</span
            >
          </li>
          <li class="flex items-start gap-2">
            <span class="text-press-accent-text shrink-0">•</span>
            <span>Right-click beats to split, merge, or delete. Drag to reorder.</span>
          </li>
          <li class="flex items-start gap-2">
            <span class="text-press-accent-text shrink-0">•</span>
            <span
              ><strong class="text-press-text">Discovery Notes</strong
              >{#if shortcuts.label("toggle_discovery_notes")}
                ({shortcuts.label("toggle_discovery_notes")}){/if} — Capture ideas as you write; promote
              to beats when ready</span
            >
          </li>
        </ul>
      </section>

      <!-- References -->
      <section class="ka-group">
        <h3 class="ka-group-title">References panel</h3>
        <p class="font-press-ui text-press-base text-press-text mb-3 max-w-press-measure">
          The right panel shows characters and locations linked to the current scene. Use it to keep
          track of who appears where.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <User class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Characters</strong> — Who appears in this scene</span
            >
          </li>
          <li class="flex items-start gap-2">
            <MapPin class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span
              ><strong class="text-press-text">Locations</strong> — Where the scene takes place</span
            >
          </li>
          <li class="flex items-start gap-2">
            <Users class="w-4 h-4 shrink-0 mt-0.5 quick-start-icon" />
            <span>Link characters and locations from your outline, or add them in kindling</span>
          </li>
        </ul>
      </section>

      <!-- Tips -->
      <section class="ka-group">
        <h3 class="ka-group-title">Tips</h3>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li>
            <strong class="text-press-text">{shortcuts.label("export") || "Export"}</strong> — Export
            your project
          </li>
          <li>
            <strong class="text-press-text"
              >{shortcuts.label("toggle_discovery_notes") || "Discovery Notes"}</strong
            > — Toggle Discovery Notes in the scene panel
          </li>
          <li>
            <strong class="text-press-text">Snapshots</strong> — Create version checkpoints before big
            changes (sidebar)
          </li>
        </ul>
      </section>
    </div>

    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button">Got it</button>
    </footer>
  </div>
</div>

<style>
  .quick-start {
    height: min(720px, calc(100dvh - 48px));
  }
  .quick-start-body h3 {
    margin: 0 0 var(--space-3xs);
  }
  .quick-start-body p {
    margin: 0;
  }
  .quick-start-body :global(.quick-start-icon) {
    color: var(--color-text-muted);
  }
</style>
