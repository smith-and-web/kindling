<script lang="ts">
  import { FileText, ChevronRight, ChevronDown, Loader2 } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function toggleBeat(beatId: string) {
    if (ui.expandedBeatId === beatId) {
      ui.setExpandedBeat(null);
    } else {
      ui.setExpandedBeat(beatId);
    }
  }

  async function saveBeatProse(beatId: string, prose: string) {
    ui.setBeatSaveStatus("saving");
    try {
      await invoke("save_beat_prose", { beatId, prose });
      currentProject.updateBeatProse(beatId, prose);
      ui.setBeatSaveStatus("saved");
      // Reset to idle after 2 seconds
      setTimeout(() => {
        if (ui.beatSaveStatus === "saved") {
          ui.setBeatSaveStatus("idle");
        }
      }, 2000);
    } catch (e) {
      console.error("Failed to save beat prose:", e);
      ui.setBeatSaveStatus("error");
    }
  }

  function handleProseInput(beatId: string, value: string) {
    // Debounce save by 500ms
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = setTimeout(() => {
      saveBeatProse(beatId, value);
    }, 500);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && ui.expandedBeatId) {
      ui.setExpandedBeat(null);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div data-testid="scene-panel" class="flex-1 flex flex-col h-full overflow-hidden">
  {#if currentProject.currentScene}
    {@const scene = currentProject.currentScene}
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-8">
        <!-- Scene Title -->
        <header class="mb-8">
          <h1
            data-testid="scene-title"
            class="text-3xl font-heading font-semibold text-text-primary"
          >
            {scene.title}
          </h1>
          {#if currentProject.currentChapter}
            <p class="text-text-secondary text-sm mt-1">
              {currentProject.currentChapter.title}
            </p>
          {/if}
        </header>

        <!-- Synopsis -->
        {#if scene.synopsis}
          <section class="mb-8">
            <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wide mb-2">
              Synopsis
            </h2>
            <div class="bg-bg-panel rounded-lg p-4 border-l-2 border-accent">
              <p class="text-text-primary font-prose italic">
                {scene.synopsis}
              </p>
            </div>
          </section>
        {/if}

        <!-- Beats -->
        {#if currentProject.beats.length > 0}
          <section>
            <h2 class="text-sm font-medium text-text-secondary uppercase tracking-wide mb-4">
              Beats
            </h2>
            <div class="space-y-4">
              {#each currentProject.beats as beat, index (beat.id)}
                {@const isExpanded = ui.expandedBeatId === beat.id}
                <article class="bg-bg-panel rounded-lg overflow-hidden">
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
                    <div class="px-4 py-4 border-t border-bg-card">
                      <div class="relative">
                        <textarea
                          data-testid="beat-prose-textarea"
                          class="w-full min-h-[200px] bg-bg-card rounded-lg p-4 text-text-primary font-prose leading-relaxed resize-y border border-bg-card focus:border-accent focus:outline-none"
                          placeholder="Write your prose for this beat..."
                          value={beat.prose || ""}
                          oninput={(e) => handleProseInput(beat.id, e.currentTarget.value)}
                        ></textarea>
                        <!-- Subtle save indicator in bottom right -->
                        {#if ui.beatSaveStatus === "saving"}
                          <div
                            data-testid="save-indicator"
                            class="absolute bottom-3 right-3 text-text-secondary/50"
                          >
                            <Loader2 class="w-4 h-4 animate-spin" />
                          </div>
                        {:else if ui.beatSaveStatus === "error"}
                          <div
                            data-testid="save-indicator"
                            class="absolute bottom-3 right-3 text-red-500/70 text-xs"
                          >
                            Error saving
                          </div>
                        {/if}
                      </div>
                      <p class="text-text-secondary text-xs mt-2">
                        Press Escape to collapse. Changes are saved automatically.
                      </p>
                    </div>
                  {:else if beat.prose}
                    <!-- Show preview of prose when collapsed -->
                    <div class="px-4 py-3 border-t border-bg-card">
                      <p
                        class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap line-clamp-3"
                      >
                        {beat.prose}
                      </p>
                    </div>
                  {/if}
                </article>
              {/each}
            </div>
          </section>
        {:else}
          <section class="text-center py-8">
            <p class="text-text-secondary">No beats in this scene</p>
          </section>
        {/if}

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
