<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Character, Location } from "../types";

  type Tab = "characters" | "locations";

  let activeTab = $state<Tab>("characters");
  let loading = $state(false);
  let expandedIds = new SvelteSet<string>();
  let isResizing = $state(false);
  let draggedId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);
  let isDragging = $state(false);

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
    if (expandedIds.has(id)) {
      expandedIds.delete(id);
    } else {
      expandedIds.add(id);
    }
  }

  function collapseAll() {
    expandedIds.clear();
  }

  function sortAlphabetically() {
    if (activeTab === "characters") {
      const sorted = [...currentProject.characters].sort((a, b) => a.name.localeCompare(b.name));
      currentProject.setCharacters(sorted);
    } else {
      const sorted = [...currentProject.locations].sort((a, b) => a.name.localeCompare(b.name));
      currentProject.setLocations(sorted);
    }
  }

  function formatAttributes(attrs: Record<string, string>): [string, string][] {
    return Object.entries(attrs).filter(([key]) => key !== "notes");
  }

  function getNotes(attrs: Record<string, string>): string | null {
    return attrs.notes || null;
  }

  function toggleReferencesPanel() {
    ui.toggleReferencesPanel();
  }

  // Pointer-based drag and drop (more reliable than HTML5 drag API in webviews)
  let draggedElement: globalThis.HTMLElement | null = null;
  let currentDragOverElement: globalThis.HTMLElement | null = null;

  function onDragHandleMouseDown(e: globalThis.MouseEvent, id: string) {
    e.preventDefault();
    e.stopPropagation();
    draggedId = id;
    isDragging = true;

    // Find the dragged element by traversing up from the handle
    const target = e.currentTarget as globalThis.HTMLElement;
    draggedElement = target.closest("[data-drag-item]") as globalThis.HTMLElement;
    if (draggedElement) {
      draggedElement.style.opacity = "0.5";
    }

    document.addEventListener("mousemove", onDragMouseMove);
    document.addEventListener("mouseup", onDragMouseUp);
    document.body.style.cursor = "grabbing";
    document.body.style.userSelect = "none";
  }

  function onDragMouseMove(e: globalThis.MouseEvent) {
    if (!isDragging || !draggedId) return;

    // Clear previous hover styling
    if (currentDragOverElement) {
      currentDragOverElement.style.outline = "";
    }

    // Find which item we're hovering over
    const itemElements = document.querySelectorAll("[data-drag-item]");
    let foundElement: globalThis.HTMLElement | null = null;
    let foundId: string | null = null;

    for (const el of itemElements) {
      const rect = el.getBoundingClientRect();
      const itemId = el.getAttribute("data-drag-item");
      if (itemId && itemId !== draggedId && e.clientY >= rect.top && e.clientY <= rect.bottom) {
        foundId = itemId;
        foundElement = el as globalThis.HTMLElement;
        break;
      }
    }

    dragOverId = foundId;
    currentDragOverElement = foundElement;

    // Style the hover target
    if (foundElement) {
      foundElement.style.outline = "2px solid var(--color-accent)";
    }
  }

  function onDragMouseUp() {
    document.removeEventListener("mousemove", onDragMouseMove);
    document.removeEventListener("mouseup", onDragMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";

    // Clear visual styling
    if (draggedElement) {
      draggedElement.style.opacity = "";
    }
    if (currentDragOverElement) {
      currentDragOverElement.style.outline = "";
    }

    if (draggedId && dragOverId && draggedId !== dragOverId) {
      // Perform the reorder
      if (activeTab === "characters") {
        const items = [...currentProject.characters];
        const fromIndex = items.findIndex((c) => c.id === draggedId);
        const toIndex = items.findIndex((c) => c.id === dragOverId);
        if (fromIndex !== -1 && toIndex !== -1) {
          const [moved] = items.splice(fromIndex, 1);
          items.splice(toIndex, 0, moved);
          currentProject.setCharacters(items);
        }
      } else {
        const items = [...currentProject.locations];
        const fromIndex = items.findIndex((l) => l.id === draggedId);
        const toIndex = items.findIndex((l) => l.id === dragOverId);
        if (fromIndex !== -1 && toIndex !== -1) {
          const [moved] = items.splice(fromIndex, 1);
          items.splice(toIndex, 0, moved);
          currentProject.setLocations(items);
        }
      }
    }

    isDragging = false;
    draggedId = null;
    dragOverId = null;
    draggedElement = null;
    currentDragOverElement = null;
  }

  // Resize handlers
  function startResize(e: globalThis.MouseEvent) {
    e.preventDefault();
    isResizing = true;
    document.addEventListener("mousemove", onResize);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor = "ew-resize";
    document.body.style.userSelect = "none";
  }

  function onResize(e: globalThis.MouseEvent) {
    if (!isResizing) return;
    // Calculate width from right edge of window
    const newWidth = window.innerWidth - e.clientX;
    ui.setReferencesPanelWidth(newWidth);
  }

  function stopResize() {
    isResizing = false;
    document.removeEventListener("mousemove", onResize);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  }

  function onResizeKeydown(e: globalThis.KeyboardEvent) {
    const step = 20;
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      ui.setReferencesPanelWidth(ui.referencesPanelWidth + step);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      ui.setReferencesPanelWidth(ui.referencesPanelWidth - step);
    }
  }

  $effect(() => {
    if (currentProject.value) {
      loadReferences();
    }
  });
</script>

<aside
  class="bg-bg-panel border-l border-bg-card flex flex-col h-full relative"
  class:w-0={ui.referencesPanelCollapsed}
  class:overflow-hidden={ui.referencesPanelCollapsed}
  class:opacity-0={ui.referencesPanelCollapsed}
  class:border-l-0={ui.referencesPanelCollapsed}
  class:p-0={ui.referencesPanelCollapsed}
  class:transition-all={!isResizing}
  class:duration-200={!isResizing}
  style={ui.referencesPanelCollapsed ? "" : `width: ${ui.referencesPanelWidth}px`}
>
  <!-- Resize handle -->
  {#if !ui.referencesPanelCollapsed}
    <div
      class="absolute left-0 top-0 bottom-0 w-1 cursor-ew-resize hover:bg-accent/50 active:bg-accent transition-colors z-10 focus:outline-none focus:bg-accent"
      onmousedown={startResize}
      onkeydown={onResizeKeydown}
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize references panel"
      aria-valuenow={ui.referencesPanelWidth}
      aria-valuemin={ui.referencesPanelMinWidth}
      aria-valuemax={ui.referencesPanelMaxWidth}
      tabindex="0"
    ></div>
  {/if}
  <!-- Header with tabs -->
  <div class="border-b border-bg-card">
    <div class="flex items-center justify-between px-4 py-2">
      <h2 class="text-sm font-heading font-medium text-text-primary">References</h2>
      <div class="flex items-center gap-1">
        <!-- Collapse All button -->
        <button
          onclick={collapseAll}
          class="text-text-secondary hover:text-text-primary p-1"
          aria-label="Collapse all"
          title="Collapse all"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 15l7-7 7 7"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 9l7-7 7 7"
            />
          </svg>
        </button>
        <!-- Sort Alphabetically button -->
        <button
          onclick={sortAlphabetically}
          class="text-text-secondary hover:text-text-primary p-1 text-xs font-bold"
          aria-label="Sort alphabetically"
          title="Sort A-Z"
        >
          <span class="flex items-center gap-0.5">
            A
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 14l-7 7-7-7"
              />
            </svg>
            Z
          </span>
        </button>
        <!-- Close panel button -->
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
            {@const isExpanded = expandedIds.has(character.id)}
            {@const attributes = formatAttributes(character.attributes)}
            {@const notes = getNotes(character.attributes)}
            <div
              class="bg-bg-card rounded-lg overflow-hidden"
              class:ring-2={dragOverId === character.id}
              class:ring-accent={dragOverId === character.id}
              style:opacity={draggedId === character.id ? 0.5 : 1}
              data-drag-item={character.id}
              role="listitem"
            >
              <div
                class="w-full flex items-center gap-3 p-3 hover:bg-beat-header transition-colors"
              >
                <!-- Drag handle -->
                <div
                  class="text-text-secondary/50 cursor-grab active:cursor-grabbing flex-shrink-0 hover:text-text-secondary"
                  onmousedown={(e) => onDragHandleMouseDown(e, character.id)}
                  role="button"
                  tabindex="-1"
                  aria-label="Drag to reorder"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 6h16M4 12h16M4 18h16"
                    />
                  </svg>
                </div>
                <!-- Clickable area for expand/collapse -->
                <button
                  onclick={() => toggleExpanded(character.id)}
                  class="flex-1 flex items-center gap-3 text-left"
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
              </div>

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

                  {#if !notes && attributes.length === 0}
                    <p class="text-text-secondary text-sm mt-3 italic">No additional details</p>
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
            {@const isExpanded = expandedIds.has(location.id)}
            {@const attributes = formatAttributes(location.attributes)}
            {@const notes = getNotes(location.attributes)}
            <div
              class="bg-bg-card rounded-lg overflow-hidden"
              class:ring-2={dragOverId === location.id}
              class:ring-accent={dragOverId === location.id}
              style:opacity={draggedId === location.id ? 0.5 : 1}
              data-drag-item={location.id}
              role="listitem"
            >
              <div
                class="w-full flex items-center gap-3 p-3 hover:bg-beat-header transition-colors"
              >
                <!-- Drag handle -->
                <div
                  class="text-text-secondary/50 cursor-grab active:cursor-grabbing flex-shrink-0 hover:text-text-secondary"
                  onmousedown={(e) => onDragHandleMouseDown(e, location.id)}
                  role="button"
                  tabindex="-1"
                  aria-label="Drag to reorder"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 6h16M4 12h16M4 18h16"
                    />
                  </svg>
                </div>
                <!-- Clickable area for expand/collapse -->
                <button
                  onclick={() => toggleExpanded(location.id)}
                  class="flex-1 flex items-center gap-3 text-left"
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
              </div>

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

                  {#if !notes && attributes.length === 0}
                    <p class="text-text-secondary text-sm mt-3 italic">No additional details</p>
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
