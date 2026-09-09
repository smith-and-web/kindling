<script lang="ts">
  import { REFERENCE_FIELD_TYPES } from "../referenceTypes";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ArrowDownAZ,
    Copy,
    ChevronDown,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
    Zap,
    GripVertical,
    Link2,
    ListChevronsDownUp,
    Settings,
    Pencil,
    Plus,
    Trash2,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import type {
    Project,
    ReferenceCopyResult,
    ReferenceItem,
    ReferenceTypeId,
    ReferenceSuggestion,
    SceneReferenceState,
    SceneReferenceStateUpdate,
    FieldDefinition,
    FieldValue,
    Tag,
  } from "../types";
  import {
    DEFAULT_REFERENCE_TYPES,
    REFERENCE_TYPE_OPTIONS,
    type ReferenceTypeOption,
    normalizeReferenceTypes,
  } from "../referenceTypes";
  import CopyReferencesDialog from "./CopyReferencesDialog.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import ReferenceEditDialog from "./ReferenceEditDialog.svelte";
  import SuggestionCard from "./SuggestionCard.svelte";
  import TagSelector from "./TagSelector.svelte";
  import Tooltip from "./Tooltip.svelte";

  let { contextSceneId, embedded = false }: { contextSceneId?: string | null; embedded?: boolean } =
    $props();
  const referenceScene = $derived(
    contextSceneId === undefined
      ? currentProject.currentScene
      : contextSceneId
        ? { id: contextSceneId }
        : null
  );
  const collapsed = $derived(!embedded && ui.referencesPanelCollapsed);

  let copyDestination = $state<Project | null>(null);
  let explicitlyRefreshedProject: Project | null = null;

  async function referencesCopied(result: ReferenceCopyResult) {
    if (currentProject.value?.id !== result.project.id) return;
    await loadReferences(true, result.project);
    if (currentProject.value?.id !== result.project.id) return;
    currentProject.setProject(result.project);
    // The explicit refresh reports failures to the copy dialog. Consume only this
    // exact store assignment so ordinary settings updates still reload the panel.
    explicitlyRefreshedProject = currentProject.value;
    const sceneId = referenceScene?.id;
    if (sceneId) await loadSuggestions(sceneId);
  }

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
  let loadReferencesRequestId = 0;
  let suggestionsRequestId = 0;
  let showReferenceTypeSettings = $state(false);
  let referenceTypeSelection = $state<ReferenceTypeId[]>([]);
  let referenceTypeSaving = $state(false);
  let referenceTypeError = $state<string | null>(null);
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
  let fieldDefsMap = $state<Record<string, FieldDefinition[]>>({});
  let fieldValuesMap = $state<Record<string, Record<string, string | null>>>({});
  let suggestions = $state<ReferenceSuggestion[]>([]);
  let suggestionsLoading = $state(false);
  let suggestionsOpen = $state(true);
  let allTags = $state<Tag[]>([]);
  let entityTagIds = $state<Record<string, string[]>>({});
  let activeTypeOption = $derived(getReferenceTypeOption(activeTab));
  let activeSceneStates = $derived(
    activeTab && referenceScene ? getSceneStatesForType(activeTab) : ([] as SceneReferenceState[])
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
    referenceScene
      ? [...linkedItems, ...unlinkedItems]
      : activeTab
        ? getReferencesForType(activeTab as ReferenceTypeId)
        : []
  );
  let ActiveIcon = $derived(activeTypeOption?.icon ?? null);
  let iconBgClass = $derived(activeTypeOption?.bgClass ?? "bg-press-accent-wash");
  let iconTextClass = $derived(activeTypeOption?.accentClass ?? "text-press-accent-text");

  async function loadReferences(reportErrors = false, projectOverride?: Project) {
    const requestId = ++loadReferencesRequestId;
    const project = projectOverride ?? currentProject.value;
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
      fieldDefsMap = {};
      fieldValuesMap = {};
      allTags = [];
      entityTagIds = {};
      currentProject.setCharacters([]);
      currentProject.setLocations([]);
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

      if (requestId !== loadReferencesRequestId) return;

      const next = {} as Record<ReferenceTypeId, ReferenceItem[]>;
      for (const [type, items] of results) {
        next[type] = items;
      }
      // Load field definitions for each entity type
      const entityTypeMap = REFERENCE_FIELD_TYPES;
      const defsMap: Record<string, FieldDefinition[]> = {};
      for (const type of enabledTypes) {
        const entityType = entityTypeMap[type] ?? type;
        try {
          defsMap[type] = await invoke<FieldDefinition[]>("get_field_definitions", {
            projectId: project.id,
            entityType,
          });
        } catch {
          if (reportErrors) throw new Error("Could not load copied fields");
          defsMap[type] = [];
        }
      }
      const vMap: Record<string, Record<string, string | null>> = {};

      // Load field values for all entities in bulk
      const allEntityIds = Object.values(next)
        .flat()
        .map((item) => item.id);
      if (allEntityIds.length > 0) {
        try {
          const allValues = await invoke<FieldValue[]>("get_field_values_bulk", {
            entityIds: allEntityIds,
          });

          for (const v of allValues) {
            if (!vMap[v.entity_id]) vMap[v.entity_id] = {};
            vMap[v.entity_id][v.field_definition_id] = v.value;
          }
        } catch {
          if (reportErrors) throw new Error("Could not load copied field values");
        }
      }

      // Load all project tags + per-entity tag assignments
      let nextTags: Tag[] = [];
      const tagMap: Record<string, string[]> = {};
      try {
        nextTags = await invoke<Tag[]>("get_tags", { projectId: project.id });

        for (const id of allEntityIds) {
          const entityType =
            Object.entries(entityTypeMap).find(([typeKey]) =>
              (next[typeKey as ReferenceTypeId] ?? []).some((item) => item.id === id)
            )?.[1] ?? "item";
          try {
            const tags = await invoke<Tag[]>("get_entity_tags", { entityType, entityId: id });
            tagMap[id] = tags.map((t) => t.id);
          } catch {
            if (reportErrors) throw new Error("Could not load copied tag assignments");
            tagMap[id] = [];
          }
        }
      } catch (e) {
        if (reportErrors) throw e;
      }
      if (requestId !== loadReferencesRequestId || currentProject.value?.id !== project.id) return;
      referencesByType = next;
      fieldDefsMap = defsMap;
      fieldValuesMap = vMap;
      allTags = nextTags;
      entityTagIds = tagMap;
      currentProject.setCharacters(next.characters ?? []);
      currentProject.setLocations(next.locations ?? []);
    } catch (e) {
      if (requestId !== loadReferencesRequestId) return;
      console.error("Failed to load references:", e);
      if (reportErrors) throw e;
    } finally {
      if (requestId === loadReferencesRequestId) {
        loading = false;
      }
    }
  }

  function openReferenceTypeSettings() {
    referenceTypeSelection = normalizeReferenceTypes(
      currentProject.value?.reference_types ?? DEFAULT_REFERENCE_TYPES
    );
    referenceTypeError = null;
    showReferenceTypeSettings = true;
  }

  function closeReferenceTypeSettings() {
    showReferenceTypeSettings = false;
  }

  function toggleReferenceType(typeId: ReferenceTypeId) {
    if (referenceTypeSelection.includes(typeId)) {
      referenceTypeSelection = referenceTypeSelection.filter((type) => type !== typeId);
    } else {
      referenceTypeSelection = [...referenceTypeSelection, typeId];
    }
  }

  async function saveReferenceTypeSettings() {
    if (!currentProject.value) return;
    referenceTypeSaving = true;
    referenceTypeError = null;
    try {
      const updatedProject = await invoke<Project>("update_project_settings", {
        projectId: currentProject.value.id,
        settings: {
          reference_types: referenceTypeSelection,
        },
      });
      currentProject.setProject(updatedProject);
      await loadReferences();
      showReferenceTypeSettings = false;
    } catch (e) {
      console.error("Failed to update reference types:", e);
      referenceTypeError = e instanceof Error ? e.message : "Failed to update reference types";
    } finally {
      referenceTypeSaving = false;
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

  async function loadSuggestions(sceneId: string) {
    const requestId = ++suggestionsRequestId;
    suggestionsLoading = true;
    try {
      const result = await invoke<ReferenceSuggestion[]>("detect_scene_references", { sceneId });
      if (requestId !== suggestionsRequestId) return;
      suggestions = result;
    } catch (e) {
      if (requestId !== suggestionsRequestId) return;
      console.error("Failed to detect references:", e);
      suggestions = [];
    } finally {
      if (requestId === suggestionsRequestId) {
        suggestionsLoading = false;
      }
    }
  }

  function referenceTypeForSuggestion(s: ReferenceSuggestion): ReferenceTypeId {
    if (s.reference_type === "character") return "characters";
    if (s.reference_type === "location") return "locations";
    return s.reference_type as ReferenceTypeId;
  }

  async function linkSuggestion(s: ReferenceSuggestion) {
    const scene = referenceScene;
    if (!scene) return;

    const refType = referenceTypeForSuggestion(s);
    const states = getSceneStatesForType(refType);
    const updates: SceneReferenceStateUpdate[] = [
      ...states.map((state, i) => ({
        reference_id: state.reference_id,
        position: i,
        expanded: state.expanded,
      })),
      { reference_id: s.reference_id, position: states.length, expanded: false },
    ];

    await saveSceneReferenceState(refType, updates);
    suggestions = suggestions.filter((x) => x.reference_id !== s.reference_id);
  }

  async function dismissSuggestion(s: ReferenceSuggestion) {
    const scene = referenceScene;
    if (!scene) return;
    try {
      await invoke("dismiss_suggestion", { sceneId: scene.id, referenceId: s.reference_id });
      suggestions = suggestions.filter((x) => x.reference_id !== s.reference_id);
    } catch (e) {
      console.error("Failed to dismiss suggestion:", e);
    }
  }

  async function linkAllSuggestions() {
    for (const s of [...suggestions]) {
      await linkSuggestion(s);
    }
  }

  async function dismissAllSuggestions() {
    for (const s of [...suggestions]) {
      await dismissSuggestion(s);
    }
  }

  async function saveSceneReferenceState(
    referenceType: ReferenceTypeId,
    updates: SceneReferenceStateUpdate[]
  ) {
    const scene = referenceScene;
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
      ui.bumpSceneReferenceRefresh();
    } catch (e) {
      console.error("Failed to save scene reference state:", e);
      sceneReferenceError = e instanceof Error ? e.message : "Failed to save scene reference state";
    }
  }

  function toggleExpanded(id: string) {
    const isExpanded = expandedIds.has(id);
    const nextExpanded = !isExpanded;
    if (referenceScene && activeTab && linkedIds.has(id)) {
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
    if (referenceScene && activeTab) {
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
    if (referenceScene) {
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
    if (!referenceScene) return;
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
    fieldValues?: Record<string, string | null>;
  }) {
    if (!currentProject.value || !editDialog) return;

    try {
      let entityId: string | null = null;

      if (editDialog.mode === "create") {
        entityId = await invoke<string>("create_reference", {
          projectId: currentProject.value.id,
          referenceType: editDialog.referenceType.id,
          reference: {
            name: data.name,
            description: data.description,
            attributes: data.attributes,
          },
        });
      } else if (editDialog.reference) {
        entityId = editDialog.reference.id;
        await invoke("update_reference", {
          referenceId: editDialog.reference.id,
          referenceType: editDialog.referenceType.id,
          reference: {
            name: data.name,
            description: data.description,
            attributes: data.attributes,
          },
        });
      }

      if (data.fieldValues && entityId) {
        for (const [defId, value] of Object.entries(data.fieldValues)) {
          if (value === null || value === undefined || value === "") {
            await invoke("clear_field_value", {
              fieldDefinitionId: defId,
              entityId,
            });
          } else {
            await invoke("set_field_value", {
              fieldDefinitionId: defId,
              entityId,
              value,
            });
          }
        }
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
      if (referenceScene) {
        loadSceneReferenceState(referenceScene.id);
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
      if (referenceScene) {
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
    const sceneId = referenceScene?.id ?? null;
    if (sceneId === lastSceneId) return;
    lastSceneId = sceneId;
    if (sceneId) {
      loadSceneReferenceState(sceneId);
      loadSuggestions(sceneId);
    } else {
      sceneReferenceStates = [];
      sceneReferenceError = null;
      sceneReferenceLoading = false;
      syncExpandedIdsFromState([]);
      suggestions = [];
    }
  });

  $effect(() => {
    const project = currentProject.value;
    untrack(() => {
      const alreadyRefreshed = project === explicitlyRefreshedProject;
      explicitlyRefreshedProject = null;
      if (project && !alreadyRefreshed) void loadReferences();
    });
  });

  onMount(() => {
    const handler = () => {
      const sceneId = referenceScene?.id;
      if (sceneId) loadSuggestions(sceneId);
    };
    const allHandler = async () => {
      const projectId = currentProject.value?.id;
      if (!projectId) return;
      try {
        await invoke("detect_all_references", { projectId });
        const sceneId = referenceScene?.id;
        if (sceneId) loadSuggestions(sceneId);
      } catch (e) {
        console.error("Failed to detect all references:", e);
      }
    };
    window.addEventListener("kindling:detectReferences", handler);
    window.addEventListener("kindling:detectAllReferences", allHandler);
    return () => {
      window.removeEventListener("kindling:detectReferences", handler);
      window.removeEventListener("kindling:detectAllReferences", allHandler);
    };
  });
</script>

<aside
  class="bg-press-surface border-l border-press-border flex flex-col h-full relative"
  class:w-0={collapsed}
  class:min-w-0={collapsed}
  class:overflow-hidden={collapsed}
  class:opacity-0={collapsed}
  class:border-l-0={collapsed}
  class:p-0={collapsed}
  class:transition-all={!isResizing}
  class:duration-200={!isResizing}
  style={embedded
    ? "width: 100%; min-height: 0; border-left: 0"
    : collapsed
      ? ""
      : `width: ${ui.referencesPanelWidth}px`}
>
  <!-- Resize handle -->
  {#if !embedded && !collapsed}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="absolute left-0 top-0 bottom-0 w-1 cursor-ew-resize hover:bg-press-accent-text active:bg-press-accent transition-colors z-press-raised focus:outline-none focus:bg-press-accent"
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
  <div class="border-b border-press-border">
    <div class="flex items-center justify-between px-4 py-2">
      <h2 class="text-press-ui font-heading font-medium text-press-text">References</h2>
      <div class="flex items-center gap-1">
        <Tooltip text="Copy references from project…" position="bottom">
          <button
            onclick={() => (copyDestination = currentProject.value)}
            disabled={!currentProject.value || editDialog !== null}
            class="text-press-muted hover:text-press-text p-1 disabled:cursor-not-allowed"
            aria-label="Copy references from project…"
          >
            <Copy class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Add Reference button -->
        <Tooltip
          text={activeTab
            ? `Add ${getReferenceTypeOption(activeTab)?.label ?? "Reference"}`
            : "Add reference"}
          position="bottom"
        >
          <button
            onclick={openCreateDialog}
            class="text-press-muted hover:text-press-text p-1 disabled:cursor-not-allowed"
            aria-label="Add reference"
            data-testid="add-reference-button"
            disabled={!activeTab}
          >
            <Plus class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Reference Types settings -->
        <Tooltip text="Reference types" position="bottom">
          <button
            onclick={openReferenceTypeSettings}
            class="text-press-muted hover:text-press-text p-1"
            aria-label="Reference types settings"
          >
            <Settings class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Collapse All button -->
        <Tooltip text="Collapse all" position="bottom">
          <button
            onclick={collapseAll}
            class="text-press-muted hover:text-press-text p-1"
            aria-label="Collapse all"
          >
            <ListChevronsDownUp class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Sort Alphabetically button -->
        <Tooltip text="Sort A-Z" position="bottom">
          <button
            onclick={sortAlphabetically}
            class="text-press-muted hover:text-press-text p-1"
            aria-label="Sort alphabetically"
          >
            <ArrowDownAZ class="w-4 h-4" />
          </button>
        </Tooltip>
        <!-- Close panel button -->
        {#if !embedded}<Tooltip text="Collapse panel" position="bottom">
            <button
              onclick={toggleReferencesPanel}
              class="text-press-muted hover:text-press-text p-1"
              aria-label="Collapse references panel"
            >
              <ChevronsRight class="w-4 h-4" />
            </button>
          </Tooltip>{/if}
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-t border-press-border overflow-x-auto">
      {#each referenceTypeOptions as typeOption (typeOption.id)}
        <button
          onclick={() => (activeTab = typeOption.id)}
          class="flex-1 px-4 py-2 text-press-ui font-medium transition-colors whitespace-nowrap"
          class:text-press-accent-text={activeTab === typeOption.id}
          class:border-b-2={activeTab === typeOption.id}
          class:border-press-accent={activeTab === typeOption.id}
          class:text-press-muted={activeTab !== typeOption.id}
        >
          {typeOption.label} ({getReferenceCount(typeOption.id)})
        </button>
      {/each}
    </div>
  </div>

  <!-- Suggested References -->
  {#if referenceScene && (suggestions.length > 0 || suggestionsLoading)}
    <div class="border-t border-press-border">
      <button
        onclick={() => (suggestionsOpen = !suggestionsOpen)}
        class="flex items-center gap-1.5 w-full px-3 py-1.5 text-press-eyebrow font-medium text-press-muted hover:text-press-text transition-colors cursor-pointer"
      >
        {#if suggestionsOpen}
          <ChevronDown class="w-3 h-3" />
        {:else}
          <ChevronRight class="w-3 h-3" />
        {/if}
        <Zap class="w-3 h-3 text-press-warning" />
        Suggested
        {#if suggestions.length > 0}
          <span
            class="ml-auto bg-press-warning-wash text-press-warning text-press-eyebrow px-1.5 py-0.5 rounded-full"
          >
            {suggestions.length}
          </span>
        {/if}
      </button>
      {#if suggestionsOpen}
        <div class="px-2 pb-2 space-y-1">
          {#if suggestionsLoading}
            <p class="text-press-eyebrow text-press-muted px-1 py-2">Detecting...</p>
          {:else}
            {#if suggestions.length > 1}
              <div class="flex items-center justify-end gap-2 px-1 pb-1">
                <button
                  onclick={linkAllSuggestions}
                  class="text-press-eyebrow text-press-accent-text hover:text-press-accent-text transition-colors"
                >
                  Link All
                </button>
                <button
                  onclick={dismissAllSuggestions}
                  class="text-press-eyebrow text-press-muted hover:text-press-text transition-colors"
                >
                  Dismiss All
                </button>
              </div>
            {/if}
            {#each suggestions as s}
              <SuggestionCard
                suggestion={s}
                onLink={linkSuggestion}
                onDismiss={dismissSuggestion}
              />
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Content -->
  <div class="flex-1 overflow-y-auto p-2">
    {#if loading}
      <div class="flex items-center justify-center p-4">
        <span class="text-press-muted text-press-ui">Loading...</span>
      </div>
    {:else if !activeTab}
      <div class="flex items-center justify-center p-4">
        <span class="text-press-muted text-press-ui">
          No reference types enabled. Use the settings cog to enable them.
        </span>
      </div>
    {:else if activeItems.length === 0}
      <div class="flex items-center justify-center p-4">
        <span class="text-press-muted text-press-ui">
          No {activeTypeOption?.label.toLowerCase() ?? "references"}
        </span>
      </div>
    {:else}
      {#if referenceScene}
        <div
          class="flex items-center justify-between px-1 pb-2 text-press-eyebrow text-press-muted"
        >
          <span class="uppercase tracking-wide">Linked to this scene</span>
          {#if sceneReferenceLoading}
            <span>Loading…</span>
          {/if}
        </div>
        {#if sceneReferenceError}
          <div class="px-1 pb-2 text-press-eyebrow text-press-error">{sceneReferenceError}</div>
        {:else if linkedItems.length === 0 && !sceneReferenceLoading}
          <div class="px-1 pb-2 text-press-eyebrow text-press-muted">
            No references linked to this scene yet.
          </div>
        {/if}
      {/if}
      <div class="space-y-2">
        {#each activeItems as reference, index (reference.id)}
          {@const isExpanded = expandedIds.has(reference.id)}
          {@const notes = getNotes(reference.attributes)}
          {@const isLinked = referenceScene ? linkedIds.has(reference.id) : false}
          {@const canDrag = !referenceScene || isLinked}
          {#if referenceScene && linkedItems.length > 0 && index === linkedItems.length}
            <div
              class="border-t border-press-border pt-3 mt-3 text-press-eyebrow text-press-muted uppercase tracking-wide"
            >
              All references
            </div>
          {/if}
          <div
            class="bg-press-sunken rounded-lg overflow-hidden"
            class:ring-2={dragOverId === reference.id}
            class:ring-press-focus={dragOverId === reference.id}
            style:opacity={draggedId === reference.id ? 0.5 : 1}
            data-drag-item={reference.id}
            role="listitem"
          >
            <div class="w-full flex items-center gap-3 p-3 hover:bg-press-sunken transition-colors">
              <!-- Drag handle -->
              <div
                class="text-press-muted cursor-grab active:cursor-grabbing shrink-0 hover:text-press-muted"
                class:opacity-40={!canDrag}
                onmousedown={(e) => onDragHandleMouseDown(e, reference.id, canDrag)}
                role="button"
                tabindex="-1"
                aria-label="Drag to reorder"
              >
                <GripVertical class="w-4 h-4" />
              </div>
              {#if referenceScene}
                <Tooltip text={isLinked ? "Unlink from scene" : "Link to scene"} position="bottom">
                  <button
                    onclick={() => toggleSceneLink(reference)}
                    class={`shrink-0 inline-flex items-center gap-1 rounded border px-2 py-1 text-press-eyebrow transition-colors ${
                      isLinked
                        ? "border-press-accent text-press-accent-text hover:border-press-accent"
                        : "border-press-border text-press-muted hover:text-press-text hover:border-press-accent"
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
                class="flex-1 min-w-0 flex items-center gap-3 text-left"
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
                  <p class="text-press-text font-medium text-press-ui truncate">{reference.name}</p>
                  {#if reference.description}
                    <p class="text-press-muted text-press-eyebrow truncate">
                      {stripHtml(reference.description)}
                    </p>
                  {/if}
                </div>
                <ChevronDown
                  class="w-4 h-4 text-press-muted transition-transform shrink-0 {isExpanded
                    ? 'rotate-180'
                    : ''}"
                />
              </button>
            </div>

            {#if isExpanded}
              {@const refFieldDefs = (activeTab ? (fieldDefsMap[activeTab] ?? []) : []).filter(
                (d) => d.visible
              )}
              {@const refFieldValues = fieldValuesMap[reference.id] ?? {}}
              {@const hasFieldValues = refFieldDefs.some(
                (d) => refFieldValues[d.id] != null && refFieldValues[d.id] !== ""
              )}
              {@const attributes = formatAttributes(reference.attributes)
                .filter(
                  ([key]) =>
                    !refFieldDefs.some(
                      (def) =>
                        def.name.trim().toLowerCase() === key.trim().toLowerCase() &&
                        refFieldValues[def.id] != null &&
                        refFieldValues[def.id] !== ""
                    )
                )
                .sort(([a], [b]) => a.localeCompare(b))}
              <div class="px-3 pb-3 border-t border-press-border">
                {#if reference.description}
                  <div
                    class="font-prose text-press-text text-press-body mt-3 leading-relaxed max-w-press-measure wrap-break-word [&>p]:mb-2 [&>p:last-child]:mb-0 [&_strong]:font-semibold [&_em]:italic"
                  >
                    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                    {@html reference.description}
                  </div>
                {/if}

                {#if notes}
                  <p
                    class="font-prose text-press-text text-press-body mt-3 leading-relaxed max-w-press-measure wrap-break-word"
                  >
                    {notes}
                  </p>
                {/if}

                {#if hasFieldValues}
                  <div class="mt-3 space-y-1.5">
                    {#each refFieldDefs as def (def.id)}
                      {@const fv = refFieldValues[def.id]}
                      {#if fv != null && fv !== ""}
                        <div class="flex gap-2 text-press-eyebrow">
                          <span class="text-press-muted font-medium shrink-0">{def.name}:</span>
                          <span class="text-press-text wrap-break-word">
                            {#if def.field_type === "checkbox"}
                              {fv === "true" ? "Yes" : "No"}
                            {:else if def.field_type === "multiselect" || def.field_type === "multi_select"}
                              {(() => {
                                try {
                                  return (JSON.parse(fv) as string[]).join(", ");
                                } catch {
                                  return fv;
                                }
                              })()}
                            {:else if def.field_type === "url"}
                              <a
                                href={fv}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-press-accent-text hover:underline">{fv}</a
                              >
                            {:else}
                              {fv}
                            {/if}
                          </span>
                        </div>
                      {/if}
                    {/each}
                  </div>
                {/if}

                {#if attributes.length > 0}
                  <div class="mt-3 space-y-1.5">
                    {#each attributes as [key, value] (key)}
                      <div class="flex gap-2 text-press-eyebrow">
                        <span class="text-press-muted font-medium shrink-0">{key}:</span>
                        <span class="text-press-text wrap-break-word">{value}</span>
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if currentProject.value}
                  <div class="mt-3">
                    <span class="text-press-eyebrow text-press-muted font-medium block mb-1"
                      >Tags</span
                    >
                    <TagSelector
                      projectId={currentProject.value.id}
                      entityType={activeTypeOption
                        ? REFERENCE_FIELD_TYPES[activeTypeOption.id]
                        : "item"}
                      entityId={reference.id}
                      {allTags}
                      entityTagIds={entityTagIds[reference.id] ?? []}
                      onTagsChanged={() => loadReferences()}
                    />
                  </div>
                {/if}

                {#if !reference.description && !notes && attributes.length === 0 && !hasFieldValues}
                  <p class="text-press-muted text-press-ui mt-3 italic">No additional details</p>
                {/if}

                <div class="flex items-center gap-2 mt-4">
                  <Tooltip text="Edit" position="bottom">
                    <button
                      onclick={() => openEditDialog(reference)}
                      class="text-press-muted hover:text-press-text p-1"
                      aria-label="Edit reference"
                    >
                      <Pencil class="w-4 h-4" />
                    </button>
                  </Tooltip>
                  <Tooltip text="Delete" position="bottom">
                    <button
                      onclick={() => (deleteTarget = reference)}
                      class="text-press-muted hover:text-press-error p-1"
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
      class="fixed right-0 top-1/2 -translate-y-1/2 bg-press-surface p-2 rounded-l-lg text-press-muted hover:text-press-text z-press-raised"
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
    projectId={currentProject.value?.id ?? ""}
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

{#if showReferenceTypeSettings}
  <div
    class="fixed inset-0 z-press-modal flex items-center justify-center bg-press-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="reference-types-title"
    tabindex="-1"
  >
    <div
      class="app-dialog-surface bg-press-surface rounded-lg shadow-press-overlay w-full max-w-md mx-4 overflow-hidden"
    >
      <div class="flex items-center justify-between px-4 py-3 border-b border-press-border">
        <h2 id="reference-types-title" class="text-press-body-lg font-medium text-press-text">
          Reference Types
        </h2>
        <Tooltip text="Close" position="left">
          <button
            type="button"
            onclick={closeReferenceTypeSettings}
            class="p-1 text-press-muted hover:text-press-text transition-colors rounded"
            aria-label="Close"
          >
            <ChevronsRight class="w-4 h-4" />
          </button>
        </Tooltip>
      </div>
      <div class="p-4 space-y-3">
        <p class="text-press-ui text-press-muted">
          Choose which reference types appear in this project’s References panel.
        </p>
        <div class="space-y-2">
          {#each REFERENCE_TYPE_OPTIONS as option (option.id)}
            <label class="flex items-center gap-2 text-press-ui text-press-text">
              <input
                type="checkbox"
                class="accent-accent"
                checked={referenceTypeSelection.includes(option.id)}
                disabled={referenceTypeSaving}
                onclick={() => toggleReferenceType(option.id)}
              />
              <span>{option.label}</span>
            </label>
          {/each}
        </div>
        {#if referenceTypeError}
          <p class="text-press-ui text-press-error">{referenceTypeError}</p>
        {/if}
      </div>
      <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-press-border">
        <button
          type="button"
          onclick={closeReferenceTypeSettings}
          class="px-4 py-2 text-press-ui text-press-muted hover:text-press-text transition-colors"
          disabled={referenceTypeSaving}
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={saveReferenceTypeSettings}
          class="px-4 py-2 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text transition-colors"
          disabled={referenceTypeSaving}
        >
          {referenceTypeSaving ? "Saving..." : "Save"}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if copyDestination && currentProject.value?.id === copyDestination.id}
  <CopyReferencesDialog
    destination={copyDestination}
    onClose={() => (copyDestination = null)}
    onComplete={referencesCopied}
  />
{/if}
