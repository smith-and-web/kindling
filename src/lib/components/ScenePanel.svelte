<script lang="ts">
  import { currentProject } from "../stores/project.svelte";
</script>

<div class="flex-1 flex flex-col h-full overflow-hidden">
  {#if currentProject.currentScene}
    {@const scene = currentProject.currentScene}
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-8">
        <!-- Scene Title -->
        <header class="mb-8">
          <h1 class="text-3xl font-heading font-semibold text-text-primary">
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
                <article class="bg-bg-panel rounded-lg overflow-hidden">
                  <!-- Beat Header -->
                  <div class="bg-beat-header px-4 py-2 flex items-center gap-3">
                    <span
                      class="w-6 h-6 rounded-full bg-accent text-white text-xs font-medium flex items-center justify-center flex-shrink-0"
                    >
                      {index + 1}
                    </span>
                    <p class="text-text-primary text-sm font-medium">
                      {beat.content}
                    </p>
                  </div>

                  <!-- Beat Prose (if exists) -->
                  {#if beat.prose}
                    <div class="px-4 py-4 border-t border-bg-card">
                      <p class="text-text-primary font-prose leading-relaxed whitespace-pre-wrap">
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
    <div class="flex-1 flex flex-col items-center justify-center text-text-secondary">
      <svg class="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <p class="text-lg">Select a scene to start writing</p>
      <p class="text-sm mt-1">Choose a scene from the sidebar to view its content</p>
    </div>
  {/if}
</div>
