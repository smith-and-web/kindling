<script lang="ts">
  import { REFERENCE_FIELD_TYPES } from "../referenceTypes";
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2, Plus, Trash2 } from "lucide-svelte";
  import type { ReferenceItem, FieldDefinition, FieldValue } from "../types";
  import type { ReferenceTypeOption } from "../referenceTypes";
  import FieldRenderer from "./FieldRenderer.svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";

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

  const entityTypeMap = REFERENCE_FIELD_TYPES;

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

  let nameError = $state<string | null>(null);

  function handleFieldChange(defId: string, value: string | null) {
    fieldValueMap = { ...fieldValueMap, [defId]: value };
  }

  async function handleSave() {
    const trimmedName = name.trim();
    if (!trimmedName) {
      nameError = "Enter a name to continue.";
      nameInput?.focus();
      return;
    }
    nameError = null;

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
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && !saving) {
      event.preventDefault();
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

<!-- Escape is the keyboard equivalent of the backdrop click; modalFocus handles it. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, onKeydown: handleKeydown }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="reference-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell">
    <DialogHeader
      title={`${reference ? "Edit" : "Add"} ${referenceType.singular}`}
      titleId="reference-dialog-title"
      {onClose}
      closeLabel="Close"
      closeTestId="reference-close"
    />

    <div class="ka-dialog-body reference-form">
      <div class="ka-group">
        <div class="ka-field od-field">
          <label for="reference-name">Name</label>
          <input
            id="reference-name"
            bind:this={nameInput}
            bind:value={name}
            type="text"
            placeholder="Enter name…"
            disabled={saving}
            aria-invalid={nameError ? true : undefined}
            aria-describedby={nameError ? "ref-name-error" : undefined}
            oninput={() => (nameError = null)}
          />
          {#if nameError}
            <p id="ref-name-error" class="ka-error">{nameError}</p>
          {/if}
        </div>

        <div class="ka-field od-field">
          <label for="reference-description">
            Description <span class="ka-optional">(optional)</span>
          </label>
          <textarea
            id="reference-description"
            rows="4"
            bind:value={description}
            placeholder="Optional description"
            disabled={saving}
          ></textarea>
        </div>

        <div class="ka-field od-field">
          <label for="reference-notes">Notes <span class="ka-optional">(optional)</span></label>
          <textarea
            id="reference-notes"
            rows="3"
            bind:value={notes}
            placeholder="Optional notes"
            disabled={saving}
          ></textarea>
        </div>
      </div>

      {#if !fieldsLoading && visibleFieldDefs.length > 0}
        <section class="ka-group" aria-labelledby="reference-custom-fields">
          <h3 id="reference-custom-fields" class="ka-group-title">Custom fields</h3>
          <div class="reference-fields">
            {#each visibleFieldDefs as def (def.id)}
              <FieldRenderer
                definition={def}
                value={fieldValueMap[def.id] ?? null}
                disabled={saving}
                onChange={(v) => handleFieldChange(def.id, v)}
              />
            {/each}
          </div>
        </section>
      {/if}

      <section class="ka-group" aria-labelledby="reference-legacy">
        <div class="ka-group-title">
          <h3 id="reference-legacy" class="reference-legacy-title">Legacy attributes</h3>
          <button
            type="button"
            onclick={addAttributeRow}
            class="ka-button ka-button--ghost"
            disabled={saving}
          >
            <Plus class="w-5 h-5" aria-hidden="true" />
            Add attribute
          </button>
        </div>
        {#if attributeRows.length === 0}
          <p class="ka-help">No attributes yet.</p>
        {:else}
          {#each attributeRows as row, index (row.id)}
            <div class="reference-attribute">
              <input
                type="text"
                bind:value={row.key}
                placeholder="Key"
                aria-label={`Attribute ${index + 1} key`}
                disabled={saving}
              />
              <input
                type="text"
                bind:value={row.value}
                placeholder="Value"
                aria-label={`Attribute ${index + 1} value`}
                disabled={saving}
              />
              <button
                type="button"
                onclick={() => removeAttributeRow(row.id)}
                class="ka-button ka-button--ghost ka-icon-button reference-remove"
                aria-label="Remove attribute"
                title="Remove attribute"
                disabled={saving}
              >
                <Trash2 class="w-5 h-5" aria-hidden="true" />
              </button>
            </div>
          {/each}
        {/if}
      </section>

      {#if error}
        <p class="ka-error" role="alert">{error}</p>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <button
        type="button"
        onclick={onClose}
        class="ka-button ka-button--secondary"
        disabled={saving}
      >
        Cancel
      </button>
      <button
        data-testid="reference-save"
        type="button"
        onclick={handleSave}
        class="ka-button"
        disabled={saving}
        aria-busy={saving || undefined}
      >
        {#if saving}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Saving…
        {:else}
          Save
        {/if}
      </button>
    </footer>
  </div>
</div>

<style>
  .reference-form {
    display: grid;
  }
  .reference-form .ka-group > .ka-field + .ka-field {
    margin-top: 4px;
  }
  /* Short fields (Role, Age) pair up; groups of options take the full row. */
  .reference-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-s);
  }
  .reference-fields > :global(.field-renderer:has(textarea, .ka-checks)) {
    grid-column: 1 / -1;
  }
  .reference-legacy-title {
    margin: 0;
    font: inherit;
  }
  .reference-attribute {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .reference-attribute input {
    flex: 1;
    min-width: 0;
  }
  @media (hover: hover) {
    .reference-remove:hover {
      color: var(--color-error);
    }
  }
</style>
