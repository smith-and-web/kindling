<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ArrowDownAZ,
    ChevronDown,
    ChevronsLeft,
    ChevronsRight,
    GripVertical,
    Link2,
    ListChevronsDownUp,
    Pencil,
    Plus,
    Trash2,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type {
    ReferenceItem,
    ReferenceTypeId,
    SceneReferenceState,
    SceneReferenceStateUpdate,
  } from "../types";
  import {
    DEFAULT_REFERENCE_TYPES,
    REFERENCE_TYPE_OPTIONS,
    type ReferenceTypeOption,
    normalizeReferenceTypes,
  } from "../referenceTypes";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import ReferenceEditDialog from "./ReferenceEditDialog.svelte";
  import Tooltip from "./Tooltip.svelte";

  let activeTab = $state<ReferenceTypeId | null>(null);
  let loading = $state(false);
  let referenceTypeOptions = $state<ReferenceTypeOption[]>([]);
  let referencesByType = $state<Record<ReferenceTypeId, ReferenceItem[]>>(
    {} as Record<ReferenceTypeId, ReferenceItem[]>
  );
  let expandedIds = new SvelteSet<string>();
  let sceneReferenceStates = $state<SceneReferenceState[]>([]);
  let sceneReferenceLoading = $state(false);
  let sceneReferenceError = $state<string | null>(null);
  let sceneReferenceRequestId = 0;
  let isResizing = $state(false);
  let draggedId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);
  let isDragging = $state(false);
  let editDialog = $state<{
    mode: "create" | "edit";
    referenceType: ReferenceTypeOption;
    reference?: ReferenceItem;
  } | null>(null);
  let deleteTarget = $state<ReferenceItem | null>(null);
  let activeTypeOption = $derived(getReferenceTypeOption(activeTab));
  let activeSceneStates = $derived(
    activeTab && currentProject.currentScene
      ? getSceneStatesForType(activeTab)
      : ([] as SceneReferenceState[])
  );
  let linkedIds = $derived(new Set(activeSceneStates.map((state) => state.reference_id)));
  let linkedItems = $derived(
    activeTab
      ? activeSceneStates
          .map((state) =>
            getReferencesForType(activeTab as ReferenceTypeId).find(
              (item) => item.id === state.reference_id
            )
          )
          .filter((item): item is ReferenceItem => item !== undefined)
      : []
  );
  let unlinkedItems = $derived(
    activeTab
      ? getReferencesForType(activeTab as ReferenceTypeId).filter((item) => !linkedIds.has(item.id))
      : []
  );
  let activeItems = $derived(
    currentProject.currentScene
      ? [...linkedItems, ...unlinkedItems]
      : activeTab
        ? getReferencesForType(activeTab as ReferenceTypeId)
        : []
  );
  let ActiveIcon = $derived(activeTypeOption?.icon ?? null);
  let iconBgClass = $derived(activeTypeOption?.bgClass ?? "bg-accent/20");
  let iconTextClass = $derived(activeTypeOption?.accentClass ?? "text-accent");

  async function loadReferences() {
    const project = currentProject.value;
    if (!project) return;

    const enabledTypes = normalizeReferenceTypes(
      project.reference_types ?? DEFAULT_REFERENCE_TYPES
    );
    referenceTypeOptions = REFERENCE_TYPE_OPTIONS.filter((option) =>
      enabledTypes.includes(option.id)
    );

    if (!activeTab || !enabledTypes.includes(activeTab)) {
      activeTab = enabledTypes[0] ?? null;
    }

    if (enabledTypes.length === 0) {
      referencesByType = {} as Record<ReferenceTypeId, ReferenceItem[]>;
      loading = false;
      return;
    }

    loading = true;
    try {
      const results = await Promise.all(
        enabledTypes.map(async (type) => {
          const items = await invoke<ReferenceItem[]>("get_references", {
            projectId: project.id,
            referenceType: type,
          });
          return [type, items] as const;
        })
      );

      const next: Record<ReferenceTypeId, ReferenceItem[]> = {
        ...(referencesByType as Record<ReferenceTypeId, ReferenceItem[]>),
      };
      for (const [type, items] of results) {
        next[type] = items;
      }
      referencesByType = next;

      if (next.characters) {
        currentProject.setCharacters(next.characters);
      }
      if (next.locations) {
        currentProject.setLocations(next.locations);
      }
    } catch (e) {
      console.error("Failed to load references:", e);
    } finally {
      loading = false;
    }
  }

  function getSceneStatesForType(type: ReferenceTypeId): SceneReferenceState[] {
    return sceneReferenceStates
      .filter((state) => state.reference_type === type)
      .sort((a, b) => a.position - b.position);
  }

  function syncExpandedIdsFromState(states: SceneReferenceState[]) {
    expandedIds.clear();
    for (const state of states) {
      if (state.expanded) {
        expandedIds.add(state.reference_id);
      }
    }
  }

  async function loadSceneReferenceState(sceneId: string) {
    const requestId = ++sceneReferenceRequestId;
    sceneReferenceLoading = true;
    sceneReferenceError = null;
    try {
      const states = await invoke<SceneReferenceState[]>("get_scene_reference_state", {
        sceneId,
      });
      if (requestId !== sceneReferenceRequestId) return;
      sceneReferenceStates = states;
      syncExpandedIdsFromState(states);
    } catch (e) {
      if (requestId !== sceneReferenceRequestId) return;
      console.error("Failed to load scene reference state:", e);
      sceneReferenceError = e instanceof Error ? e.message : "Failed to load scene reference state";
      sceneReferenceStates = [];
      syncExpandedIdsFromState([]);
    } finally {
      if (requestId === sceneReferenceRequestId) {
        sceneReferenceLoading = false;
      }
    }
  }

  async function saveSceneReferenceState(
    referenceType: ReferenceTypeId,
    updates: SceneReferenceStateUpdate[]
  ) {
    const scene = currentProject.currentScene;
    if (!scene) return;
    try {
      await invoke("save_scene_reference_state", {
        sceneId: scene.id,
        referenceType,
        states: updates,
      });
      const nextStates = [
        ...sceneReferenceStates.filter((state) => state.reference_type !== referenceType),
        ...updates.map((update) => ({
          scene_id: scene.id,
          reference_type: referenceType,
          reference_id: update.reference_id,
          position: update.position,
          expanded: update.expanded,
        })),
      ];
      sceneReferenceStates = nextStates;
      syncExpandedIdsFromState(nextStates);
    } catch (e) {
      console.error("Failed to save scene reference state:", e);
      sceneReferenceError = e instanceof Error ? e.message : "Failed to save scene reference state";
    }
  }

  function toggleExpanded(id: string) {
    const isExpanded = expandedIds.has(id);
    const nextExpanded = !isExpanded;
    if (currentProject.currentScene && activeTab && linkedIds.has(id)) {
      const states = getSceneStatesForType(activeTab);
      const updates = states.map((state, index) => ({
        reference_id: state.reference_id,
        position: index,
        expanded: state.reference_id === id ? nextExpanded : state.expanded,
      }));
      saveSceneReferenceState(activeTab, updates);
      return;
    }

    if (isExpanded) {
      expandedIds.delete(id);
    } else {
      expandedIds.add(id);
    }
  }

  function collapseAll() {
    if (currentProject.currentScene && activeTab) {
      const states = getSceneStatesForType(activeTab);
      const updates = states.map((state, index) => ({
        reference_id: state.reference_id,
        position: index,
        expanded: false,
      }));
      saveSceneReferenceState(activeTab, updates);
      return;
    }
    expandedIds.clear();
  }

  function sortAlphabetically() {
    if (!activeTab) return;
    if (currentProject.currentScene) {
      const states = getSceneStatesForType(activeTab);
      const itemMap = new Map((referencesByType[activeTab] ?? []).map((item) => [item.id, item]));
      const sorted = [...states].sort((a, b) => {
        const aName = itemMap.get(a.reference_id)?.name ?? "";
        const bName = itemMap.get(b.reference_id)?.name ?? "";
        return aName.localeCompare(bName);
      });
      const updates = sorted.map((state, index) => ({
        reference_id: state.reference_id,
        position: index,
        expanded: state.expanded,
      }));
      saveSceneReferenceState(activeTab, updates);
      return;
    }

    const items = referencesByType[activeTab] ?? [];
    const sorted = [...items].sort((a, b) => a.name.localeCompare(b.name));
    referencesByType = { ...referencesByType, [activeTab]: sorted };
    if (activeTab === "characters") {
      currentProject.setCharacters(sorted);
    } else if (activeTab === "locations") {
      currentProject.setLocations(sorted);
    }
  }

  async function toggleSceneLink(reference: ReferenceItem) {
    if (!currentProject.currentScene) return;
    const referenceType = reference.reference_type;
    const states = getSceneStatesForType(referenceType);
    const isLinked = states.some((state) => state.reference_id === reference.id);

    let updates: SceneReferenceStateUpdate[];
    if (isLinked) {
      updates = states
        .filter((state) => state.reference_id !== reference.id)
        .map((state, index) => ({
          reference_id: state.reference_id,
          position: index,
          expanded: state.expanded,
        }));
    } else {
      updates = [
        ...states.map((state, index) => ({
          reference_id: state.reference_id,
          position: index,
          expanded: state.expanded,
        })),
        {
          reference_id: reference.id,
          position: states.length,
          expanded: false,
        },
      ];
    }

    await saveSceneReferenceState(referenceType, updates);
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

  function getReferenceTypeOption(type: ReferenceTypeId | null) {
    return REFERENCE_TYPE_OPTIONS.find((option) => option.id === type) ?? null;
  }

  function getReferencesForType(type: ReferenceTypeId) {
    return referencesByType[type] ?? [];
  }

  function getReferenceCount(type: ReferenceTypeId) {
    return referencesByType[type]?.length ?? 0;
  }

  function openCreateDialog() {
    const typeOption = getReferenceTypeOption(activeTab);
    if (!typeOption) return;
    editDialog = { mode: "create", referenceType: typeOption };
  }

  function openEditDialog(reference: ReferenceItem) {
    const typeOption = getReferenceTypeOption(reference.reference_type);
    if (!typeOption) return;
    editDialog = { mode: "edit", referenceType: typeOption, reference };
  }

  async function handleSaveReference(data: {
    name: string;
    description: string | null;
    attributes: Record<string, string>;
  }) {
    if (!currentProject.value || !editDialog) return;

    try {
      if (editDialog.mode === "create") {
        await invoke("create_reference", {
          projectId: currentProject.value.id,
          referenceType: editDialog.referenceType.id,
          reference: data,
        });
      } else if (editDialog.reference) {
        await invoke("update_reference", {
          referenceId: editDialog.reference.id,
          referenceType: editDialog.referenceType.id,
          reference: data,
        });
      }
      await loadReferences();
    } catch (e) {
      console.error("Failed to save reference:", e);
      throw e;
    }
  }

  async function handleDeleteReference() {
    if (!deleteTarget) return;
    try {
      await invoke("delete_reference", {
        referenceId: deleteTarget.id,
        referenceType: deleteTarget.reference_type,
      });
      await loadReferences();
      if (currentProject.currentScene) {
        loadSceneReferenceState(currentProject.currentScene.id);
      }
    } catch (e) {
      console.error("Failed to delete reference:", e);
      ui.showError(`Failed to delete reference: ${e}`);
    } finally {
      deleteTarget = null;
    }
  }

  // Pointer-based drag and drop (more reliable than HTML5 drag API in webviews)
  let draggedElement: globalThis.HTMLElement | null = null;
  let currentDragOverElement: globalThis.HTMLElement | null = null;

  function onDragHandleMouseDown(e: globalThis.MouseEvent, id: string, canDrag = true) {
    if (!canDrag) {
      return;
    }
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

    if (draggedId && dragOverId && draggedId !== dragOverId && activeTab) {
      if (currentProject.currentScene) {
        const states = getSceneStatesForType(activeTab);
        const fromIndex = states.findIndex((state) => state.reference_id === draggedId);
        const toIndex = states.findIndex((state) => state.reference_id === dragOverId);
        if (fromIndex !== -1 && toIndex !== -1) {
          const nextStates = [...states];
          const [moved] = nextStates.splice(fromIndex, 1);
          nextStates.splice(toIndex, 0, moved);
          const updates = nextStates.map((state, index) => ({
            reference_id: state.reference_id,
            position: index,
            expanded: state.expanded,
          }));
          saveSceneReferenceState(activeTab, updates);
        }
      } else {
        const items = [...(referencesByType[activeTab] ?? [])];
        const fromIndex = items.findIndex((item) => item.id === draggedId);
        const toIndex = items.findIndex((item) => item.id === dragOverId);
        if (fromIndex !== -1 && toIndex !== -1) {
          const [moved] = items.splice(fromIndex, 1);
          items.splice(toIndex, 0, moved);
          referencesByType = { ...referencesByType, [activeTab]: items };
          if (activeTab === "characters") {
            currentProject.setCharacters(items);
          } else if (activeTab === "locations") {
            currentProject.setLocations(items);
          }
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

  let lastSceneId: string | null = null;
  $effect(() => {
    const sceneId = currentProject.currentScene?.id ?? null;
    if (sceneId === lastSceneId) return;
    lastSceneId = sceneId;
    if (sceneId) {
      loadSceneReferenceState(sceneId);
    } else {
      sceneReferenceStates = [];
      sceneReferenceError = null;
      sceneReferenceLoading = false;
      syncExpandedIdsFromState([]);
    }
  });

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
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
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
        <!-- Add Reference button -->
        <Tooltip
          text={activeTab
            ? `Add ${getReferenceTypeOption(activeTab)?.label ?? "Reference"}`
            : "Add reference"}
          position="bottom"
        >
          <button
            onclick={openCreateDialog}
            class="text-text-secondary hover:text-text-primary p-1 disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="Add reference"
            disabled={!activeTab}
          >
            <Plus class="w-4 h-4" />
          </button>
        </Tooltip>
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
    <div class="flex border-t border-bg-card overflow-x-auto">
      {#each referenceTypeOptions as typeOption (typeOption.id)}
        <button
          onclick={() => (activeTab = typeOption.id)}
          class="flex-1 px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap"
          class:text-accent={activeTab === typeOption.id}
          class:border-b-2={activeTab === typeOption.id}
          class:border-accent={activeTab === typeOption.id}
          class:text-text-secondary={activeTab !== typeOption.id}
        >
          {typeOption.label} ({getReferenceCount(typeOption.id)})
        </button>
      {/each}
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary text-sm">Loading...</span>
      </div>
    {:else if !activeTab}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary text-sm">
          No reference types enabled. Configure them in Project Settings.
        </span>
      </div>
    {:else if activeItems.length === 0}
      <div class="flex items-center justify-center p-4">
        <span class="text-text-secondary text-sm">
          No {activeTypeOption?.label.toLowerCase() ?? "references"}
        </span>
      </div>
    {:else}
      {#if currentProject.currentScene}
        <div class="flex items-center justify-between px-1 pb-2 text-xs text-text-secondary">
          <span class="uppercase tracking-wide">Linked to this scene</span>
          {#if sceneReferenceLoading}
            <span>Loading…</span>
          {/if}
        </div>
        {#if sceneReferenceError}
          <div class="px-1 pb-2 text-xs text-red-400">{sceneReferenceError}</div>
        {:else if linkedItems.length === 0 && !sceneReferenceLoading}
          <div class="px-1 pb-2 text-xs text-text-secondary">
            No references linked to this scene yet.
          </div>
        {/if}
      {/if}
      <div class="space-y-2">
        {#each activeItems as reference, index (reference.id)}
          {@const isExpanded = expandedIds.has(reference.id)}
          {@const attributes = formatAttributes(reference.attributes)}
          {@const notes = getNotes(reference.attributes)}
          {@const isLinked = currentProject.currentScene ? linkedIds.has(reference.id) : false}
          {@const canDrag = !currentProject.currentScene || isLinked}
          {#if currentProject.currentScene && linkedItems.length > 0 && index === linkedItems.length}
            <div
              class="border-t border-bg-card pt-3 mt-3 text-xs text-text-secondary uppercase tracking-wide"
            >
              All references
            </div>
          {/if}
          <div
            class="bg-bg-card rounded-lg overflow-hidden"
            class:ring-2={dragOverId === reference.id}
            class:ring-accent={dragOverId === reference.id}
            style:opacity={draggedId === reference.id ? 0.5 : 1}
            data-drag-item={reference.id}
            role="listitem"
          >
            <div class="w-full flex items-center gap-3 p-3 hover:bg-beat-header transition-colors">
              <!-- Drag handle -->
              <div
                class="text-text-secondary/50 cursor-grab active:cursor-grabbing shrink-0 hover:text-text-secondary"
                class:opacity-40={!canDrag}
                onmousedown={(e) => onDragHandleMouseDown(e, reference.id, canDrag)}
                role="button"
                tabindex="-1"
                aria-label="Drag to reorder"
              >
                <GripVertical class="w-4 h-4" />
              </div>
              {#if currentProject.currentScene}
                <Tooltip text={isLinked ? "Unlink from scene" : "Link to scene"} position="bottom">
                  <button
                    onclick={() => toggleSceneLink(reference)}
                    class={`shrink-0 inline-flex items-center gap-1 rounded border px-2 py-1 text-xs transition-colors ${
                      isLinked
                        ? "border-accent/60 text-accent hover:border-accent"
                        : "border-bg-card text-text-secondary hover:text-text-primary hover:border-accent/40"
                    }`}
                    aria-label={isLinked ? "Unlink from scene" : "Link to scene"}
                  >
                    <Link2 class="w-3 h-3" />
                    <span>{isLinked ? "Unlink" : "Link"}</span>
                  </button>
                </Tooltip>
              {/if}
              <!-- Clickable area for expand/collapse -->
              <button
                onclick={() => toggleExpanded(reference.id)}
                class="flex-1 flex items-center gap-3 text-left"
              >
                <!-- Reference icon -->
                <div
                  class={`w-8 h-8 rounded-full ${iconBgClass} flex items-center justify-center shrink-0`}
                >
                  {#if ActiveIcon}
                    <ActiveIcon class={`w-4 h-4 ${iconTextClass}`} />
                  {/if}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-text-primary font-medium text-sm truncate">{reference.name}</p>
                  {#if reference.description}
                    <p class="text-text-secondary text-xs truncate">
                      {stripHtml(reference.description)}
                    </p>
                  {/if}
                </div>
                <ChevronDown
                  class="w-4 h-4 text-text-secondary transition-transform shrink-0 {isExpanded
                    ? 'rotate-180'
                    : ''}"
                />
              </button>
            </div>

            {#if isExpanded}
              <div class="px-3 pb-3 border-t border-bg-panel">
                {#if reference.description}
                  <div
                    class="text-text-primary text-sm mt-3 leading-relaxed max-w-none wrap-break-word [&>p]:mb-2 [&>p:last-child]:mb-0 [&_strong]:font-semibold [&_em]:italic"
                  >
                    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                    {@html reference.description}
                  </div>
                {/if}

                {#if notes}
                  <p class="text-text-primary text-sm mt-3 leading-relaxed wrap-break-word">
                    {notes}
                  </p>
                {/if}

                {#if attributes.length > 0}
                  <div class="mt-3 space-y-1.5">
                    {#each attributes as [key, value] (key)}
                      <div class="flex gap-2 text-xs">
                        <span class="text-text-secondary font-medium shrink-0">{key}:</span>
                        <span class="text-text-primary wrap-break-word">{value}</span>
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if !reference.description && !notes && attributes.length === 0}
                  <p class="text-text-secondary text-sm mt-3 italic">No additional details</p>
                {/if}

                <div class="flex items-center gap-2 mt-4">
                  <Tooltip text="Edit" position="bottom">
                    <button
                      onclick={() => openEditDialog(reference)}
                      class="text-text-secondary hover:text-text-primary p-1"
                      aria-label="Edit reference"
                    >
                      <Pencil class="w-4 h-4" />
                    </button>
                  </Tooltip>
                  <Tooltip text="Delete" position="bottom">
                    <button
                      onclick={() => (deleteTarget = reference)}
                      class="text-text-secondary hover:text-red-400 p-1"
                      aria-label="Delete reference"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </Tooltip>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
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

{#if editDialog}
  <ReferenceEditDialog
    referenceType={editDialog.referenceType}
    reference={editDialog.reference}
    onClose={() => (editDialog = null)}
    onSave={handleSaveReference}
  />
{/if}

{#if deleteTarget}
  <ConfirmDialog
    title="Delete reference?"
    message={`This will permanently delete "${deleteTarget.name}".`}
    confirmLabel="Delete"
    onConfirm={handleDeleteReference}
    onCancel={() => (deleteTarget = null)}
  />
{/if}
