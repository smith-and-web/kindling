<script lang="ts">
  import { FileText, ChevronRight, ChevronDown, Loader2, Plus, Pencil, Lock } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, tick } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Beat } from "../types";
  import NovelEditor from "./NovelEditor.svelte";
  import Tooltip from "./Tooltip.svelte";

  // Check if scene is locked (either directly or via parent chapter)
  const isLocked = $derived(
    currentProject.currentScene?.locked || currentProject.currentChapter?.locked || false
  );

  // Refs for beat articles to scroll into view
  let beatRefs = new SvelteMap<string, HTMLElement>();

  // Action to register beat element refs
  function registerBeatRef(node: HTMLElement, beatId: string) {
    beatRefs.set(beatId, node);
    return {
      destroy() {
        beatRefs.delete(beatId);
      },
    };
  }

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let pendingSaveBeatId: string | null = null;
  let synopsisSaveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Synopsis editing state
  let editingSynopsis = $state(false);
  let synopsisText = $state("");
  let synopsisSaving = $state(false);

  // New beat state
  let addingBeat = $state(false);
  let newBeatContent = $state("");
  let creatingBeat = $state(false);

  function syncPendingProse(beatId: string) {
    const pendingProse = pendingProseUpdates.get(beatId);
    if (pendingProse !== undefined) {
      currentProject.updateBeatProse(beatId, pendingProse);
      pendingProseUpdates.delete(beatId);
    }
  }

  function flushPendingSave(beatId?: string) {
    const targetBeatId = beatId ?? pendingSaveBeatId;
    if (!targetBeatId) return;

    if (saveTimeout && pendingSaveBeatId === targetBeatId) {
      clearTimeout(saveTimeout);
      saveTimeout = null;
      pendingSaveBeatId = null;
    }

    const draft = draftProse.get(targetBeatId);
    if (draft !== undefined) {
      saveBeatProse(targetBeatId, draft);
    }
  }

  async function toggleBeat(beatId: string) {
    if (ui.expandedBeatId === beatId) {
      flushPendingSave(beatId);
      // Collapsing - sync any pending prose updates to the store
      syncPendingProse(beatId);
      ui.setExpandedBeat(null);
    } else {
      // If we're switching from another beat, sync its pending updates first
      if (ui.expandedBeatId) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
      }
      ui.setExpandedBeat(beatId);
      // Wait for DOM to update, then scroll the beat into view
      await tick();
      const beatElement = beatRefs.get(beatId);
      if (beatElement) {
        beatElement.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
  }

  // Track pending prose updates to sync to store when beat is collapsed
  let pendingProseUpdates = new SvelteMap<string, string>();
  let draftProse = new SvelteMap<string, string>();

  // Local save status to avoid global store updates causing re-renders
  let localSaveStatus = $state<"idle" | "saving" | "error">("idle");

  async function saveBeatProse(beatId: string, prose: string) {
    localSaveStatus = "saving";
    try {
      await invoke("save_beat_prose", { beatId, prose });
      if (!currentProject.beats.some((beat) => beat.id === beatId)) {
        draftProse.delete(beatId);
        localSaveStatus = "idle";
        return;
      }
      // Don't update the store while editing - it causes re-renders and flashing
      // Instead, track the update and sync when the beat is collapsed
      pendingProseUpdates.set(beatId, prose);
      draftProse.delete(beatId);

      if (ui.expandedBeatId !== beatId) {
        currentProject.updateBeatProse(beatId, prose);
        pendingProseUpdates.delete(beatId);
      }
      // Keep showing "saving" for 1 more second so user sees the indicator
      setTimeout(() => {
        localSaveStatus = "idle";
      }, 1000);
    } catch (e) {
      console.error("Failed to save beat prose:", e);
      localSaveStatus = "error";
    }
  }

  function handleProseInput(beatId: string, value: string) {
    // Debounce save by 500ms
    draftProse.set(beatId, value);
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    pendingSaveBeatId = beatId;
    saveTimeout = setTimeout(() => {
      saveTimeout = null;
      pendingSaveBeatId = null;
      const draft = draftProse.get(beatId);
      if (draft !== undefined) {
        saveBeatProse(beatId, draft);
      }
    }, 500);
  }

  function handleEditorUpdate(beatId: string) {
    return (html: string) => {
      handleProseInput(beatId, html);
    };
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (ui.expandedBeatId) {
        flushPendingSave(ui.expandedBeatId);
        syncPendingProse(ui.expandedBeatId);
        ui.setExpandedBeat(null);
      }
      if (editingSynopsis) {
        editingSynopsis = false;
      }
      if (addingBeat) {
        addingBeat = false;
        newBeatContent = "";
      }
    }
  }

  // Synopsis functions
  function startEditingSynopsis() {
    synopsisText = currentProject.currentScene?.synopsis || "";
    editingSynopsis = true;
  }

  async function saveSynopsis() {
    if (!currentProject.currentScene) return;
    synopsisSaving = true;
    try {
      const synopsis = synopsisText.trim() || null;
      await invoke("save_scene_synopsis", {
        sceneId: currentProject.currentScene.id,
        synopsis,
      });
      currentProject.updateSceneSynopsis(currentProject.currentScene.id, synopsis);
      editingSynopsis = false;
    } catch (e) {
      console.error("Failed to save synopsis:", e);
    } finally {
      synopsisSaving = false;
    }
  }

  function handleSynopsisInput(value: string) {
    synopsisText = value;
    // Debounce auto-save
    if (synopsisSaveTimeout) {
      clearTimeout(synopsisSaveTimeout);
    }
    synopsisSaveTimeout = setTimeout(() => {
      saveSynopsis();
    }, 1000);
  }

  // Beat functions
  function startAddingBeat() {
    addingBeat = true;
    newBeatContent = "";
  }

  async function createBeat() {
    if (!currentProject.currentScene || !newBeatContent.trim()) return;
    creatingBeat = true;
    try {
      const beat = await invoke<Beat>("create_beat", {
        sceneId: currentProject.currentScene.id,
        content: newBeatContent.trim(),
      });
      currentProject.addBeat(beat);
      addingBeat = false;
      newBeatContent = "";
      // Auto-expand the new beat for immediate editing
      ui.setExpandedBeat(beat.id);
    } catch (e) {
      console.error("Failed to create beat:", e);
    } finally {
      creatingBeat = false;
    }
  }

  function handleNewBeatKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      createBeat();
    }
  }

  // Strip HTML tags for plain text preview
  function stripHtml(html: string): string {
    return html.replace(/<[^>]*>/g, "").trim();
  }

  let lastSceneId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (lastSceneId && sceneId !== lastSceneId) {
      flushPendingSave(ui.expandedBeatId ?? undefined);
      if (ui.expandedBeatId) {
        syncPendingProse(ui.expandedBeatId);
      }
      pendingProseUpdates.clear();
      draftProse.clear();
      ui.setExpandedBeat(null);
    }
    lastSceneId = sceneId;
  });

  onDestroy(() => {
    flushPendingSave(ui.expandedBeatId ?? undefined);
    if (ui.expandedBeatId) {
      syncPendingProse(ui.expandedBeatId);
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div data-testid="scene-panel" class="flex-1 flex flex-col h-full overflow-hidden">
  {#if currentProject.currentScene}
    {@const scene = currentProject.currentScene}
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-8">
        <!-- Scene Title -->
        <header class="mb-8">
          <div class="flex items-center gap-3">
            <h1
              data-testid="scene-title"
              class="text-3xl font-heading font-semibold text-text-primary"
            >
              {scene.title}
            </h1>
            {#if isLocked}
              <span
                class="flex items-center gap-1 px-2 py-1 bg-amber-500/10 text-amber-500 rounded-lg text-sm"
              >
                <Lock class="w-4 h-4" />
                Locked
              </span>
            {/if}
          </div>
          {#if currentProject.currentChapter}
            <p class="text-text-secondary text-sm mt-1">
              {currentProject.currentChapter.title}
            </p>
          {/if}
        </header>

        <!-- Locked Banner -->
        {#if isLocked}
          <div class="mb-8 px-4 py-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
            <div class="flex items-center gap-2 text-amber-500">
              <Lock class="w-4 h-4" />
              <span class="font-medium">This scene is locked</span>
            </div>
            <p class="text-text-secondary text-sm mt-1">
              {#if currentProject.currentChapter?.locked}
                The parent chapter is locked. Unlock the chapter to edit this scene.
              {:else}
                Unlock this scene from the sidebar to make changes.
              {/if}
            </p>
          </div>
        {/if}

        <!-- Synopsis -->
        <section class="mb-8">
          <div class="flex items-center justify-between mb-2">
            <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wide">
              Synopsis
            </h2>
            {#if scene.synopsis && !editingSynopsis && !isLocked}
              <Tooltip text="Edit synopsis" position="left">
                <button
                  onclick={startEditingSynopsis}
                  class="text-text-secondary hover:text-text-primary transition-colors p-1"
                  aria-label="Edit synopsis"
                >
                  <Pencil class="w-3.5 h-3.5" />
                </button>
              </Tooltip>
            {/if}
          </div>
          {#if editingSynopsis && !isLocked}
            <div class="relative">
              <textarea
                class="w-full min-h-[100px] bg-bg-card rounded-lg p-4 text-text-primary font-prose italic leading-relaxed resize-y border border-accent focus:outline-none"
                placeholder="Write a brief synopsis for this scene..."
                bind:value={synopsisText}
                oninput={(e) => handleSynopsisInput(e.currentTarget.value)}
              ></textarea>
              {#if synopsisSaving}
                <div
                  class="absolute bottom-3 right-3 flex items-center gap-1.5 text-text-secondary/50"
                >
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  <span class="text-xs">Saving...</span>
                </div>
              {/if}
            </div>
            <p class="text-text-secondary text-xs mt-2">
              Press Escape to close. Changes are saved automatically.
            </p>
          {:else if scene.synopsis}
            <div class="bg-bg-panel rounded-lg p-4 border-l-2 border-accent">
              <p class="text-text-primary font-prose italic">
                {scene.synopsis}
              </p>
            </div>
          {:else if !isLocked}
            <button
              onclick={startEditingSynopsis}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-bg-card text-text-secondary hover:text-text-primary hover:border-accent transition-colors"
            >
              <Plus class="w-4 h-4" />
              <span class="text-sm">Add Synopsis</span>
            </button>
          {:else}
            <div
              class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg border border-dashed border-bg-card text-text-secondary/50"
            >
              <Lock class="w-4 h-4" />
              <span class="text-sm">Scene is locked</span>
            </div>
          {/if}
        </section>

        <!-- Beats -->
        <section>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wide">Beats</h2>
            {#if currentProject.beats.length > 0 && !addingBeat && !isLocked}
              <button
                onclick={startAddingBeat}
                class="flex items-center gap-1 text-text-secondary hover:text-text-primary transition-colors text-sm"
              >
                <Plus class="w-3.5 h-3.5" />
                <span>Add Beat</span>
              </button>
            {/if}
          </div>
          {#if currentProject.beats.length > 0}
            <div class="space-y-4">
              {#each currentProject.beats as beat, index (beat.id)}
                {@const isExpanded = ui.expandedBeatId === beat.id}
                <article
                  class="bg-bg-panel rounded-lg overflow-hidden"
                  use:registerBeatRef={beat.id}
                >
                  <!-- Beat Header (clickable to expand) -->
                  <button
                    data-testid="beat-header"
                    onclick={() => toggleBeat(beat.id)}
                    class="w-full bg-beat-header px-4 py-2 flex items-center gap-3 hover:bg-beat-header/80 transition-colors cursor-pointer text-left"
                  >
                    <span class="text-text-secondary flex-shrink-0">
                      {#if isExpanded}
                        <ChevronDown class="w-4 h-4" />
                      {:else}
                        <ChevronRight class="w-4 h-4" />
                      {/if}
                    </span>
                    <span
                      class="w-6 h-6 rounded-full bg-accent text-white text-xs font-medium flex items-center justify-center flex-shrink-0"
                    >
                      {index + 1}
                    </span>
                    <p class="text-text-primary text-sm font-medium flex-1">
                      {beat.content}
                    </p>
                  </button>

                  <!-- Expanded Beat Content -->
                  {#if isExpanded}
                    <div class="border-t border-bg-card relative" style="height: 50rem;">
                      <NovelEditor
                        content={beat.prose || ""}
                        placeholder={isLocked
                          ? "Scene is locked"
                          : "Write your prose for this beat..."}
                        readonly={isLocked}
                        saveStatus={localSaveStatus}
                        onUpdate={handleEditorUpdate(beat.id)}
                      />
                    </div>
                  {:else if beat.prose}
                    <!-- Show preview of prose when collapsed -->
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
              onclick={startAddingBeat}
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
                onkeydown={handleNewBeatKeydown}
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
                    onclick={createBeat}
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

        <!-- Scene Prose (if exists and no beats) -->
        {#if scene.prose && currentProject.beats.length === 0}
          <section class="mt-8">
            <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wide mb-4">
              Content
            </h2>
            <div class="bg-bg-panel rounded-lg p-6">
              <p class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap">
                {scene.prose}
              </p>
            </div>
          </section>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Empty State -->
    <div
      data-testid="empty-state"
      class="flex-1 flex flex-col items-center justify-center text-text-secondary"
    >
      <FileText class="w-16 h-16 mb-4 opacity-50" strokeWidth={1.5} />
      <p class="text-lg">Select a scene to start writing</p>
      <p class="text-sm mt-1">Choose a scene from the sidebar to view its content</p>
    </div>
  {/if}
</div>
