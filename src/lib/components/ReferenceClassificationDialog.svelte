<script lang="ts">
  import { CircleAlert, Loader2 } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { safeProse } from "../utils/safeHtml";
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

  /** Returns true when references are empty and the dialog should auto-close. */
  async function loadReferences(): Promise<boolean> {
    loading = true;
    error = null;
    try {
      const groups = await Promise.all(
        REFERENCE_TYPE_OPTIONS.map(async ({ id }) => {
          const command =
            id === "characters"
              ? "get_characters"
              : id === "locations"
                ? "get_locations"
                : "get_references";
          const rows = await invoke<Array<Character | Location | ReferenceItem>>(command, {
            projectId,
            ...(command === "get_references" ? { referenceType: id } : {}),
          });
          return rows.map((row) => ({
            id: row.id,
            name: row.name,
            description: row.description,
            reference_type: id,
            original_type: id,
          }));
        })
      );
      references = groups.flat();
      return references.length === 0;
    } catch (e) {
      console.error("Failed to load reference classifications:", e);
      error = e instanceof Error ? e.message : "Failed to load references";
      references = [];
      return false;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadReferences().then((shouldAutoClose) => {
      if (shouldAutoClose) onClose();
    });
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
</script>

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="reference-classification-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-default dialog-shell">
    <DialogHeader
      title="Review reference types"
      titleId="reference-classification-title"
      {onClose}
      disabled={saving}
    />
    <div class="ka-dialog-body classify">
      <p class="classify-lede">
        We found some references during import. Tweak their type now, or skip to keep our best
        guess.
      </p>

      {#if loading}
        <div class="ka-progress od-field" role="status">
          <span>Loading references…</span>
          <progress aria-label="Loading references"></progress>
        </div>
      {:else if error}
        <div class="ka-notice ka-notice--error od-row-top" role="alert">
          <CircleAlert class="w-5 h-5" aria-hidden="true" />
          <div class="od-field od-fill">
            <strong
              >{references.length === 0
                ? "Couldn’t load references"
                : "Couldn’t save reference types"}</strong
            >
            <p>{error}</p>
          </div>
        </div>
      {:else if references.length === 0}
        <div class="ka-empty od-stack">
          <h4>No references found</h4>
          <p>No references detected for this project.</p>
        </div>
      {/if}

      {#if !loading && references.length > 0}
        <table class="classify-table">
          <thead>
            <tr>
              <th scope="col">Reference</th>
              <th scope="col" class="classify-type-col">Type</th>
            </tr>
          </thead>
          <tbody>
            {#each references as reference (reference.id)}
              <tr>
                <td>
                  <div class="ka-label classify-name">{reference.name}</div>
                  {#if reference.description}
                    <div class="classify-desc">
                      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                      {@html safeProse(reference.description)}
                    </div>
                  {/if}
                </td>
                <td>
                  <select
                    aria-label={`Type for ${reference.name}`}
                    bind:value={reference.reference_type}
                    disabled={saving}
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
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <button
        type="button"
        onclick={onClose}
        class="ka-button ka-button--secondary"
        disabled={saving}
      >
        Skip for now
      </button>
      <button
        type="button"
        onclick={saveChanges}
        class="ka-button"
        disabled={saving || loading || references.length === 0}
        aria-busy={saving || undefined}
      >
        {#if saving}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Saving…
        {:else}
          Apply changes
        {/if}
      </button>
    </footer>
  </div>
</div>

<style>
  .classify {
    display: grid;
    align-content: start;
    gap: var(--space-s);
  }
  .classify-lede {
    margin: 0;
    max-width: 60ch;
    font: var(--text-ui) / 1.6 var(--font-ui);
    color: var(--color-text);
  }
  .classify-table {
    width: 100%;
    border-collapse: collapse;
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  .classify-table th {
    position: sticky;
    top: calc(-1 * var(--space-m));
    z-index: var(--z-raised);
    padding: var(--space-2xs) var(--space-2xs);
    border-bottom: var(--border-hair);
    background: var(--color-surface);
    font: 600 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    text-align: left;
  }
  .classify-type-col {
    width: 12rem;
  }
  .classify-table td {
    padding: var(--space-xs) var(--space-2xs);
    border-bottom: var(--border-hair);
    vertical-align: top;
  }
  .classify-name {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  .classify-desc {
    margin-top: var(--space-3xs);
    font: var(--text-small) / var(--leading-relaxed) var(--font-ui);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }
  .classify-desc :global(p) {
    margin: 0 0 var(--space-2xs);
  }
  .classify-desc :global(p:last-child) {
    margin-bottom: 0;
  }
  .classify-desc :global(strong) {
    font-weight: 600;
  }
  .classify-desc :global(em) {
    font-style: italic;
  }
  .classify-table select {
    width: 100%;
  }
  .ka-notice p {
    margin: 0;
  }
  .ka-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
  }
  .ka-empty p {
    margin: 0;
  }
</style>
