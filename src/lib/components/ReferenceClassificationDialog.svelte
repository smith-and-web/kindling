<script lang="ts">
  /* eslint-disable no-undef */
  import { invoke } from "@tauri-apps/api/core";
  import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import type {
    Character,
    Location,
    Project,
    ReferenceItem,
    ReferenceReclassification,
    ReferenceTypeId,
  } from "../types";

  interface Props {
    projectId: string;
    onClose: () => void;
    onComplete: (project: Project) => void;
  }

  let { projectId, onClose, onComplete }: Props = $props();

  type ReferenceRow = {
    id: string;
    name: string;
    description: string | null;
    reference_type: ReferenceTypeId;
    original_type: ReferenceTypeId;
  };

  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let references = $state<ReferenceRow[]>([]);

  const typeOptions = REFERENCE_TYPE_OPTIONS.map((option) => ({
    id: option.id,
    label: option.label,
  }));

  async function loadReferences() {
    loading = true;
    error = null;
    try {
      const [characters, locations, items, objectives, organizations] = await Promise.all([
        invoke<Character[]>("get_characters", { projectId }).then((rows) =>
          rows.map((row) => ({
            id: row.id,
            name: row.name,
            description: row.description,
            reference_type: "characters" as ReferenceTypeId,
            original_type: "characters" as ReferenceTypeId,
          }))
        ),
        invoke<Location[]>("get_locations", { projectId }).then((rows) =>
          rows.map((row) => ({
            id: row.id,
            name: row.name,
            description: row.description,
            reference_type: "locations" as ReferenceTypeId,
            original_type: "locations" as ReferenceTypeId,
          }))
        ),
        invoke<ReferenceItem[]>("get_references", { projectId, referenceType: "items" }).then(
          (rows) =>
            rows.map((row) => ({
              id: row.id,
              name: row.name,
              description: row.description,
              reference_type: "items" as ReferenceTypeId,
              original_type: "items" as ReferenceTypeId,
            }))
        ),
        invoke<ReferenceItem[]>("get_references", {
          projectId,
          referenceType: "objectives",
        }).then((rows) =>
          rows.map((row) => ({
            id: row.id,
            name: row.name,
            description: row.description,
            reference_type: "objectives" as ReferenceTypeId,
            original_type: "objectives" as ReferenceTypeId,
          }))
        ),
        invoke<ReferenceItem[]>("get_references", {
          projectId,
          referenceType: "organizations",
        }).then((rows) =>
          rows.map((row) => ({
            id: row.id,
            name: row.name,
            description: row.description,
            reference_type: "organizations" as ReferenceTypeId,
            original_type: "organizations" as ReferenceTypeId,
          }))
        ),
      ]);

      references = [...characters, ...locations, ...items, ...objectives, ...organizations];
    } catch (e) {
      console.error("Failed to load reference classifications:", e);
      error = e instanceof Error ? e.message : "Failed to load references";
      references = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadReferences();
  });

  function updateReferenceType(id: string, nextType: ReferenceTypeId) {
    references = references.map((row) =>
      row.id === id ? { ...row, reference_type: nextType } : row
    );
  }

  async function saveChanges() {
    const changes: ReferenceReclassification[] = references
      .filter((row) => row.reference_type !== row.original_type)
      .map((row) => ({ reference_id: row.id, new_type: row.reference_type }));

    if (changes.length === 0) {
      onClose();
      return;
    }

    saving = true;
    error = null;
    try {
      const project = await invoke<Project>("reclassify_references", {
        projectId,
        changes,
      });
      onComplete(project);
    } catch (e) {
      console.error("Failed to save reference classifications:", e);
      error = e instanceof Error ? e.message : "Failed to save reference classifications";
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  role="dialog"
  aria-modal="true"
  aria-labelledby="reference-classification-title"
  tabindex="-1"
>
  <div class="bg-bg-panel rounded-lg p-6 max-w-3xl w-full mx-4 shadow-xl">
    <h3
      id="reference-classification-title"
      class="text-lg font-heading font-medium text-text-primary mb-2"
    >
      Review Reference Types
    </h3>
    <p class="text-text-secondary text-sm mb-4">
      We found some references during import. Tweak their type now, or skip to keep our best guess.
    </p>

    {#if loading}
      <div class="text-sm text-text-secondary py-6 text-center">Loading references…</div>
    {:else if error}
      <div class="text-sm text-red-400 py-6 text-center">{error}</div>
    {:else if references.length === 0}
      <div class="text-sm text-text-secondary py-6 text-center">
        No references detected for this project.
      </div>
    {:else}
      <div class="max-h-[60vh] overflow-y-auto border border-bg-card rounded-lg">
        <table class="w-full text-sm">
          <thead class="sticky top-0 bg-bg-panel">
            <tr class="text-left text-text-secondary">
              <th class="px-4 py-3 font-medium">Reference</th>
              <th class="px-4 py-3 font-medium w-48">Type</th>
            </tr>
          </thead>
          <tbody>
            {#each references as reference (reference.id)}
              <tr class="border-t border-bg-card">
                <td class="px-4 py-3 align-top">
                  <div class="text-text-primary font-medium wrap-break-word">
                    {reference.name}
                  </div>
                  {#if reference.description}
                    <div
                      class="text-xs text-text-secondary mt-1 leading-relaxed wrap-break-word [&>p]:mb-2 [&>p:last-child]:mb-0 [&_strong]:font-semibold [&_em]:italic"
                    >
                      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                      {@html reference.description}
                    </div>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  <select
                    class="w-full bg-bg-card border border-bg-card rounded-md px-2 py-1 text-sm text-text-primary"
                    bind:value={reference.reference_type}
                    onchange={(event) =>
                      updateReferenceType(
                        reference.id,
                        (event.currentTarget as HTMLSelectElement).value as ReferenceTypeId
                      )}
                  >
                    {#each typeOptions as option (option.id)}
                      <option value={option.id}>{option.label}</option>
                    {/each}
                  </select>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <div class="flex items-center justify-between mt-6">
      <button
        onclick={onClose}
        class="px-4 py-2 rounded bg-bg-card text-text-primary hover:bg-beat-header transition-colors"
      >
        Skip for now
      </button>
      <button
        onclick={saveChanges}
        class="px-4 py-2 rounded bg-accent text-white hover:bg-accent/90 transition-colors disabled:opacity-60"
        disabled={saving || loading || references.length === 0}
      >
        {saving ? "Saving…" : "Apply changes"}
      </button>
    </div>
  </div>
</div>
