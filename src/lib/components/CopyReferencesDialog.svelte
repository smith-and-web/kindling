<script lang="ts">
  import { countLabel } from "../utils/plural";
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { CircleCheck, Loader2 } from "lucide-svelte";
  import { REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { copyErrorMessage, copyReferences, previewReferenceCopy } from "../referenceCopy";
  import type {
    Project,
    ReferenceCopyKey,
    ReferenceCopyPreview,
    ReferenceCopyRequest,
    ReferenceCopyResult,
    ReferenceTypeId,
  } from "../types";

  let {
    destination,
    onClose,
    onComplete,
  }: {
    destination: Project;
    onClose: () => void;
    onComplete: (result: ReferenceCopyResult) => Promise<void>;
  } = $props();
  let dialog: HTMLDialogElement;
  let projects = $state<Project[]>([]);
  let sourceId = $state("");
  let search = $state("");
  let selection = $state<ReferenceCopyKey[] | null>(null);
  let keepBoth = $state<ReferenceCopyKey[]>([]);
  let preview = $state<ReferenceCopyPreview | null>(null);
  let loadingProjects = $state(true);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let result = $state<ReferenceCopyResult | null>(null);
  let refreshing = $state(false);
  let refreshFailed = $state(false);
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  let generation = 0;
  let alive = true;
  const selectedCount = $derived(selection?.length ?? 0);
  const source = $derived(projects.find((p) => p.id === sourceId));
  // Why the primary is unavailable, shown beside it (never a silent disabled state).
  const blockedReason = $derived(
    loadingProjects || loading || saving || result || error
      ? null
      : !sourceId
        ? projects.length
          ? "Choose a source project."
          : null
        : !preview
          ? null
          : selectedCount === 0
            ? "Choose references to copy."
            : preview.copied === 0
              ? "Every selected reference matches one already here and will be skipped."
              : null
  );
  const matches = (name: string) =>
    name.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase());
  const key = (row: ReferenceCopyKey): ReferenceCopyKey => ({
    id: row.id,
    reference_type: row.reference_type,
  });
  const same = (a: ReferenceCopyKey, b: ReferenceCopyKey) =>
    a.id === b.id && a.reference_type === b.reference_type;
  function request(): ReferenceCopyRequest {
    return {
      source_project_id: sourceId,
      destination_project_id: destination.id,
      selection,
      keep_both: keepBoth,
    };
  }
  async function loadProjects() {
    loadingProjects = true;
    error = null;
    try {
      const all = await invoke<Project[]>("get_all_projects");
      if (alive) projects = all.filter((p) => p.id !== destination.id);
    } catch (e) {
      if (alive) error = copyErrorMessage(e);
    } finally {
      if (alive) loadingProjects = false;
    }
  }
  async function loadPreview() {
    clearTimeout(previewTimer);
    previewTimer = undefined;
    const current = ++generation;
    if (!sourceId) {
      preview = null;
      loading = false;
      return;
    }
    loading = true;
    error = null;
    try {
      const next = await previewReferenceCopy(request());
      if (!alive || current !== generation) return;
      preview = next;
      // An initial null selection means all, including source-disabled categories.
      if (selection === null) selection = next.references.map(key);
    } catch (e) {
      if (alive && current === generation) {
        error = copyErrorMessage(e);
        preview = null;
      }
    } finally {
      if (alive && current === generation) loading = false;
    }
  }
  function changeSource() {
    selection = null;
    keepBoth = [];
    preview = null;
    search = "";
    void loadPreview();
  }
  function schedulePreview() {
    clearTimeout(previewTimer);
    // Invalidate in-flight results immediately, before the debounce starts a new request.
    generation++;
    loading = true;
    error = null;
    previewTimer = setTimeout(() => void loadPreview(), 200);
  }
  function selectRows(rows: ReferenceCopyKey[], selected: boolean) {
    const current = selection ?? [];
    selection = selected
      ? [...current, ...rows.filter((r) => !current.some((c) => same(c, r))).map(key)]
      : current.filter((c) => !rows.some((r) => same(c, r)));
    keepBoth = keepBoth.filter((k) => selection?.some((s) => same(s, k)));
    schedulePreview();
  }
  function chooseDuplicate(row: ReferenceCopyKey, keep: boolean) {
    keepBoth = keep
      ? [...keepBoth.filter((k) => !same(k, row)), key(row)]
      : keepBoth.filter((k) => !same(k, row));
    schedulePreview();
  }
  async function refreshCopied() {
    if (!result) return;
    refreshing = true;
    refreshFailed = false;
    error = null;
    try {
      await onComplete(result);
    } catch (e) {
      refreshFailed = true;
      error = `The copy completed, but the panel could not refresh. ${copyErrorMessage(e)}`;
    } finally {
      refreshing = false;
    }
  }
  async function submit() {
    if (!preview || loading || saving || result || preview.copied === 0) return;
    saving = true;
    error = null;
    try {
      result = await copyReferences(request(), preview.revision);
      await refreshCopied();
    } catch (e) {
      error = copyErrorMessage(e);
      // Every failed commit needs a new reviewed preview, including a lost response.
      // Never automatically retry an operation that could have committed.
      preview = null;
    } finally {
      saving = false;
    }
  }
  function close() {
    if (!saving && !refreshing) onClose();
  }
  function cancel(event: Event) {
    event.preventDefault();
    close();
  }
  function categoryName(type: ReferenceTypeId) {
    return REFERENCE_TYPE_OPTIONS.find((t) => t.id === type)?.label ?? type;
  }
  onMount(() => {
    const previous = document.activeElement;
    dialog.showModal();
    void loadProjects();
    return () => {
      clearTimeout(previewTimer);
      alive = false;
      generation++;
      dialog.close();
      if (previous instanceof HTMLElement && previous.isConnected) previous.focus();
    };
  });
</script>

<dialog
  bind:this={dialog}
  oncancel={cancel}
  aria-labelledby="copy-references-title"
  class="app-dialog-surface ka-dialog-default copy-dialog"
>
  <DialogHeader
    title="Copy references from another project"
    titleId="copy-references-title"
    onClose={close}
    disabled={saving || refreshing}
  />
  <div class="ka-dialog-body copy-body">
    <dl class="ka-facts copy-facts">
      <div>
        <dt>Destination</dt>
        <dd>{destination.name}</dd>
      </div>
    </dl>
    <p class="ka-help">
      These are independent copies. Changes won't update other projects. Scene links are not copied.
    </p>
    {#if result}
      <div role="status" class="ka-notice ka-notice--success od-row-top">
        <CircleCheck class="w-5 h-5" aria-hidden="true" />
        <div class="od-field od-fill">
          <strong>References copied</strong>
          <p>
            Copied {countLabel(result.copied, "reference")} from {source?.name} to {destination.name}.
            Skipped {result.skipped}
            possible duplicates.
          </p>
        </div>
      </div>
      {#if refreshing}<p role="status" class="ka-help copy-inline-status">
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />Refreshing references…
        </p>{/if}
    {:else}
      <div class="ka-field od-field">
        <label for="copy-source">Source project</label>
        <select
          id="copy-source"
          bind:value={sourceId}
          onchange={changeSource}
          disabled={loadingProjects || saving}
        >
          <option value="">Choose a project</option>
          {#each projects as project (project.id)}
            <option value={project.id}
              >{project.name} · {project.project_type} · {project.created_at.slice(0, 10)} · {project.id.slice(
                0,
                8
              )}</option
            >
          {/each}
        </select>
      </div>
      {#if loadingProjects}<p role="status" class="ka-help copy-inline-status">
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />Loading projects…
        </p>
      {:else if projects.length === 0 && !error}
        <div class="ka-empty od-stack copy-empty">
          <h4>No other projects</h4>
          <p>Create another project first, then copy its references here.</p>
        </div>
      {/if}
      {#if preview}
        <div class="ka-field od-field">
          <label for="copy-search">Search references</label>
          <input
            id="copy-search"
            type="search"
            bind:value={search}
            disabled={saving}
            placeholder="Search by name"
          />
        </div>
        <div class="copy-selection">
          <span class="copy-selection-count">{selectedCount} selected across all categories</span>
          <div class="copy-selection-actions">
            <button
              type="button"
              disabled={saving}
              onclick={() => selectRows(preview?.references ?? [], true)}
              class="ka-button ka-button--ghost">Select all references</button
            >
            <button
              type="button"
              disabled={saving}
              onclick={() => selectRows(preview?.references ?? [], false)}
              class="ka-button ka-button--ghost">Clear selection</button
            >
          </div>
        </div>
        {#if preview.references.length === 0}<p class="ka-help">
            This project has no references to copy.
          </p>{/if}
        <div class="copy-categories">
          {#each REFERENCE_TYPE_OPTIONS as category (category.id)}
            {@const rows = preview.references.filter(
              (r) => r.reference_type === category.id && matches(r.name)
            )}
            {#if rows.length}
              <fieldset disabled={saving} class="copy-category">
                <legend class="ka-group-title">{category.label}</legend>
                <label class="ka-check copy-all">
                  <input
                    type="checkbox"
                    checked={rows.every((r) => selection?.some((s) => same(s, r)))}
                    indeterminate={rows.some((r) => selection?.some((s) => same(s, r))) &&
                      !rows.every((r) => selection?.some((s) => same(s, r)))}
                    onchange={(e) => selectRows(rows, e.currentTarget.checked)}
                  />
                  Select {search.trim() ? "visible" : "all"}
                  {category.label.toLowerCase()}
                </label>
                <ul class="copy-rows">
                  {#each rows as row (row.id)}
                    <li class="copy-row">
                      <label class="ka-check copy-check">
                        <input
                          type="checkbox"
                          checked={selection?.some((s) => same(s, row))}
                          onchange={(e) => selectRows([row], e.currentTarget.checked)}
                        />
                        {row.name}
                      </label>
                      {#if row.description}<p class="copy-meta copy-description">
                          {row.description}
                        </p>{/if}
                      {#if selection?.some((s) => same(s, row)) && row.conflict}
                        <label class="copy-duplicate"
                          >Possible duplicate: {row.name}
                          <select
                            aria-label={`Duplicate choice for ${row.name}`}
                            value={keepBoth.some((k) => same(k, row)) ? "keep" : "skip"}
                            onchange={(e) => chooseDuplicate(row, e.currentTarget.value === "keep")}
                          >
                            <option value="skip">Skip</option><option value="keep">Keep both</option
                            >
                          </select>
                        </label>
                      {/if}
                      {#if row.action === "copy" && row.destination_name !== row.name}<p
                          class="copy-meta"
                        >
                          Copy as: {row.destination_name}
                        </p>{/if}
                    </li>
                  {/each}
                </ul>
              </fieldset>
            {/if}
          {/each}
          {#if preview.references.length > 0 && !preview.references.some((r) => matches(r.name))}<p
              class="ka-help"
            >
              No references match your search. Selections are unchanged.
            </p>{/if}
        </div>
        {#if preview.changes.length}
          <details aria-busy={loading} class="ka-disclosure copy-changes">
            <summary
              >Add {preview.changes.filter((c) => c.kind === "field").length} fields and {preview.changes.filter(
                (c) => c.kind === "tag"
              ).length} tags</summary
            >
            <div>
              <p class="ka-help">
                New fields are also available on existing references in their category. Existing
                values stay unchanged.
              </p>
              <ul class="copy-change-list">
                {#each preview.changes as change}
                  <li>
                    {change.kind === "field" ? `${change.entity_type} field` : "Tag"}: {change.source_name}{change.source_name !==
                    change.destination_name
                      ? ` → ${change.destination_name}`
                      : ""}
                  </li>
                {/each}
              </ul>
            </div>
          </details>
        {/if}
        {#if preview.enabled_types.length}<p class="ka-help">
            Enable categories: {preview.enabled_types.map(categoryName).join(", ")}
          </p>{/if}
        {#if preview.skipped}<p class="ka-help">
            Matches use category and name, not content. Renamed references may be copied again.
          </p>{/if}
      {/if}
      {#if sourceId}
        <p role="status" class="copy-status">
          {#if loading}
            Updating preview…
          {:else if preview}
            {preview.copied} to copy · {preview.skipped} possible duplicates skipped
          {/if}
        </p>
      {/if}
    {/if}
    {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
  </div>
  <footer class="ka-dialog-footer">
    {#if blockedReason}<p
        id="copy-blocked-reason"
        class="ka-help ka-dialog-footer-start copy-reason"
      >
        {blockedReason}
      </p>{/if}
    {#if refreshFailed}<button
        type="button"
        onclick={refreshCopied}
        disabled={refreshing}
        aria-busy={refreshing || undefined}
        class="ka-button">Refresh references</button
      >{/if}
    {#if error && !result && !loading}
      <button
        type="button"
        onclick={() => (sourceId ? loadPreview() : loadProjects())}
        disabled={saving}
        class="ka-button ka-button--secondary"
        >{sourceId ? "Refresh preview" : "Retry loading projects"}</button
      >
    {/if}
    <button
      type="button"
      onclick={close}
      disabled={saving || refreshing}
      class={result && !refreshFailed ? "ka-button" : "ka-button ka-button--secondary"}
      >{result ? "Done" : "Cancel"}</button
    >
    {#if !result}<button
        type="button"
        onclick={submit}
        disabled={!preview || preview.copied === 0 || loading || saving}
        aria-busy={saving || undefined}
        aria-describedby={blockedReason ? "copy-blocked-reason" : undefined}
        class="ka-button"
      >
        {#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />{/if}
        {saving ? "Copying…" : `Copy ${countLabel(preview?.copied ?? 0, "reference")}`}
      </button>{/if}
  </footer>
</dialog>

<style>
  .copy-dialog {
    max-height: calc(100dvh - 48px);
    margin: auto;
    padding: 0;
    overflow: hidden;
  }
  .copy-dialog[open] {
    display: flex;
    flex-direction: column;
  }
  .copy-dialog::backdrop {
    background: var(--color-overlay-scrim);
  }
  .copy-body {
    display: grid;
    align-content: start;
    gap: var(--space-s);
  }
  .copy-body > p {
    margin: 0;
  }
  /* The header's rule already closes the top; only rule it off below. */
  .copy-facts {
    border-bottom: var(--border-hair);
  }
  .copy-facts > :global(:first-child) {
    border-top: 0;
  }
  .copy-inline-status {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .copy-empty {
    padding-block: var(--space-s);
  }
  .copy-empty h4 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
  }
  .copy-empty p {
    margin: 0;
  }
  .copy-selection {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--space-2xs) var(--space-s);
    font: var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .copy-selection-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3xs);
  }
  .copy-categories {
    display: grid;
    gap: var(--space-m);
  }
  .copy-category {
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .copy-category legend {
    margin-bottom: var(--space-2xs);
    padding: 0;
  }
  .copy-rows {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: var(--border-hair);
  }
  .copy-all {
    color: var(--color-text-muted);
  }
  .copy-row {
    display: grid;
    gap: var(--space-3xs);
    padding-block: var(--space-3xs) var(--space-2xs);
    border-bottom: var(--border-hair);
  }
  .copy-check {
    color: var(--color-text);
    cursor: pointer;
    overflow-wrap: anywhere;
  }
  .copy-meta {
    margin: 0;
    padding-left: calc(20px + var(--space-xs));
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .copy-description {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .copy-duplicate {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2xs) var(--space-s);
    padding-left: calc(20px + var(--space-xs));
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .copy-duplicate select {
    width: auto;
    min-width: 10rem;
  }
  .copy-changes > div > p {
    margin: 0 0 var(--space-2xs);
  }
  .copy-change-list {
    display: grid;
    gap: var(--space-3xs);
    margin: 0;
    padding-left: var(--space-s);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .copy-status {
    font: 500 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .copy-reason {
    margin-block: 0;
  }
</style>
