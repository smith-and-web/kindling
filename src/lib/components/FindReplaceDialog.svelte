<script lang="ts">
  import { onMount, tick } from "svelte";
  import { proseSaves, type ProseSave } from "../utils/proseSaves";
  import { invoke } from "@tauri-apps/api/core";
  import {
    proseText,
    searchText,
    replaceProse,
    type ProseDocument,
    type ProseReplacement,
  } from "../utils/proseSearch";

  let {
    projectId,
    sceneId,
    initialScope = "scene",
    showReplace = false,
    prepare,
    onApplied,
    onOpenScene,
    onDiscardDrafts,
    onClose,
  }: {
    projectId: string;
    sceneId: string | null;
    initialScope?: "scene" | "project";
    showReplace?: boolean;
    prepare: () => Promise<void>;
    onApplied: (changes: Pick<ProseReplacement, "id" | "prose">[]) => void;
    onOpenScene?: (doc: ProseDocument) => Promise<void>;
    onDiscardDrafts?: (drafts: ProseSave[]) => Promise<void>;
    onClose: () => void;
  } = $props();
  let scope = $state<"scene" | "project">("scene");
  let replacing = $state(false);
  let query = $state("");
  let replacement = $state("");
  let caseSensitive = $state(false);
  let wholeWord = $state(false);
  let documents = $state<{ doc: ProseDocument; text: string }[]>([]);
  let busy = $state(true);
  let error = $state("");
  let message = $state("");
  let selected = $state(0);
  let confirming = $state(false);
  let undo = $state<ProseReplacement[][]>([]);
  let dialog: HTMLDialogElement;
  let findInput: HTMLInputElement | undefined;
  let mounted = false;
  let loadFailed = $state(false);
  let pendingDrafts = $state.raw<ProseSave[]>([]);
  let confirmingDiscard = $state(false);
  const results = $derived(
    documents
      .filter(({ doc }) => scope === "project" || doc.scene_id === sceneId)
      .flatMap(({ doc, text }) => {
        const { matches } = searchText(text, query, { caseSensitive, wholeWord });
        const readOnly = doc.locked || pendingDrafts.some((draft) => draft.id === doc.id);
        return matches.map((match) => ({ doc, text, match, readOnly }));
      })
  );
  const active = $derived(results[selected]);
  const editableCount = $derived(results.filter((result) => !result.readOnly).length);

  export function configure(nextScope: "scene" | "project", showReplacement: boolean) {
    scope = sceneId ? nextScope : "project";
    replacing = showReplacement;
    reset();
    if (!busy) findInput?.focus();
  }

  onMount(() => {
    mounted = true;
    const previousFocus = document.activeElement as HTMLElement | null;
    scope = sceneId ? initialScope : "project";
    replacing = showReplace;
    dialog.showModal();
    void load();
    return () => {
      mounted = false;
      previousFocus?.focus();
    };
  });

  async function load(retryRecovered = false) {
    busy = true;
    error = "";
    message = "";
    loadFailed = false;
    confirmingDiscard = false;
    try {
      if (retryRecovered) {
        const saved = await proseSaves.retryRecovered(projectId);
        if (mounted && saved.length) onApplied(saved);
      }
      if (!mounted) return;
      await prepare();
      if (!mounted) return;
      const loaded = await invoke<ProseDocument[]>("get_search_documents", { projectId });
      if (!mounted) return;
      pendingDrafts = proseSaves.draftsForRecovery(projectId);
      selected = 0;
      documents = loaded.map((doc) => ({ doc, text: proseText(doc.prose) }));
    } catch (e) {
      if (!mounted) return;
      error = String(e);
      loadFailed = true;
      pendingDrafts = proseSaves.draftsForRecovery(projectId);
    } finally {
      if (mounted) {
        busy = false;
        await tick();
        if (mounted && findInput?.isConnected) findInput.focus();
      }
    }
  }

  async function openScene() {
    if (!active || !onOpenScene) return;
    busy = true;
    try {
      await onOpenScene(active.doc);
      if (mounted) onClose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function reset() {
    selected = 0;
    confirming = false;
    message = "";
  }
  function navigate(direction: number) {
    if (!results.length) return;
    if (selected >= results.length) selected = direction > 0 ? 0 : results.length - 1;
    else selected = (selected + direction + results.length) % results.length;
  }

  function advancePast(after: { id: string; offset: number }) {
    const docIndex = documents.findIndex((entry) => entry.doc.id === after.id);
    const next = results.findIndex((result) => {
      const index = documents.findIndex((entry) => entry.doc.id === result.doc.id);
      return index > docIndex || (index === docIndex && result.match.from >= after.offset);
    });
    // Do not wrap automatically into freshly inserted text containing the query.
    selected = next < 0 ? results.length : next;
  }

  async function discardDrafts() {
    if (!onDiscardDrafts) return;
    busy = true;
    error = "";
    try {
      await onDiscardDrafts(pendingDrafts);
      if (mounted) await load();
    } catch (e) {
      if (mounted) error = String(e);
    } finally {
      if (mounted) busy = false;
    }
  }

  async function apply(
    changes: ProseReplacement[],
    undoing = false,
    after?: { id: string; offset: number }
  ) {
    busy = true;
    error = "";
    message = "";
    confirming = false;
    try {
      await invoke("replace_prose_batch", { projectId, changes });
      if (!mounted) return;
      documents = documents.map((entry) => {
        const change = changes.find((change) => change.id === entry.doc.id);
        return change
          ? { doc: { ...entry.doc, prose: change.prose }, text: proseText(change.prose) }
          : entry;
      });
      onApplied(changes);
      if (undoing) undo = undo.slice(0, -1);
      else
        undo = [
          ...undo,
          changes.map((change) => ({
            id: change.id,
            expected_prose: change.prose,
            prose: change.expected_prose,
          })),
        ];
      message = undoing ? "Replacement undone." : "Replacement saved.";
      if (after) advancePast(after);
      else selected = Math.min(selected, Math.max(0, results.length - 1));
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function replace(all: boolean) {
    const targets = all
      ? results.filter((result) => !result.readOnly)
      : active && !active.readOnly
        ? [active]
        : [];
    const ids = [...new Set(targets.map((result) => result.doc.id))];
    const changes = ids
      .map((id) => {
        const group = targets.filter((result) => result.doc.id === id);
        const doc = group[0].doc;
        return {
          id,
          expected_prose: doc.prose,
          prose: replaceProse(
            doc.prose,
            group.map((result) => result.match),
            replacement
          ),
        };
      })
      .filter((change) => change.expected_prose !== change.prose);
    const after =
      !all && active
        ? { id: active.doc.id, offset: active.match.from + replacement.length }
        : undefined;
    if (changes.length) void apply(changes, false, after);
    else {
      confirming = false;
      message = "No changes needed.";
      if (after) advancePast(after);
    }
  }
</script>

<dialog
  bind:this={dialog}
  class="app-dialog-surface find-dialog"
  aria-labelledby="find-title"
  oncancel={(event) => {
    event.preventDefault();
    if (!busy) onClose();
  }}
  onkeydown={(event) => {
    event.stopPropagation();
  }}
>
  <div class="flex items-center justify-between gap-4 mb-4">
    <h2 id="find-title" class="font-heading text-press-h2">Find and Replace</h2>
    <button type="button" onclick={onClose} disabled={busy} aria-label="Close Find and Replace"
      >Close</button
    >
  </div>
  <fieldset disabled={busy || loadFailed} class="flex flex-col gap-3">
    <label class="flex flex-col gap-1"
      >Find
      <input
        bind:this={findInput}
        bind:value={query}
        oninput={reset}
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            navigate(event.shiftKey ? -1 : 1);
          }
        }}
        type="text"
      />
    </label>
    <div class="flex flex-wrap items-center gap-4">
      <label
        >Search in
        <select bind:value={scope} onchange={reset}>
          <option value="scene" disabled={!sceneId}>Current scene</option>
          <option value="project">Entire project</option>
        </select>
      </label>
      <label
        ><input type="checkbox" bind:checked={caseSensitive} onchange={reset} /> Match case</label
      >
      <label><input type="checkbox" bind:checked={wholeWord} onchange={reset} /> Whole words</label>
      <label><input type="checkbox" bind:checked={replacing} /> Replace</label>
    </div>
    {#if replacing}
      <label class="flex flex-col gap-1"
        >Replace with
        <input type="text" bind:value={replacement} oninput={() => (confirming = false)} />
      </label>
    {/if}
    <p class="text-press-muted text-press-small">
      Searches visible prose in Fixed scenes. Flexible, Undefined and archived scenes are excluded;
      locked scenes are searchable but cannot be replaced.
    </p>
    <div class="flex items-center gap-3">
      <p role="status" class="flex-1">
        {busy
          ? "Loading prose…"
          : query
            ? `${results.length} ${results.length === 1 ? "match" : "matches"}`
            : "Enter text to find."}
      </p>
      <button type="button" disabled={!results.length} onclick={() => navigate(-1)}>Previous</button
      >
      <button type="button" disabled={!results.length} onclick={() => navigate(1)}>Next</button>
    </div>
    {#if active}
      <div class="result-preview">
        {#if onOpenScene}<button type="button" onclick={openScene} class="mb-3">Open scene</button
          >{/if}
        <p class="text-press-small text-press-muted mb-2">
          {Math.min(selected + 1, results.length)} of {results.length} · {active.doc.chapter_title} /
          {active.doc.scene_title}{active.doc.beat_title !== null
            ? ` / ${active.doc.beat_title}`
            : ""}{active.doc.locked ? " · Locked" : active.readOnly ? " · Unsaved draft" : ""}
        </p>
        <p class="font-prose text-press-body whitespace-pre-wrap break-words">
          {active.match.from > 100 ? "…" : ""}{active.text.slice(
            Math.max(0, active.match.from - 100),
            active.match.from
          )}<mark>{active.text.slice(active.match.from, active.match.to)}</mark>{active.text.slice(
            active.match.to,
            active.match.to + 160
          )}{active.text.length > active.match.to + 160 ? "…" : ""}
        </p>
      </div>
    {:else if query && !busy && !error}
      <p>
        {results.length
          ? "No more matches ahead. Use Next or Previous to continue."
          : "No matches found."}
      </p>
    {/if}
    {#if replacing}
      <div class="flex flex-wrap gap-3">
        <button type="button" disabled={!active || active.readOnly} onclick={() => replace(false)}
          >Replace match</button
        >
        <button type="button" disabled={!editableCount} onclick={() => (confirming = true)}
          >Replace all</button
        >
        <button
          type="button"
          disabled={!undo.length}
          onclick={() => apply(undo[undo.length - 1], true)}>Undo replacement</button
        >
      </div>
      {#if confirming}
        <div class="result-preview">
          <p>
            Replace {editableCount} matches in {scope === "project"
              ? "the entire project"
              : "the current scene"}? {results.length - editableCount} locked or unsaved matches will
            be skipped.
          </p>
          <div class="flex gap-3 mt-3">
            <button type="button" onclick={() => replace(true)}>Confirm replace all</button>
            <button type="button" onclick={() => (confirming = false)}>Cancel</button>
          </div>
        </div>
      {/if}
    {/if}
  </fieldset>
  {#if message}<p role="status" class="mt-3">{message}</p>{/if}
  {#if error}<p role="alert" class="text-press-error mt-3">{error}</p>{/if}
  {#if loadFailed || pendingDrafts.length}
    <div class="mt-3 flex flex-col gap-3">
      <button type="button" disabled={busy} onclick={() => load(true)}
        >{loadFailed ? "Retry loading" : "Retry saving drafts"}</button
      >
      {#if pendingDrafts.length}
        <p>
          Unsaved drafts are retained for this session, including after closing the project. Locked
          or missing documents and unrecognized save errors are not retried automatically and do not
          block other scenes. You can retry after resolving the save error, or copy these drafts
          before discarding them. Matches in documents with unsaved drafts cannot be replaced.
        </p>
        {#each pendingDrafts as draft, index}
          <details>
            <summary>Unsaved {draft.kind === "beat" ? "beat" : "scene"} {index + 1}</summary>
            <label class="flex flex-col gap-1 mt-3"
              >Draft text (select to copy)
              <textarea
                readonly
                rows="5"
                class="w-full bg-press-sunken text-press-text p-3"
                value={proseText(draft.prose)}
              ></textarea>
            </label>
          </details>
        {/each}
        {#if onDiscardDrafts}
          {#if confirmingDiscard}
            <p>
              Discard these unsaved drafts? Their changes will be lost. Copy any text you want to
              keep first.
            </p>
            <div class="flex gap-3">
              <button type="button" disabled={busy} onclick={discardDrafts}
                >Confirm discard drafts</button
              >
              <button type="button" disabled={busy} onclick={() => (confirmingDiscard = false)}
                >Keep drafts</button
              >
            </div>
          {:else}
            <button type="button" disabled={busy} onclick={() => (confirmingDiscard = true)}
              >Discard unsaved drafts…</button
            >
          {/if}
        {/if}
      {/if}
    </div>
  {/if}
</dialog>

<style>
  .find-dialog {
    width: min(46rem, calc(100vw - var(--space-l)));
    max-height: calc(100vh - var(--space-l));
    overflow: auto;
    padding: var(--space-l);
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-m);
    font-family: var(--font-ui);
    font-size: var(--text-ui);
    margin: auto;
  }
  .find-dialog::backdrop {
    background: var(--color-overlay-scrim);
  }
  input[type="text"],
  select {
    padding: var(--space-xs) var(--space-s);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
    background: var(--color-surface-sunken);
    color: var(--color-text);
    min-width: 0;
  }
  button {
    padding: var(--space-xs) var(--space-s);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
    cursor: pointer;
  }
  button:hover {
    background: var(--color-surface-sunken);
  }
  button:disabled {
    color: var(--color-text-muted);
    cursor: default;
  }
  .result-preview {
    border-block: 1px solid var(--color-border);
    padding-block: var(--space-m);
  }
  mark {
    background: var(--color-accent-wash);
    color: var(--color-text);
    outline: 1px solid var(--color-accent);
  }
</style>
