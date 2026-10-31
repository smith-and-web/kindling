<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Pencil, Plus, Trash2, X, Loader2 } from "lucide-svelte";
  import type { FieldDefinition, FieldType, FieldEntityType } from "../types";
  import { t } from "../i18n";

  let {
    projectId,
    onState,
    onChange,
    entityType,
    entityLabel,
  }: {
    projectId: string;
    onState?: (state: { dirty: boolean; busy: boolean }) => void;
    onChange?: () => void;
    entityType: FieldEntityType;
    entityLabel: string;
  } = $props();

  let definitions = $state<FieldDefinition[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let editingDef = $state<Partial<FieldDefinition> | null>(null);
  let editMode = $state<"create" | "edit">("create");
  let saving = $state(false);
  $effect(() => {
    const state = { dirty: editingDef !== null, busy: saving };
    untrack(() => onState?.(state));
  });
  onDestroy(() => onState?.({ dirty: false, busy: false }));

  const FIELD_TYPES: { value: FieldType; label: string }[] = [
    { value: "text", label: t("references.fieldDefinitionManager.fieldTypeText") },
    { value: "number", label: t("references.fieldDefinitionManager.fieldTypeNumber") },
    { value: "date", label: t("references.fieldDefinitionManager.fieldTypeDate") },
    { value: "select", label: t("references.fieldDefinitionManager.fieldTypeSelect") },
    { value: "multiselect", label: t("references.fieldDefinitionManager.fieldTypeMultiselect") },
    { value: "checkbox", label: t("references.fieldDefinitionManager.fieldTypeCheckbox") },
    { value: "url", label: t("references.fieldDefinitionManager.fieldTypeUrl") },
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
      onChange?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteDefinition(id: string) {
    if (saving) return;
    saving = true;
    try {
      await invoke("delete_field_definition", { projectId, definitionId: id });
      await loadDefinitions();
      onChange?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
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

  // Display the field type the way the editor names it ("Multi-select", not "multi_select").
  function typeLabel(type: string) {
    const normalized = type === "multi_select" ? "multiselect" : type;
    return FIELD_TYPES.find((option) => option.value === normalized)?.label ?? type;
  }
</script>

<section class="fields" aria-labelledby={`fields-${entityType}`}>
  <div class="fields-head">
    <h4 id={`fields-${entityType}`} class="ka-group-title fields-title">
      {t("references.fieldDefinitionManager.fieldsTitle", { label: entityLabel })}
    </h4>
    <button
      type="button"
      onclick={openCreateForm}
      class="ka-button ka-button--secondary"
      disabled={!!editingDef}
    >
      <Plus class="w-5 h-5" aria-hidden="true" />
      {t("references.fieldDefinitionManager.addField")}
    </button>
  </div>

  {#if loading}
    <p class="ka-help" role="status">{t("references.fieldDefinitionManager.loadingFields")}</p>
  {:else if error}
    <p class="ka-error" role="alert">{error}</p>
  {:else if definitions.length === 0 && !editingDef}
    <p class="ka-help">{t("references.fieldDefinitionManager.noFields")}</p>
  {:else}
    <ul class="ka-tagtree">
      {#each definitions as def}
        <li>
          <div class="ka-tagrow">
            <span class="ka-name">{def.name}</span>
            <span class="ka-help fields-type">{typeLabel(def.field_type)}</span>
            {#if def.required}
              <span class="ka-badge">{t("references.fieldDefinitionManager.requiredBadge")}</span>
            {/if}
            <div class="ka-row">
              <button
                type="button"
                onclick={() => openEditForm(def)}
                class="ka-button ka-button--ghost ka-icon-button"
                aria-label={t("references.fieldDefinitionManager.editField")}
                title={t("references.fieldDefinitionManager.editField")}
              >
                <Pencil class="w-5 h-5" aria-hidden="true" />
              </button>
              <button
                type="button"
                onclick={() => deleteDefinition(def.id)}
                class="ka-button ka-button--ghost ka-icon-button fields-delete"
                aria-label={t("references.fieldDefinitionManager.deleteField")}
                title={t("references.fieldDefinitionManager.deleteField")}
              >
                <Trash2 class="w-5 h-5" aria-hidden="true" />
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if editingDef}
    <div class="ka-tagedit">
      <div class="fields-edit-head">
        <h5>
          {editMode === "create"
            ? t("references.fieldDefinitionManager.newField")
            : t("references.fieldDefinitionManager.editFieldTitle")}
        </h5>
        <button
          type="button"
          onclick={cancelEdit}
          class="ka-button ka-button--ghost ka-icon-button"
          aria-label={t("references.fieldDefinitionManager.closeFieldEditor")}
          title={t("references.fieldDefinitionManager.closeFieldEditor")}
        >
          <X class="w-5 h-5" aria-hidden="true" />
        </button>
      </div>

      <div class="ka-field od-field">
        <label for="field-name">{t("references.fieldDefinitionManager.name")}</label>
        <input
          id="field-name"
          type="text"
          bind:value={editingDef.name}
          placeholder={t("references.fieldDefinitionManager.namePlaceholder")}
          disabled={saving}
        />
      </div>

      <div class="ka-field od-field">
        <label for="field-type">{t("references.fieldDefinitionManager.type")}</label>
        <select id="field-type" bind:value={editingDef.field_type} disabled={saving}>
          {#each FIELD_TYPES as ft}
            <option value={ft.value}>{ft.label}</option>
          {/each}
        </select>
      </div>

      {#if needsOptions}
        <div class="ka-field od-field">
          <label for="field-options">{t("references.fieldDefinitionManager.options")}</label>
          <input
            id="field-options"
            type="text"
            value={optionsText}
            oninput={handleOptionsChange}
            placeholder={t("references.fieldDefinitionManager.optionsPlaceholder")}
            disabled={saving}
          />
        </div>
      {/if}

      <div class="ka-field od-field">
        <label for="field-default"
          >{t("references.fieldDefinitionManager.defaultValue")}
          <span class="ka-optional">{t("references.fieldDefinitionManager.optional")}</span></label
        >
        <input
          id="field-default"
          type="text"
          bind:value={editingDef.default_value}
          placeholder={t("references.fieldDefinitionManager.optionalPlaceholder")}
          disabled={saving}
        />
      </div>

      <div class="ka-checks">
        <label class="ka-check">
          <input type="checkbox" bind:checked={editingDef.required} disabled={saving} />
          {t("references.fieldDefinitionManager.required")}
        </label>
        <label class="ka-check">
          <input type="checkbox" bind:checked={editingDef.visible} disabled={saving} />
          {t("references.fieldDefinitionManager.visible")}
        </label>
      </div>

      <div class="ka-row fields-edit-actions">
        <button
          type="button"
          onclick={cancelEdit}
          class="ka-button ka-button--secondary"
          disabled={saving}
        >
          {t("references.fieldDefinitionManager.cancel")}
        </button>
        <button
          type="button"
          onclick={saveDefinition}
          class="ka-button"
          disabled={saving || !editingDef.name?.trim()}
          aria-busy={saving || undefined}
        >
          {#if saving}
            <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          {/if}
          {editMode === "create"
            ? t("references.fieldDefinitionManager.add")
            : t("references.fieldDefinitionManager.save")}
        </button>
      </div>
    </div>
  {/if}
</section>

<style>
  .fields {
    display: grid;
    gap: var(--space-s);
  }
  .fields + :global(.fields) {
    margin-top: var(--space-m);
    padding-top: var(--space-m);
    border-top: var(--border-hair);
  }
  .fields-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-s);
  }
  .fields-title {
    margin: 0;
  }
  /* The next group's rule closes the list; don't double it. */
  .ka-tagtree > li:last-child > .ka-tagrow {
    border-bottom: 0;
  }
  .fields-type {
    margin: 0;
  }
  .fields-edit-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: calc(-1 * var(--space-2xs)) calc(-1 * var(--space-2xs)) 0 0;
  }
  .fields-edit-head h5 {
    margin: 0;
    font: 600 var(--text-base) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .fields-edit-actions {
    justify-content: flex-end;
  }
  @media (hover: hover) {
    .fields-delete:hover {
      color: var(--color-error);
    }
  }
</style>
