<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ArrowDownAZ,
    ChevronDown,
    ChevronsLeft,
    ChevronsRight,
    GripVertical,
    ListChevronsDownUp,
    MapPin,
    User,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type { Character, Location } from "../types";
  import Tooltip from "./Tooltip.svelte";

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

  // Strip HTML tags for preview text
  function stripHtml(html: string | null | undefined): string {
    if (!html) return "";
    return html.replace(/<[^>]*>/g, "").trim();
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
        <Tooltip text="Collapse all" position="bottom">
          <button
            onclick={collapseAll}
            class="text-text-secondary hover:text-text-primary p-1"
            aria-label="Collapse all"
          >
            <ListChevronsDownUp class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Sort Alphabetically button -->
        <Tooltip text="Sort A-Z" position="bottom">
          <button
            onclick={sortAlphabetically}
            class="text-text-secondary hover:text-text-primary p-1"
            aria-label="Sort alphabetically"
          >
            <ArrowDownAZ class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Close panel button -->
        <Tooltip text="Collapse panel" position="bottom">
          <button
            onclick={toggleReferencesPanel}
            class="text-text-secondary hover:text-text-primary p-1"
            aria-label="Collapse references panel"
          >
            <ChevronsRight class="w-4 h-4" />
          </button>
        </Tooltip>
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
                  <GripVertical class="w-4 h-4" />
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
                    <User class="w-4 h-4 text-accent" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-text-primary font-medium text-sm truncate">{character.name}</p>
                    {#if character.description}
                      <p class="text-text-secondary text-xs truncate">
                        {stripHtml(character.description)}
                      </p>
                    {/if}
                  </div>
                  <ChevronDown
                    class="w-4 h-4 text-text-secondary transition-transform flex-shrink-0 {isExpanded
                      ? 'rotate-180'
                      : ''}"
                  />
                </button>
              </div>

              {#if isExpanded}
                <div class="px-3 pb-3 border-t border-bg-panel">
                  {#if character.description}
                    <div
                      class="text-text-primary text-sm mt-3 leading-relaxed max-w-none break-words [&>p]:mb-2 [&>p:last-child]:mb-0 [&_strong]:font-semibold [&_em]:italic"
                    >
                      {@html character.description}
                    </div>
                  {/if}

                  {#if notes}
                    <p class="text-text-primary text-sm mt-3 leading-relaxed break-words">
                      {notes}
                    </p>
                  {/if}

                  {#if attributes.length > 0}
                    <div class="mt-3 space-y-1.5">
                      {#each attributes as [key, value] (key)}
                        <div class="flex gap-2 text-xs">
                          <span class="text-text-secondary font-medium shrink-0">{key}:</span>
                          <span class="text-text-primary break-words">{value}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}

                  {#if !character.description && !notes && attributes.length === 0}
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
                  <GripVertical class="w-4 h-4" />
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
                    <MapPin class="w-4 h-4 text-spark-gold" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-text-primary font-medium text-sm truncate">{location.name}</p>
                    {#if location.description}
                      <p class="text-text-secondary text-xs truncate">
                        {stripHtml(location.description)}
                      </p>
                    {/if}
                  </div>
                  <ChevronDown
                    class="w-4 h-4 text-text-secondary transition-transform flex-shrink-0 {isExpanded
                      ? 'rotate-180'
                      : ''}"
                  />
                </button>
              </div>

              {#if isExpanded}
                <div class="px-3 pb-3 border-t border-bg-panel">
                  {#if location.description}
                    <div
                      class="text-text-primary text-sm mt-3 leading-relaxed max-w-none break-words [&>p]:mb-2 [&>p:last-child]:mb-0 [&_strong]:font-semibold [&_em]:italic"
                    >
                      {@html location.description}
                    </div>
                  {/if}

                  {#if notes}
                    <p class="text-text-primary text-sm mt-3 leading-relaxed break-words">
                      {notes}
                    </p>
                  {/if}

                  {#if attributes.length > 0}
                    <div class="mt-3 space-y-1.5">
                      {#each attributes as [key, value] (key)}
                        <div class="flex gap-2 text-xs">
                          <span class="text-text-secondary font-medium shrink-0">{key}:</span>
                          <span class="text-text-primary break-words">{value}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}

                  {#if !location.description && !notes && attributes.length === 0}
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
  <Tooltip text="Expand references" position="left">
    <button
      onclick={toggleReferencesPanel}
      class="fixed right-0 top-1/2 -translate-y-1/2 bg-bg-panel p-2 rounded-l-lg text-text-secondary hover:text-text-primary z-10"
      aria-label="Expand references panel"
    >
      <ChevronsLeft class="w-5 h-5" />
    </button>
  </Tooltip>
{/if}
