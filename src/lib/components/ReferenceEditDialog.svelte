<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, Plus, Trash2, X } from "lucide-svelte";
  import type { ReferenceItem, FieldDefinition, FieldValue, FieldEntityType } from "../types";
  import type { ReferenceTypeOption } from "../referenceTypes";
  import FieldRenderer from "./FieldRenderer.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    referenceType,
    reference,
    projectId,
    onSave,
    onClose,
  }: {
    referenceType: ReferenceTypeOption;
    reference?: ReferenceItem;
    projectId: string;
    onSave: (data: {
      name: string;
      description: string | null;
      attributes: Record<string, string>;
      fieldValues: Record<string, string | null>;
    }) => Promise<void>;
    onClose: () => void;
  } = $props();

  type AttributeRow = { id: string; key: string; value: string };

  const entityTypeMap: Record<string, FieldEntityType> = {
    characters: "character",
    locations: "location",
    items: "item",
    objectives: "objective",
    organizations: "organization",
  };

  let name = $state("");
  let description = $state("");
  let notes = $state("");
  let attributeRows = $state<AttributeRow[]>([]);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let nameInput: HTMLInputElement | null = $state(null);

  let fieldDefs = $state<FieldDefinition[]>([]);
  let fieldValueMap = $state<Record<string, string | null>>({});
  let fieldsLoading = $state(true);

  const makeRowId = () =>
    globalThis.crypto?.randomUUID?.() ?? `attr-${Date.now()}-${Math.random()}`;

  async function loadFieldDefinitions() {
    fieldsLoading = true;
    try {
      const entityType = entityTypeMap[referenceType.id] ?? referenceType.id;
      fieldDefs = await invoke("get_field_definitions", {
        projectId,
        entityType,
      });

      if (reference) {
        const values: FieldValue[] = await invoke("get_field_values", {
          entityId: reference.id,
        });
        const map: Record<string, string | null> = {};
        for (const v of values) {
          map[v.field_definition_id] = v.value;
        }
        fieldValueMap = map;
      }
    } catch {
      fieldDefs = [];
    } finally {
      fieldsLoading = false;
    }
  }

  $effect(() => {
    name = reference?.name ?? "";
    description = reference?.description ?? "";

    const attrs = reference?.attributes ?? {};
    notes = attrs.notes ?? "";

    attributeRows = Object.entries(attrs)
      .filter(([key]) => key !== "notes")
      .map(([key, value]) => ({ id: makeRowId(), key, value }));
  });

  $effect(() => {
    if (projectId && referenceType) {
      loadFieldDefinitions();
    }
  });

  $effect(() => {
    if (nameInput) {
      nameInput.focus();
      nameInput.select();
    }
  });

  function addAttributeRow() {
    attributeRows = [...attributeRows, { id: makeRowId(), key: "", value: "" }];
  }

  function removeAttributeRow(id: string) {
    attributeRows = attributeRows.filter((row) => row.id !== id);
  }

  function handleFieldChange(defId: string, value: string | null) {
    fieldValueMap = { ...fieldValueMap, [defId]: value };
  }

  async function handleSave() {
    const trimmedName = name.trim();
    if (!trimmedName) {
      error = "Name cannot be empty";
      return;
    }

    saving = true;
    error = null;

    try {
      const attributes: Record<string, string> = {};
      for (const row of attributeRows) {
        const key = row.key.trim();
        if (!key) continue;
        attributes[key] = row.value.trim();
      }
      if (notes.trim()) {
        attributes.notes = notes.trim();
      }

      await onSave({
        name: trimmedName,
        description: description.trim() ? description.trim() : null,
        attributes,
        fieldValues: fieldValueMap,
      });
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save reference";
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && !saving) {
      handleSave();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  const visibleFieldDefs = $derived(fieldDefs.filter((d) => d.visible));
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="reference-dialog-title"
  tabindex="-1"
>
  <!-- Dialog -->
  <div class="bg-bg-panel rounded-lg shadow-xl w-full max-w-xl mx-4 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-bg-card">
      <h2 id="reference-dialog-title" class="text-lg font-medium text-text-primary">
        {reference ? "Edit" : "Add"}
        {referenceType.label}
      </h2>
      <Tooltip text="Close" position="left">
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-text-secondary hover:text-text-primary transition-colors rounded"
          aria-label="Close"
        >
          <X class="w-5 h-5" />
        </button>
      </Tooltip>
    </div>

    <!-- Content -->
    <div class="p-4 space-y-4">
      <div>
        <label for="reference-name" class="block text-sm text-text-secondary mb-1">Name</label>
        <input
          id="reference-name"
          bind:this={nameInput}
          bind:value={name}
          type="text"
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent"
          placeholder="Enter name..."
          disabled={saving}
        />
      </div>

      <div>
        <label for="reference-description" class="block text-sm text-text-secondary mb-1">
          Description
        </label>
        <textarea
          id="reference-description"
          rows="4"
          bind:value={description}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent resize-none"
          placeholder="Optional description"
          disabled={saving}
        ></textarea>
      </div>

      <div>
        <label for="reference-notes" class="block text-sm text-text-secondary mb-1">Notes</label>
        <textarea
          id="reference-notes"
          rows="3"
          bind:value={notes}
          class="w-full bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 focus:outline-none focus:border-accent resize-none"
          placeholder="Optional notes"
          disabled={saving}
        ></textarea>
      </div>

      {#if !fieldsLoading && visibleFieldDefs.length > 0}
        <div class="space-y-3">
          <span class="block text-sm text-text-secondary">Custom Fields</span>
          {#each visibleFieldDefs as def (def.id)}
            <FieldRenderer
              definition={def}
              value={fieldValueMap[def.id] ?? null}
              disabled={saving}
              onChange={(v) => handleFieldChange(def.id, v)}
            />
          {/each}
        </div>
      {/if}

      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-secondary">Additional Attributes</span>
          <button
            type="button"
            onclick={addAttributeRow}
            class="text-text-secondary hover:text-text-primary text-xs flex items-center gap-1"
            disabled={saving}
          >
            <Plus class="w-3 h-3" />
            Add attribute
          </button>
        </div>
        {#if attributeRows.length === 0}
          <p class="text-xs text-text-secondary">No attributes yet.</p>
        {:else}
          <div class="space-y-2">
            {#each attributeRows as row (row.id)}
              <div class="flex gap-2 items-center">
                <input
                  type="text"
                  bind:value={row.key}
                  placeholder="Key"
                  class="flex-1 bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
                  disabled={saving}
                />
                <input
                  type="text"
                  bind:value={row.value}
                  placeholder="Value"
                  class="flex-1 bg-bg-card text-text-primary border border-bg-card rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
                  disabled={saving}
                />
                <button
                  type="button"
                  onclick={() => removeAttributeRow(row.id)}
                  class="text-text-secondary hover:text-red-400 p-1"
                  aria-label="Remove attribute"
                  disabled={saving}
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      {#if error}
        <p class="text-sm text-red-400">{error}</p>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-bg-card">
      <button
        type="button"
        onclick={onClose}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleSave}
        class="px-4 py-2 text-sm bg-accent text-white rounded-lg hover:bg-accent/80 transition-colors disabled:opacity-50 flex items-center gap-2"
        disabled={saving || !name.trim()}
      >
        {#if saving}
          <Loader2 class="w-4 h-4 animate-spin" />
          Saving...
        {:else}
          Save
        {/if}
      </button>
    </div>
  </div>
</div>
