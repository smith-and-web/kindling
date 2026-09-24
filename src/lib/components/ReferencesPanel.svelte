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
    PanelRightClose,
    PanelRightOpen,
    GripVertical,
    Link2,
    ListChevronsDownUp,
    Pencil,
    Plus,
    Trash2,
    Unlink,
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
  import { safeProse } from "../utils/safeHtml";

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

  // Tabs: roving focus with arrows, Home and End.
  function onTabKeydown(event: KeyboardEvent) {
    const ids = referenceTypeOptions.map((option) => option.id);
    const index = ids.indexOf(activeTab as ReferenceTypeId);
    let next: number;
    if (event.key === "ArrowRight") next = (index + 1) % ids.length;
    else if (event.key === "ArrowLeft") next = (index - 1 + ids.length) % ids.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = ids.length - 1;
    else return;
    event.preventDefault();
    activeTab = ids[next];
    document.getElementById(`refs-tab-${ids[next]}`)?.focus();
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
  class="refs"
  class:is-collapsed={collapsed}
  class:is-embedded={embedded}
  aria-label="References"
  style={embedded || collapsed ? undefined : `width: ${ui.referencesPanelWidth}px`}
>
  {#if collapsed}
    <div class="refs-rail">
      <button
        type="button"
        onclick={toggleReferencesPanel}
        class="ka-button ka-button--ghost ka-icon-button"
        aria-label={`Expand references panel, ${referenceTypeOptions.reduce((n, o) => n + getReferenceCount(o.id), 0)} references`}
        title="Expand references panel"
      >
        <PanelRightOpen class="w-5 h-5" aria-hidden="true" />
      </button>
    </div>
  {:else}
    <!-- Resize handle -->
    {#if !embedded}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="refs-resize"
        class:is-active={isResizing}
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
    <div class="refs-head">
      <h2>References</h2>
      <div class="refs-head-actions">
        <button
          type="button"
          onclick={() => (copyDestination = currentProject.value)}
          disabled={!currentProject.value || editDialog !== null}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label="Copy references from project…"
          title="Copy references from project…"
        >
          <Copy class="w-5 h-5" aria-hidden="true" />
        </button>
        <button
          type="button"
          onclick={openCreateDialog}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label={activeTab
            ? `Add ${getReferenceTypeOption(activeTab)?.label ?? "reference"}`
            : "Add reference"}
          title={activeTab
            ? `Add ${getReferenceTypeOption(activeTab)?.label ?? "reference"}`
            : "Add reference"}
          data-testid="add-reference-button"
          disabled={!activeTab}
        >
          <Plus class="w-5 h-5" aria-hidden="true" />
        </button>
        <button
          type="button"
          onclick={collapseAll}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label="Collapse all"
          title="Collapse all"
        >
          <ListChevronsDownUp class="w-5 h-5" aria-hidden="true" />
        </button>
        <button
          type="button"
          onclick={sortAlphabetically}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label="Sort alphabetically"
          title="Sort A–Z"
        >
          <ArrowDownAZ class="w-5 h-5" aria-hidden="true" />
        </button>
        {#if !embedded}
          <button
            type="button"
            onclick={toggleReferencesPanel}
            class="ka-button ka-button--ghost ka-icon-button"
            aria-label="Collapse references panel"
            title="Collapse references panel"
          >
            <PanelRightClose class="w-5 h-5" aria-hidden="true" />
          </button>
        {/if}
      </div>
    </div>

    <!-- Tabs -->
    <div class="ka-tablist refs-tabs" role="tablist" aria-label="Reference types">
      {#each referenceTypeOptions as typeOption (typeOption.id)}
        <button
          type="button"
          role="tab"
          id={`refs-tab-${typeOption.id}`}
          aria-selected={activeTab === typeOption.id}
          aria-controls="refs-panel"
          tabindex={activeTab === typeOption.id ? 0 : -1}
          onclick={() => (activeTab = typeOption.id)}
          onkeydown={onTabKeydown}
        >
          {typeOption.label}<span class="refs-tab-count">{getReferenceCount(typeOption.id)}</span>
        </button>
      {/each}
    </div>

    <div
      class="refs-scroll"
      id="refs-panel"
      role="tabpanel"
      aria-labelledby={activeTab ? `refs-tab-${activeTab}` : undefined}
    >
      <!-- Suggested References -->
      {#if referenceScene && (suggestions.length > 0 || suggestionsLoading)}
        <section class="refs-group" aria-labelledby="refs-suggested-title">
          <div class="refs-group-head">
            <button
              type="button"
              onclick={() => (suggestionsOpen = !suggestionsOpen)}
              class="refs-disclose"
              aria-expanded={suggestionsOpen}
            >
              <ChevronRight class="w-4 h-4 refs-chev" aria-hidden="true" />
              <h3 id="refs-suggested-title" class="refs-group-label">Suggested</h3>
              {#if suggestions.length > 0}
                <span class="ka-badge">{suggestions.length}</span>
              {/if}
            </button>
            {#if suggestionsOpen && !suggestionsLoading && suggestions.length > 1}
              <button
                type="button"
                onclick={linkAllSuggestions}
                class="ka-button ka-button--ghost refs-group-action"
              >
                Link all
              </button>
              <button
                type="button"
                onclick={dismissAllSuggestions}
                class="ka-button ka-button--ghost refs-group-action"
              >
                Dismiss all
              </button>
            {/if}
          </div>
          {#if suggestionsOpen}
            {#if suggestionsLoading}
              <div class="ka-progress od-field refs-finding" role="status">
                <span>Finding mentions…</span>
                <progress aria-label="Finding mentions"></progress>
              </div>
            {:else}
              <ul class="refs-list">
                {#each suggestions as s}
                  <li>
                    <SuggestionCard
                      suggestion={s}
                      onLink={linkSuggestion}
                      onDismiss={dismissSuggestion}
                    />
                  </li>
                {/each}
              </ul>
            {/if}
          {/if}
        </section>
      {/if}

      <!-- Content -->
      {#if loading}
        <div class="ka-progress od-field refs-finding" role="status">
          <span>Loading references…</span>
          <progress aria-label="Loading references"></progress>
        </div>
      {:else if !activeTab}
        <p class="ka-help refs-empty">
          No reference types enabled. Enable them in Settings → Reference Types.
        </p>
      {:else if activeItems.length === 0}
        <div class="ka-empty od-stack refs-empty-state">
          <h4>No {activeTypeOption?.label.toLowerCase() ?? "references"} yet</h4>
          <p>
            Add {activeTypeOption?.label.toLowerCase() ?? "references"} to keep track of who and what
            appears in your story.
          </p>
          <button type="button" class="ka-button" onclick={openCreateDialog}>
            <Plus class="w-5 h-5" aria-hidden="true" />
            Add {activeTypeOption?.label.toLowerCase() ?? "reference"}
          </button>
        </div>
      {:else}
        <section class="refs-group">
          {#if referenceScene}
            <div class="refs-group-head">
              <h3 class="refs-group-label refs-group-title">Linked to this scene</h3>
              {#if sceneReferenceLoading}
                <span class="ka-help" role="status">Loading…</span>
              {/if}
            </div>
            {#if sceneReferenceError}
              <p class="ka-error refs-empty" role="alert">{sceneReferenceError}</p>
            {:else if linkedItems.length === 0 && !sceneReferenceLoading}
              <p class="ka-help refs-empty">
                No references linked to this scene yet. Link one below or accept a suggestion.
              </p>
            {/if}
          {/if}
          <div class="refs-list" role="list">
            {#each activeItems as reference, index (reference.id)}
              {@const isExpanded = expandedIds.has(reference.id)}
              {@const notes = getNotes(reference.attributes)}
              {@const isLinked = referenceScene ? linkedIds.has(reference.id) : false}
              {@const canDrag = !referenceScene || isLinked}
              {#if referenceScene && index === linkedItems.length}
                <!-- A list's children must be items, so the heading is one. -->
                <div
                  class="refs-group-head"
                  class:refs-group-divider={linkedItems.length > 0}
                  role="listitem"
                >
                  <h3 class="refs-group-label refs-group-title">All references</h3>
                </div>
              {/if}
              <div
                class="refs-item"
                class:is-drop-target={dragOverId === reference.id}
                class:is-dragging={draggedId === reference.id}
                data-drag-item={reference.id}
                role="listitem"
              >
                <div class="refs-row">
                  <!-- Drag handle -->
                  <div
                    class="refs-grip"
                    class:is-disabled={!canDrag}
                    onmousedown={(e) => onDragHandleMouseDown(e, reference.id, canDrag)}
                    aria-hidden="true"
                    title="Drag to reorder"
                  >
                    <GripVertical class="w-4 h-4" aria-hidden="true" />
                  </div>
                  <!-- Clickable area for expand/collapse -->
                  <button
                    type="button"
                    onclick={() => toggleExpanded(reference.id)}
                    class="refs-main"
                    aria-expanded={isExpanded}
                  >
                    <span class="refs-avatar" aria-hidden="true">
                      {#if ActiveIcon}
                        <ActiveIcon class="w-4 h-4" />
                      {/if}
                    </span>
                    <span class="refs-text">
                      <span class="refs-name">{reference.name}</span>
                      {#if reference.description && !isExpanded}
                        <span class="refs-desc">{stripHtml(reference.description)}</span>
                      {/if}
                    </span>
                    <ChevronDown class="w-4 h-4 refs-chev" aria-hidden="true" />
                  </button>
                  {#if referenceScene}
                    <button
                      type="button"
                      onclick={() => toggleSceneLink(reference)}
                      class="ka-button refs-link"
                      class:ka-button--ghost={isLinked}
                      class:ka-button--secondary={!isLinked}
                      aria-label={isLinked
                        ? `Unlink ${reference.name} from this scene`
                        : `Link ${reference.name} to this scene`}
                      title={isLinked ? "Unlink from scene" : "Link to scene"}
                    >
                      {#if isLinked}
                        <Unlink class="w-4 h-4" aria-hidden="true" />
                        Unlink
                      {:else}
                        <Link2 class="w-4 h-4" aria-hidden="true" />
                        Link
                      {/if}
                    </button>
                  {/if}
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
                  <div class="refs-detail">
                    {#if reference.description}
                      <div class="refs-prose">
                        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                        {@html safeProse(reference.description)}
                      </div>
                    {/if}

                    {#if notes}
                      <p class="refs-prose">{notes}</p>
                    {/if}

                    {#if hasFieldValues || attributes.length > 0}
                      <dl class="ka-facts refs-facts">
                        {#each refFieldDefs as def (def.id)}
                          {@const fv = refFieldValues[def.id]}
                          {#if fv != null && fv !== ""}
                            <div>
                              <dt>{def.name}</dt>
                              <dd>
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
                                  <a href={fv} target="_blank" rel="noopener noreferrer">{fv}</a>
                                {:else}
                                  {fv}
                                {/if}
                              </dd>
                            </div>
                          {/if}
                        {/each}
                        {#each attributes as [key, value] (key)}
                          <div>
                            <dt>{key}</dt>
                            <dd>{value}</dd>
                          </div>
                        {/each}
                      </dl>
                    {/if}

                    {#if currentProject.value}
                      <div class="refs-tags">
                        <span class="refs-group-label">Tags</span>
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
                      <p class="ka-help">No additional details yet.</p>
                    {/if}

                    <div class="refs-detail-actions">
                      <button
                        type="button"
                        onclick={() => openEditDialog(reference)}
                        class="ka-button ka-button--ghost"
                        aria-label="Edit reference"
                      >
                        <Pencil class="w-5 h-5" aria-hidden="true" />
                        Edit
                      </button>
                      <button
                        type="button"
                        onclick={() => (deleteTarget = reference)}
                        class="ka-button ka-button--ghost refs-danger"
                        aria-label="Delete reference"
                      >
                        <Trash2 class="w-5 h-5" aria-hidden="true" />
                        Delete
                      </button>
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}
</aside>

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

{#if copyDestination && currentProject.value?.id === copyDestination.id}
  <CopyReferencesDialog
    destination={copyDestination}
    onClose={() => (copyDestination = null)}
    onComplete={referencesCopied}
  />
{/if}

<style>
  .refs {
    position: relative;
    flex: none;
    container: refs / inline-size;
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--color-surface);
    border-left: var(--border-hair);
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .refs.is-embedded {
    width: 100%;
    border-left: 0;
  }
  .refs > * {
    min-width: 0;
  }
  .refs.is-collapsed {
    display: block;
    width: 56px;
  }
  @media (max-width: 1280px) {
    .refs:not(.is-collapsed, .is-embedded) {
      max-width: 320px;
    }
  }
  .refs-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2xs);
    padding: var(--space-2xs) 6px;
  }
  .refs-resize {
    position: absolute;
    left: -5px;
    top: 0;
    bottom: 0;
    z-index: var(--z-raised);
    width: 9px;
    cursor: col-resize;
  }
  .refs-resize::after {
    content: "";
    position: absolute;
    left: 4px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: transparent;
    transition: background var(--ka-motion);
  }
  .refs-resize:hover::after,
  .refs-resize:focus-visible::after,
  .refs-resize.is-active::after {
    left: 3px;
    width: 2px;
    background: var(--color-accent-text);
  }
  .refs-resize:focus-visible {
    outline: none;
  }

  .refs-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3xs);
    padding: var(--space-2xs) var(--space-3xs) var(--space-2xs) var(--space-s);
  }
  .refs-head h2 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .refs-head-actions {
    display: flex;
    align-items: center;
    margin-left: auto;
  }
  .refs-tabs {
    flex-wrap: nowrap;
    gap: var(--space-3xs);
    padding: 0 var(--space-xs);
    overflow-x: auto;
  }
  .refs-tabs > button {
    white-space: nowrap;
  }
  .refs-tab-count {
    margin-left: 6px;
    font-weight: 400;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .refs-tabs > button[aria-selected="true"] .refs-tab-count {
    color: inherit;
  }

  .refs-scroll {
    min-height: 0;
    overflow-y: auto;
    padding: var(--space-2xs) var(--space-s) var(--space-m);
  }
  .refs-group {
    padding: var(--space-2xs) 0 var(--space-xs);
  }
  .refs-group + .refs-group {
    border-top: var(--border-hair);
  }
  .refs-group-head {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    min-height: var(--control-target);
  }
  .refs-group-divider {
    margin-top: var(--space-2xs);
    border-top: var(--border-hair);
  }
  .refs-disclose {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    flex: 1;
    min-height: var(--control-target);
    padding: 0 0 0 var(--space-3xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }
  .refs-group-label {
    margin: 0;
    font: 500 var(--text-small) / 1.5 var(--font-ui);
    letter-spacing: 0;
    color: var(--color-text-muted);
  }
  .refs-group-title {
    padding-left: var(--space-2xs);
  }
  .refs-group-action {
    flex: none;
    white-space: nowrap;
    padding: var(--space-2xs) 10px;
    font-size: var(--text-small);
  }
  .refs :global(.refs-chev) {
    flex: none;
    color: var(--color-text-muted);
    transition: transform var(--ka-motion) ease-out;
  }
  .refs-disclose[aria-expanded="true"] :global(.refs-chev) {
    transform: rotate(90deg);
  }
  .refs-main[aria-expanded="true"] :global(.refs-chev) {
    transform: rotate(180deg);
  }
  .refs-finding {
    padding: var(--space-2xs);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .refs-empty {
    margin: 0;
    padding: var(--space-2xs) var(--space-2xs) var(--space-3xs);
  }
  .refs-empty-state {
    padding: var(--space-m) var(--space-2xs);
  }
  .refs-empty-state h4 {
    margin: 0;
    font: 550 var(--text-h3) / var(--leading-tight) var(--font-display);
    color: var(--color-text);
  }
  .refs-list {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .refs-item {
    position: relative;
    border-radius: var(--radius-xs);
  }
  /* The row being dragged reads as lifted-out: muted text on the sunken fill. */
  .refs-item.is-dragging {
    background: var(--color-surface-sunken);
    color: var(--color-text-muted);
  }
  .refs-item.is-drop-target::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: -3px;
    height: 2px;
    border-radius: 2px;
    background: var(--color-accent-text);
  }
  .refs-row {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .refs-grip {
    position: absolute;
    left: -14px;
    top: 50%;
    display: flex;
    width: 14px;
    margin-top: -8px;
    color: var(--color-text-muted);
    cursor: grab;
    opacity: 0;
  }
  .refs-grip.is-disabled {
    cursor: not-allowed;
  }
  .refs-row:hover > .refs-grip:not(.is-disabled),
  .refs-row:focus-within > .refs-grip:not(.is-disabled) {
    opacity: 1;
  }
  .refs-main {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    flex: 1;
    min-width: 0;
    min-height: 56px;
    padding: 6px var(--space-2xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.4 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  @media (hover: hover) {
    .refs-main:hover {
      background: var(--color-surface-sunken);
    }
  }
  .refs-avatar {
    display: grid;
    place-items: center;
    flex: none;
    width: 32px;
    height: 32px;
    border: var(--border-hair);
    border-radius: 50%;
    background: var(--color-surface-sunken);
    color: var(--color-text-muted);
  }
  .refs-text {
    display: grid;
    flex: 1;
    min-width: 0;
  }
  .refs-name {
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .refs-desc {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .refs-link {
    flex: none;
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
  }
  .refs-detail {
    display: grid;
    gap: var(--space-xs);
    margin: var(--space-3xs) 0 var(--space-2xs) 52px;
    padding: 0 var(--space-2xs) var(--space-2xs) 0;
  }
  .refs-prose {
    margin: 0;
    max-width: var(--measure);
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .refs-prose :global(p) {
    margin: 0 0 var(--space-2xs);
  }
  .refs-prose :global(p:last-child) {
    margin-bottom: 0;
  }
  .refs-facts {
    font-size: var(--text-small);
  }
  .refs-facts :where(dt, dd) {
    padding-block: 6px;
    overflow-wrap: anywhere;
  }
  .refs-facts a {
    color: var(--color-accent-text);
  }
  .refs-tags {
    display: grid;
    gap: var(--space-3xs);
  }
  .refs-detail-actions {
    display: flex;
    flex-wrap: wrap;
    margin-left: calc(-1 * var(--space-s));
  }
  @media (hover: hover) {
    .refs-danger:hover {
      color: var(--color-error);
    }
  }
  .refs-main:not([aria-expanded="true"]) .refs-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* A narrow panel (it can be resized down in any window size) keeps its
     five 44px actions on one row: the tabs already name what is listed, so
     the title steps back to the accessibility tree, and the tabs and the
     suggested-group actions tighten or wrap instead of clipping. */
  @container refs (max-width: 360px) {
    .refs-head {
      flex-wrap: nowrap;
      padding-left: var(--space-xs);
    }
    .refs-head h2 {
      font-size: var(--text-base);
    }
    .refs-tabs {
      gap: 0;
      padding: 0 var(--space-2xs);
    }
    .refs-tabs > button {
      padding-inline: var(--space-2xs);
    }
    .refs-group-head {
      flex-wrap: wrap;
    }
    .refs-group-head .refs-disclose {
      flex-basis: 100%;
    }
  }
  @container refs (max-width: 320px) {
    .refs-head h2 {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
      white-space: nowrap;
    }
    .refs-tabs > button {
      padding-inline: var(--space-3xs);
    }
  }
</style>
