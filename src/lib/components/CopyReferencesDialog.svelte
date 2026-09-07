<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
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
  class="bg-press-surface text-press-text border border-press-border rounded-lg shadow-press-overlay p-0 w-full max-w-2xl max-h-[90vh] overflow-hidden m-auto backdrop:bg-press-overlay"
>
  <div class="flex flex-col max-h-[90vh]">
    <div class="p-6 space-y-4 overflow-y-auto min-h-0">
      <h2 id="copy-references-title" class="font-heading text-press-h3">
        Copy references from project…
      </h2>
      <p class="text-press-ui">Destination: <strong>{destination.name}</strong></p>
      <p class="text-press-small text-press-muted">
        These are independent copies. Changes won't update other projects. Scene links are not
        copied.
      </p>
      {#if result}
        <p role="status" class="text-press-ui">
          Copied {result.copied} references from {source?.name} to {destination.name}. Skipped {result.skipped}
          possible duplicates.
        </p>
        {#if refreshing}<p role="status">Refreshing references…</p>{/if}
      {:else}
        <label class="block text-press-ui" for="copy-source">Source project</label>
        <select
          id="copy-source"
          bind:value={sourceId}
          onchange={changeSource}
          disabled={loadingProjects || saving}
          class="w-full bg-press-bg border border-press-border rounded px-3 py-2 text-press-base"
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
        {#if loadingProjects}<p role="status">Loading projects…</p>
        {:else if projects.length === 0 && !error}<p>
            Create another project first, then copy its references here.
          </p>{/if}
        {#if preview}
          <label for="copy-search" class="block text-press-ui">Search references</label>
          <input
            id="copy-search"
            type="search"
            bind:value={search}
            disabled={saving}
            placeholder="Search by name"
            class="w-full bg-press-bg border border-press-border rounded px-3 py-2 text-press-base"
          />
          <div class="flex flex-wrap gap-3 text-press-ui">
            <span>{selectedCount} selected across all categories</span>
            <button
              disabled={saving}
              onclick={() => selectRows(preview?.references ?? [], true)}
              class="underline">Select all references</button
            >
            <button
              disabled={saving}
              onclick={() => selectRows(preview?.references ?? [], false)}
              class="underline">Clear selection</button
            >
          </div>
          {#if preview.references.length === 0}<p>This project has no references to copy.</p>{/if}
          <div class="space-y-4">
            {#each REFERENCE_TYPE_OPTIONS as category (category.id)}
              {@const rows = preview.references.filter(
                (r) => r.reference_type === category.id && matches(r.name)
              )}
              {#if rows.length}
                <fieldset disabled={saving} class="border-t border-press-border pt-2 min-w-0">
                  <legend class="text-press-ui font-medium">{category.label}</legend>
                  <label class="flex gap-2 items-center text-press-small mb-2">
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
                  {#each rows as row (row.id)}
                    <div class="py-2 border-t border-press-border space-y-1">
                      <label class="flex gap-2 items-start text-press-ui break-words">
                        <input
                          type="checkbox"
                          checked={selection?.some((s) => same(s, row))}
                          onchange={(e) => selectRows([row], e.currentTarget.checked)}
                        />
                        {row.name}
                      </label>
                      {#if row.description}<p
                          class="text-press-small text-press-muted line-clamp-2"
                        >
                          {row.description}
                        </p>{/if}
                      {#if selection?.some((s) => same(s, row)) && row.conflict}
                        <label class="block text-press-small"
                          >Possible duplicate: {row.name}
                          <select
                            aria-label={`Duplicate choice for ${row.name}`}
                            value={keepBoth.some((k) => same(k, row)) ? "keep" : "skip"}
                            onchange={(e) => chooseDuplicate(row, e.currentTarget.value === "keep")}
                            class="bg-press-bg border border-press-border rounded px-2 py-1 text-press-base ml-2"
                          >
                            <option value="skip">Skip</option><option value="keep">Keep both</option
                            >
                          </select>
                        </label>
                      {/if}
                      {#if row.action === "copy" && row.destination_name !== row.name}<p
                          class="text-press-small"
                        >
                          Copy as: {row.destination_name}
                        </p>{/if}
                    </div>
                  {/each}
                </fieldset>
              {/if}
            {/each}
            {#if preview.references.length > 0 && !preview.references.some( (r) => matches(r.name) )}<p
              >
                No references match your search. Selections are unchanged.
              </p>{/if}
          </div>
          {#if preview.changes.length}
            <details aria-busy={loading} class="border-t border-press-border pt-3">
              <summary class="text-press-ui cursor-pointer"
                >Add {preview.changes.filter((c) => c.kind === "field").length} fields and {preview.changes.filter(
                  (c) => c.kind === "tag"
                ).length} tags</summary
              >
              <p class="text-press-small text-press-muted my-2">
                New fields are also available on existing references in their category. Existing
                values stay unchanged.
              </p>
              <ul class="text-press-small space-y-1">
                {#each preview.changes as change}
                  <li>
                    {change.kind === "field" ? `${change.entity_type} field` : "Tag"}: {change.source_name}{change.source_name !==
                    change.destination_name
                      ? ` → ${change.destination_name}`
                      : ""}
                  </li>
                {/each}
              </ul>
            </details>
          {/if}
          {#if preview.enabled_types.length}<p class="text-press-small">
              Enable categories: {preview.enabled_types.map(categoryName).join(", ")}
            </p>{/if}
          {#if preview.skipped}<p class="text-press-small text-press-muted">
              Matches use category and name, not content. Renamed references may be copied again.
            </p>{/if}
        {/if}
        {#if sourceId}
          <p role="status" class="text-press-ui">
            {#if loading}
              Updating preview…
            {:else if preview}
              {preview.copied} to copy · {preview.skipped} possible duplicates skipped
            {/if}
          </p>
        {/if}
      {/if}
      {#if error}<p role="alert" class="text-press-error text-press-ui">{error}</p>{/if}
    </div>
    <div class="flex flex-wrap justify-end gap-3 border-t border-press-border px-6 py-4 shrink-0">
      {#if refreshFailed}<button
          onclick={refreshCopied}
          disabled={refreshing}
          class="px-4 py-2 border border-press-border rounded text-press-ui"
          >Refresh references</button
        >{/if}
      {#if error && !result && !loading}
        <button
          onclick={() => (sourceId ? loadPreview() : loadProjects())}
          disabled={saving}
          class="px-4 py-2 border border-press-border rounded text-press-ui"
          >{sourceId ? "Refresh preview" : "Retry loading projects"}</button
        >
      {/if}
      <button
        onclick={close}
        disabled={saving || refreshing}
        class="px-4 py-2 border border-press-border rounded text-press-ui"
        >{result ? "Done" : "Cancel"}</button
      >
      {#if !result}<button
          onclick={submit}
          disabled={!preview || preview.copied === 0 || loading || saving}
          class="px-4 py-2 bg-press-accent text-press-on-accent rounded text-press-ui disabled:cursor-not-allowed"
        >
          {saving ? "Copying…" : `Copy ${preview?.copied ?? 0} references`}
        </button>{/if}
    </div>
  </div>
</dialog>
