<!--
  QuickStartDialog.svelte - In-app Quick Start guide

  Accessible from Help → Quick Start. Covers:
  - Import formats
  - Sidebar (chapters & scenes)
  - Scene panel (synopsis, beats, discovery notes)
  - References panel
-->
<script lang="ts">
  import {
    BookOpen,
    ChevronDown,
    FileText,
    Kanban,
    MapPin,
    PenTool,
    User,
    Users,
    X,
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
  class="fixed inset-0 bg-press-overlay flex items-center justify-center z-press-modal p-4"
  role="dialog"
  aria-modal="true"
  aria-labelledby="quick-start-title"
  tabindex="-1"
>
  <div
    class="app-dialog-surface bg-press-surface rounded-xl shadow-press-overlay max-w-2xl w-full max-h-[85vh] flex flex-col"
    data-testid="quick-start-dialog"
  >
    <div class="flex items-center justify-between p-6 border-b border-press-border shrink-0">
      <h2 id="quick-start-title" class="text-press-h3 font-heading font-semibold text-press-text">
        Quick Start
      </h2>
      <button
        onclick={onClose}
        class="p-2 text-press-muted hover:text-press-text hover:bg-press-sunken rounded-lg transition-colors"
        aria-label="Close"
      >
        <X class="w-5 h-5" />
      </button>
    </div>

    <div class="overflow-y-auto p-6 space-y-6">
      <!-- Import -->
      <section>
        <h3 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-3">
          Import Your Outline
        </h3>
        <p class="font-prose text-press-body text-press-text mb-3 max-w-press-measure">
          Kindling works with your existing outline. Import from the Start Screen or File → Import.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <Kanban class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span><strong class="text-press-text">Plottr</strong> — .pltr files</span>
          </li>
          <li class="flex items-start gap-2">
            <PenTool class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span><strong class="text-press-text">yWriter 7</strong> — .yw7 files</span>
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Markdown</strong> — Single .md file with # Chapter, ##
              Scene, - Beat structure</span
            >
          </li>
          <li class="flex items-start gap-2">
            <BookOpen class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Longform</strong> — Obsidian vault or index file</span
            >
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Scrivener 3</strong> — .scriv project bundles</span
            >
          </li>
        </ul>
      </section>

      <!-- Sidebar -->
      <section>
        <h3 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-3">
          Sidebar — Chapters & Scenes
        </h3>
        <p class="font-prose text-press-body text-press-text mb-3 max-w-press-measure">
          The left sidebar shows your project structure. Click a chapter to expand or collapse its
          scenes. Click a scene to load it in the editor.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <ChevronDown class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span>Chapters group related scenes together</span>
          </li>
          <li class="flex items-start gap-2">
            <FileText class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span>Scenes are your primary writing units — each has a synopsis and beats</span>
          </li>
          <li class="flex items-start gap-2">
            <span class="text-press-accent-text shrink-0">•</span>
            <span>Right-click chapters or scenes for more options (reorder, archive, lock)</span>
          </li>
        </ul>
      </section>

      <!-- Scene Panel -->
      <section>
        <h3 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-3">
          Scene Panel — Synopsis & Beats
        </h3>
        <p class="font-prose text-press-body text-press-text mb-3 max-w-press-measure">
          When you select a scene, the main area shows its synopsis and beats. Beats are the key
          story moments — expand each to write prose.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <Zap class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Synopsis</strong> — Brief overview of the scene (from your
              outline)</span
            >
          </li>
          <li class="flex items-start gap-2">
            <Zap class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
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
              ><strong class="text-press-text">Discovery Notes</strong> (⌘D) — Capture ideas as you write;
              promote to beats when ready</span
            >
          </li>
        </ul>
      </section>

      <!-- References -->
      <section>
        <h3 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-3">
          References Panel
        </h3>
        <p class="font-prose text-press-body text-press-text mb-3 max-w-press-measure">
          The right panel shows characters and locations linked to the current scene. Use it to keep
          track of who appears where.
        </p>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li class="flex items-start gap-2">
            <User class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Characters</strong> — Who appears in this scene</span
            >
          </li>
          <li class="flex items-start gap-2">
            <MapPin class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span
              ><strong class="text-press-text">Locations</strong> — Where the scene takes place</span
            >
          </li>
          <li class="flex items-start gap-2">
            <Users class="w-4 h-4 text-press-accent-text shrink-0 mt-0.5" />
            <span>Link characters and locations from your outline, or add them in Kindling</span>
          </li>
        </ul>
      </section>

      <!-- Tips -->
      <section>
        <h3 class="text-press-ui font-semibold text-press-text uppercase tracking-wide mb-3">
          Tips
        </h3>
        <ul class="space-y-2 text-press-ui text-press-muted">
          <li><strong class="text-press-text">⌘E</strong> — Export your project</li>
          <li>
            <strong class="text-press-text">⌘D</strong> — Toggle Discovery Notes in the scene panel
          </li>
          <li>
            <strong class="text-press-text">Snapshots</strong> — Create version checkpoints before big
            changes (sidebar)
          </li>
        </ul>
      </section>
    </div>

    <div class="p-6 border-t border-press-border shrink-0">
      <button
        onclick={onClose}
        class="w-full py-2 px-4 bg-press-accent hover:bg-press-accent-text text-press-on-accent font-medium rounded-lg transition-colors"
      >
        Got it
      </button>
    </div>
  </div>
</div>
