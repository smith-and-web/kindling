<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { GripVertical, Pencil, Plus, Trash2, X, Loader2, Check } from "lucide-svelte";
  import type { FieldDefinition, FieldType, FieldEntityType } from "../types";
  import Tooltip from "./Tooltip.svelte";

  let {
    projectId,
    entityType,
    entityLabel,
  }: {
    projectId: string;
    entityType: FieldEntityType;
    entityLabel: string;
  } = $props();

  let definitions = $state<FieldDefinition[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editingDef = $state<Partial<FieldDefinition> | null>(null);
  let editMode = $state<"create" | "edit">("create");
  let saving = $state(false);

  const FIELD_TYPES: { value: FieldType; label: string }[] = [
    { value: "text", label: "Text" },
    { value: "number", label: "Number" },
    { value: "date", label: "Date" },
    { value: "select", label: "Dropdown" },
    { value: "multiselect", label: "Multi-select" },
    { value: "checkbox", label: "Checkbox" },
    { value: "url", label: "URL" },
  ];

  async function loadDefinitions() {
    loading = true;
    error = null;
    try {
      definitions = await invoke("get_field_definitions", {
        projectId,
        entityType,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (projectId && entityType) {
      loadDefinitions();
    }
  });

  function openCreateForm() {
    editMode = "create";
    editingDef = {
      name: "",
      field_type: "text" as FieldType,
      options: null,
      default_value: null,
      required: false,
      visible: true,
    };
  }

  function openEditForm(def: FieldDefinition) {
    editMode = "edit";
    editingDef = {
      ...def,
      field_type: def.field_type === "multi_select" ? "multiselect" : def.field_type,
    };
  }

  function cancelEdit() {
    editingDef = null;
  }

  async function saveDefinition() {
    if (!editingDef || !editingDef.name?.trim()) return;

    saving = true;
    try {
      if (editMode === "create") {
        await invoke("create_field_definition", {
          projectId,
          definition: {
            entity_type: entityType,
            name: editingDef.name.trim(),
            field_type: editingDef.field_type ?? "text",
            options: editingDef.options || null,
            default_value: editingDef.default_value || null,
            required: editingDef.required ?? false,
            visible: editingDef.visible ?? true,
          },
        });
      } else if (editingDef.id) {
        await invoke("update_field_definition", {
          projectId,
          definitionId: editingDef.id,
          definition: {
            name: editingDef.name.trim(),
            field_type: editingDef.field_type ?? "text",
            options: editingDef.options || null,
            default_value: editingDef.default_value || null,
            required: editingDef.required ?? false,
            visible: editingDef.visible ?? true,
          },
        });
      }
      editingDef = null;
      await loadDefinitions();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteDefinition(id: string) {
    try {
      await invoke("delete_field_definition", { projectId, definitionId: id });
      await loadDefinitions();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  let optionsText = $state("");

  $effect(() => {
    if (editingDef?.options) {
      try {
        const arr = JSON.parse(editingDef.options) as string[];
        optionsText = arr.join(", ");
      } catch {
        optionsText = editingDef.options;
      }
    } else {
      optionsText = "";
    }
  });

  function handleOptionsChange(e: Event) {
    const text = (e.target as HTMLInputElement).value;
    optionsText = text;
    if (editingDef) {
      const arr = text
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      editingDef.options = arr.length > 0 ? JSON.stringify(arr) : null;
    }
  }

  const needsOptions = $derived(
    editingDef?.field_type === "select" || editingDef?.field_type === "multiselect"
  );

  const inputClass =
    "w-full bg-press-sunken text-press-text border border-press-border rounded-lg px-3 py-2 text-press-ui focus:outline-none focus:border-press-accent";
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <h3 class="text-press-ui font-medium text-press-text">{entityLabel} Fields</h3>
    <button
      type="button"
      onclick={openCreateForm}
      class="text-press-muted hover:text-press-text text-press-eyebrow flex items-center gap-1"
      disabled={!!editingDef}
    >
      <Plus class="w-3 h-3" />
      Add field
    </button>
  </div>

  {#if loading}
    <p class="text-press-eyebrow text-press-muted">Loading fields...</p>
  {:else if error}
    <p class="text-press-eyebrow text-press-error">{error}</p>
  {:else if definitions.length === 0 && !editingDef}
    <p class="text-press-eyebrow text-press-muted">No custom fields defined yet.</p>
  {:else}
    <div class="space-y-1">
      {#each definitions as def}
        <div class="flex items-center gap-2 py-1.5 px-2 bg-press-sunken rounded-lg text-press-ui">
          <GripVertical class="w-3.5 h-3.5 text-press-muted shrink-0" />
          <span class="flex-1 text-press-text truncate">{def.name}</span>
          <span class="text-press-eyebrow text-press-muted capitalize">{def.field_type}</span>
          {#if def.required}
            <span class="text-press-eyebrow text-press-error">req</span>
          {/if}
          <Tooltip text="Edit" position="bottom">
            <button
              onclick={() => openEditForm(def)}
              class="p-1 text-press-muted hover:text-press-text"
              aria-label="Edit field"
            >
              <Pencil class="w-3.5 h-3.5" />
            </button>
          </Tooltip>
          <Tooltip text="Delete" position="bottom">
            <button
              onclick={() => deleteDefinition(def.id)}
              class="p-1 text-press-muted hover:text-press-error"
              aria-label="Delete field"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </Tooltip>
        </div>
      {/each}
    </div>
  {/if}

  {#if editingDef}
    <div class="bg-press-sunken rounded-lg p-3 space-y-3 border border-press-accent">
      <div class="flex items-center justify-between">
        <span class="text-press-ui font-medium text-press-text">
          {editMode === "create" ? "New Field" : "Edit Field"}
        </span>
        <button onclick={cancelEdit} class="p-1 text-press-muted hover:text-press-text">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div>
        <label class="block text-press-eyebrow text-press-muted mb-1" for="field-name">Name</label>
        <input
          id="field-name"
          type="text"
          bind:value={editingDef.name}
          class={inputClass}
          placeholder="e.g. Age, Genre, Status..."
          disabled={saving}
        />
      </div>

      <div>
        <label class="block text-press-eyebrow text-press-muted mb-1" for="field-type">Type</label>
        <select
          id="field-type"
          bind:value={editingDef.field_type}
          class={inputClass}
          disabled={saving}
        >
          {#each FIELD_TYPES as ft}
            <option value={ft.value}>{ft.label}</option>
          {/each}
        </select>
      </div>

      {#if needsOptions}
        <div>
          <label class="block text-press-eyebrow text-press-muted mb-1" for="field-options">
            Options (comma-separated)
          </label>
          <input
            id="field-options"
            type="text"
            value={optionsText}
            oninput={handleOptionsChange}
            class={inputClass}
            placeholder="Option A, Option B, Option C"
            disabled={saving}
          />
        </div>
      {/if}

      <div>
        <label class="block text-press-eyebrow text-press-muted mb-1" for="field-default">
          Default value
        </label>
        <input
          id="field-default"
          type="text"
          bind:value={editingDef.default_value}
          class={inputClass}
          placeholder="Optional"
          disabled={saving}
        />
      </div>

      <div class="flex items-center gap-4">
        <label
          class="inline-flex items-center gap-1.5 text-press-ui text-press-text cursor-pointer"
        >
          <input
            type="checkbox"
            class="accent-accent"
            bind:checked={editingDef.required}
            disabled={saving}
          />
          Required
        </label>
        <label
          class="inline-flex items-center gap-1.5 text-press-ui text-press-text cursor-pointer"
        >
          <input
            type="checkbox"
            class="accent-accent"
            bind:checked={editingDef.visible}
            disabled={saving}
          />
          Visible
        </label>
      </div>

      <div class="flex justify-end gap-2">
        <button
          onclick={cancelEdit}
          class="px-3 py-1.5 text-press-ui text-press-muted hover:text-press-text"
          disabled={saving}
        >
          Cancel
        </button>
        <button
          onclick={saveDefinition}
          class="px-3 py-1.5 text-press-ui bg-press-accent text-press-on-accent rounded-lg hover:bg-press-accent-text flex items-center gap-1.5"
          disabled={saving || !editingDef.name?.trim()}
        >
          {#if saving}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
          {:else}
            <Check class="w-3.5 h-3.5" />
          {/if}
          {editMode === "create" ? "Add" : "Save"}
        </button>
      </div>
    </div>
  {/if}
</div>
