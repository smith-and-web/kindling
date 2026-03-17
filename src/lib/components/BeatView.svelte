<script lang="ts">
  import {
    ChevronRight,
    ChevronDown,
    Loader2,
    Plus,
    Lock,
    GripVertical,
    MoreVertical,
  } from "lucide-svelte";
  import type { Beat } from "../types";
  import { ui } from "../stores/ui.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import NovelEditor from "./NovelEditor.svelte";

  let {
    beats,
    isLocked = false,
    addingBeat = $bindable(false),
    newBeatContent = $bindable(""),
    creatingBeat = false,
    dragOverBeatId = null,
    hoveredBeatId = $bindable<string | null>(null),
    localSaveStatus = "idle",
    novelEditorRef = $bindable(null),
    onToggleBeat,
    onStartAddingBeat,
    onCreateBeat,
    onNewBeatKeydown,
    onEditorUpdate,
    onBeatDragHandleMouseDown,
    getBeatContextMenuItems,
    getBeatWordCount,
    registerBeatRef,
    stripHtml,
  }: {
    beats: Beat[];
    isLocked?: boolean;
    addingBeat: boolean;
    newBeatContent: string;
    creatingBeat: boolean;
    dragOverBeatId: string | null;
    hoveredBeatId: string | null;
    localSaveStatus: "idle" | "saving" | "error";
    novelEditorRef: { getSplitBeforeParagraph: () => number | null } | null;
    onToggleBeat: (beatId: string) => void;
    onStartAddingBeat: () => void;
    onCreateBeat: () => void;
    onNewBeatKeydown: (e: KeyboardEvent) => void;
    onEditorUpdate: (beatId: string) => (html: string) => void;
    onBeatDragHandleMouseDown: (e: MouseEvent, beatId: string) => void;
    getBeatContextMenuItems: (beat: Beat) => Array<{
      label: string;
      action: () => void | Promise<void>;
      danger?: boolean;
      disabled?: boolean;
      divider?: boolean;
    }>;
    getBeatWordCount: (prose: string) => number;
    registerBeatRef: (node: HTMLElement, beatId: string) => { destroy: () => void };
    stripHtml: (html: string) => string;
  } = $props();

  let beatContextMenu: { beat: Beat; x: number; y: number } | null = $state(null);
</script>

<section>
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-sm font-semibold text-text-primary uppercase tracking-wide">Beats</h2>
    {#if beats.length > 0 && !addingBeat && !isLocked}
      <button
        onclick={onStartAddingBeat}
        class="flex items-center gap-1 text-text-secondary hover:text-text-primary transition-colors text-sm"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>Add Beat</span>
      </button>
    {/if}
  </div>
  {#if beats.length > 0}
    <div class="space-y-4">
      {#each beats as beat, index}
        {@const isExpanded = ui.expandedBeatId === beat.id}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <article
          data-drag-beat={beat.id}
          data-testid="beat-item"
          class="bg-bg-panel rounded-lg overflow-hidden select-none relative"
          class:ring-2={dragOverBeatId === beat.id}
          class:ring-accent={dragOverBeatId === beat.id}
          use:registerBeatRef={beat.id}
          onmouseenter={() => (hoveredBeatId = beat.id)}
          onmouseleave={() => (hoveredBeatId = null)}
        >
          <!-- Beat Header -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="w-full bg-beat-header px-4 py-2 flex items-center gap-2 hover:bg-beat-header/80 transition-colors cursor-pointer"
            oncontextmenu={(e) => {
              if (isLocked) return;
              beatContextMenu = { beat, x: e.clientX, y: e.clientY };
              e.preventDefault();
            }}
          >
            {#if !isLocked}
              <div
                data-testid="beat-drag-handle"
                onmousedown={(e) => onBeatDragHandleMouseDown(e, beat.id)}
                class="cursor-grab active:cursor-grabbing p-0.5 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
                class:opacity-0={hoveredBeatId !== beat.id}
                class:opacity-100={hoveredBeatId === beat.id}
                role="button"
                tabindex="-1"
                aria-label="Drag to reorder"
              >
                <GripVertical class="w-3.5 h-3.5" />
              </div>
            {/if}
            <button
              data-testid="beat-header"
              onclick={() => onToggleBeat(beat.id)}
              class="flex-1 flex items-center gap-3 text-left min-w-0"
            >
              <span class="text-text-secondary shrink-0">
                {#if isExpanded}
                  <ChevronDown class="w-4 h-4" />
                {:else}
                  <ChevronRight class="w-4 h-4" />
                {/if}
              </span>
              <span
                class="w-6 h-6 rounded-full bg-accent text-white text-xs font-medium flex items-center justify-center shrink-0"
              >
                {index + 1}
              </span>
              <p class="text-text-primary text-sm font-medium flex-1 truncate">
                {beat.content}
              </p>
              {#if beat.prose}
                <span class="text-xs text-text-secondary shrink-0" title="Word count">
                  {getBeatWordCount(beat.prose)}w
                </span>
              {/if}
            </button>
            {#if !isLocked}
              <button
                data-testid="beat-menu-button"
                onclick={(e) => {
                  beatContextMenu = { beat, x: e.clientX, y: e.clientY };
                }}
                class="p-1 text-text-secondary hover:text-text-primary transition-opacity shrink-0"
                class:opacity-0={hoveredBeatId !== beat.id}
                class:opacity-100={hoveredBeatId === beat.id}
                aria-label="Beat menu"
              >
                <MoreVertical class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>

          <!-- Expanded Beat Content -->
          {#if isExpanded}
            <div class="border-t border-bg-card relative" style="height: 50rem;">
              <NovelEditor
                bind:this={novelEditorRef}
                content={beat.prose || ""}
                placeholder={isLocked ? "Scene is locked" : "Write your prose for this beat..."}
                readonly={isLocked}
                saveStatus={localSaveStatus}
                onUpdate={onEditorUpdate(beat.id)}
              />
            </div>
          {:else if beat.prose}
            <div class="px-4 py-3 border-t border-bg-card">
              <p
                class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap line-clamp-3"
              >
                {stripHtml(beat.prose)}
              </p>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {:else if !addingBeat && !isLocked}
    <button
      onclick={onStartAddingBeat}
      class="w-full flex items-center justify-center gap-2 px-4 py-8 rounded-lg border border-dashed border-bg-card text-text-secondary hover:text-text-primary hover:border-accent transition-colors"
    >
      <Plus class="w-4 h-4" />
      <span class="text-sm">Add Your First Beat</span>
    </button>
  {:else if !addingBeat && isLocked}
    <div
      class="w-full flex items-center justify-center gap-2 px-4 py-8 rounded-lg border border-dashed border-bg-card text-text-secondary/50"
    >
      <Lock class="w-4 h-4" />
      <span class="text-sm">Scene is locked</span>
    </div>
  {/if}

  <!-- Add Beat Input -->
  {#if addingBeat && !isLocked}
    <div class="mt-4 bg-bg-panel rounded-lg p-4">
      <input
        type="text"
        class="w-full bg-bg-card rounded-lg px-4 py-3 text-text-primary text-sm border border-accent focus:outline-none"
        placeholder="Describe what happens in this beat..."
        bind:value={newBeatContent}
        onkeydown={onNewBeatKeydown}
        disabled={creatingBeat}
      />
      <div class="flex items-center justify-between mt-3">
        <p class="text-text-secondary text-xs">Press Enter to create, Escape to cancel</p>
        <div class="flex gap-2">
          <button
            onclick={() => {
              addingBeat = false;
              newBeatContent = "";
            }}
            class="px-3 py-1.5 text-text-secondary hover:text-text-primary text-sm transition-colors"
            disabled={creatingBeat}
          >
            Cancel
          </button>
          <button
            onclick={onCreateBeat}
            disabled={creatingBeat || !newBeatContent.trim()}
            class="px-3 py-1.5 bg-accent text-white text-sm rounded hover:bg-accent/80 transition-colors disabled:opacity-50"
          >
            {#if creatingBeat}
              <Loader2 class="w-4 h-4 animate-spin" />
            {:else}
              Create Beat
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>

{#if beatContextMenu}
  <ContextMenu
    items={getBeatContextMenuItems(beatContextMenu.beat)}
    x={beatContextMenu.x}
    y={beatContextMenu.y}
    onClose={() => (beatContextMenu = null)}
  />
{/if}
