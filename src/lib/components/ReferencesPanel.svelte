<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Character, Location } from "../types";

  type Tab = "characters" | "locations";

  let activeTab = $state<Tab>("characters");
  let loading = $state(false);
  let expandedId = $state<string | null>(null);

  async function loadReferences() {
    if (!currentProject.value) return;

    loading = true;
    try {
      const [characters, locations] = await Promise.all([
        invoke<Character[]>("get_characters", { projectId: currentProject.value.id }),
        invoke<Location[]>("get_locations", { projectId: currentProject.value.id }),
      ]);
      currentProject.setCharacters(characters);
      currentProject.setLocations(locations);
    } catch (e) {
      console.error("Failed to load references:", e);
    } finally {
      loading = false;
    }
  }

  function toggleExpanded(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function formatAttributes(attrs: Record<string, string>): [string, string][] {
    return Object.entries(attrs).filter(([key]) => key !== "notes");
  }

  function getNotes(attrs: Record<string, string>): string | null {
    return attrs.notes || null;
  }

  function toggleReferencesPanel() {
    ui.referencesPanelCollapsed = !ui.referencesPanelCollapsed;
  }

  $effect(() => {
    if (currentProject.value) {
      loadReferences();
    }
  });
</script>

<aside
  class="w-72 bg-bg-panel border-l border-bg-card flex flex-col h-full transition-all"
  class:w-0={ui.referencesPanelCollapsed}
  class:overflow-hidden={ui.referencesPanelCollapsed}
>
  <!-- Header with tabs -->
  <div class="border-b border-bg-card">
    <div class="flex items-center justify-between px-4 py-2">
      <h2 class="text-sm font-heading font-medium text-text-primary">References</h2>
      <button
        onclick={toggleReferencesPanel}
        class="text-text-secondary hover:text-text-primary p-1"
        aria-label="Collapse references panel"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 5l7 7-7 7M5 5l7 7-7 7"
          />
        </svg>
      </button>
    </div>

    <!-- Tabs -->
    <div class="flex border-t border-bg-card">
      <button
        onclick={() => (activeTab = "characters")}
        class="flex-1 px-4 py-2 text-sm font-medium transition-colors"
        class:text-accent={activeTab === "characters"}
        class:border-b-2={activeTab === "characters"}
        class:border-accent={activeTab === "characters"}
        class:text-text-secondary={activeTab !== "characters"}
      >
        Characters ({currentProject.characters.length})
      </button>
      <button
        onclick={() => (activeTab = "locations")}
        class="flex-1 px-4 py-2 text-sm font-medium transition-colors"
        class:text-accent={activeTab === "locations"}
        class:border-b-2={activeTab === "locations"}
        class:border-accent={activeTab === "locations"}
        class:text-text-secondary={activeTab !== "locations"}
      >
        Locations ({currentProject.locations.length})
      </button>
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary text-sm">Loading...</span>
      </div>
    {:else if activeTab === "characters"}
      {#if currentProject.characters.length === 0}
        <div class="flex items-center justify-center p-4">
          <span class="text-text-secondary text-sm">No characters</span>
        </div>
      {:else}
        <div class="space-y-2">
          {#each currentProject.characters as character (character.id)}
            {@const isExpanded = expandedId === character.id}
            {@const attributes = formatAttributes(character.attributes)}
            {@const notes = getNotes(character.attributes)}
            <div class="bg-bg-card rounded-lg overflow-hidden">
              <button
                onclick={() => toggleExpanded(character.id)}
                class="w-full flex items-center gap-3 p-3 text-left hover:bg-beat-header transition-colors"
              >
                <!-- Character icon -->
                <div
                  class="w-8 h-8 rounded-full bg-accent/20 flex items-center justify-center flex-shrink-0"
                >
                  <svg
                    class="w-4 h-4 text-accent"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                    />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-text-primary font-medium text-sm truncate">{character.name}</p>
                  {#if character.description}
                    <p class="text-text-secondary text-xs truncate">{character.description}</p>
                  {/if}
                </div>
                <svg
                  class="w-4 h-4 text-text-secondary transition-transform flex-shrink-0"
                  class:rotate-180={isExpanded}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 9l-7 7-7-7"
                  />
                </svg>
              </button>

              {#if isExpanded}
                <div class="px-3 pb-3 border-t border-bg-panel">
                  {#if notes}
                    <p class="text-text-primary text-sm mt-3 leading-relaxed">{notes}</p>
                  {/if}

                  {#if attributes.length > 0}
                    <div class="mt-3 space-y-1.5">
                      {#each attributes as [key, value] (key)}
                        <div class="flex gap-2 text-xs">
                          <span class="text-text-secondary font-medium shrink-0">{key}:</span>
                          <span class="text-text-primary">{value}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {:else if activeTab === "locations"}
      {#if currentProject.locations.length === 0}
        <div class="flex items-center justify-center p-4">
          <span class="text-text-secondary text-sm">No locations</span>
        </div>
      {:else}
        <div class="space-y-2">
          {#each currentProject.locations as location (location.id)}
            {@const isExpanded = expandedId === location.id}
            {@const attributes = formatAttributes(location.attributes)}
            {@const notes = getNotes(location.attributes)}
            <div class="bg-bg-card rounded-lg overflow-hidden">
              <button
                onclick={() => toggleExpanded(location.id)}
                class="w-full flex items-center gap-3 p-3 text-left hover:bg-beat-header transition-colors"
              >
                <!-- Location icon -->
                <div
                  class="w-8 h-8 rounded-full bg-spark-gold/20 flex items-center justify-center flex-shrink-0"
                >
                  <svg
                    class="w-4 h-4 text-spark-gold"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
                    />
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-text-primary font-medium text-sm truncate">{location.name}</p>
                  {#if location.description}
                    <p class="text-text-secondary text-xs truncate">{location.description}</p>
                  {/if}
                </div>
                <svg
                  class="w-4 h-4 text-text-secondary transition-transform flex-shrink-0"
                  class:rotate-180={isExpanded}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 9l-7 7-7-7"
                  />
                </svg>
              </button>

              {#if isExpanded}
                <div class="px-3 pb-3 border-t border-bg-panel">
                  {#if notes}
                    <p class="text-text-primary text-sm mt-3 leading-relaxed">{notes}</p>
                  {/if}

                  {#if attributes.length > 0}
                    <div class="mt-3 space-y-1.5">
                      {#each attributes as [key, value] (key)}
                        <div class="flex gap-2 text-xs">
                          <span class="text-text-secondary font-medium shrink-0">{key}:</span>
                          <span class="text-text-primary">{value}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</aside>

<!-- Collapsed panel toggle -->
{#if ui.referencesPanelCollapsed}
  <button
    onclick={toggleReferencesPanel}
    class="fixed right-0 top-1/2 -translate-y-1/2 bg-bg-panel p-2 rounded-l-lg text-text-secondary hover:text-text-primary z-10"
    aria-label="Expand references panel"
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
      />
    </svg>
  </button>
{/if}
