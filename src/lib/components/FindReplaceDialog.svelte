<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
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
  class="app-dialog-surface ka-dialog-default find-dialog"
  aria-labelledby="find-title"
  oncancel={(event) => {
    event.preventDefault();
    if (!busy) onClose();
  }}
  onkeydown={(event) => {
    event.stopPropagation();
  }}
>
  <DialogHeader title="Find and Replace" titleId="find-title" {onClose} disabled={busy} />
  <div class="ka-dialog-body find-body">
    <fieldset disabled={busy || loadFailed} class="find-form">
      <div class="ka-field od-field">
        <label for="find-query">Find</label>
        <div class="find-query">
          <input
            id="find-query"
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
            aria-describedby="find-status"
          />
          <button
            type="button"
            class="ka-button ka-button--secondary"
            disabled={!results.length}
            onclick={() => navigate(-1)}
            title="Previous match (Shift+Enter)">Previous</button
          >
          <button
            type="button"
            class="ka-button ka-button--secondary"
            disabled={!results.length}
            onclick={() => navigate(1)}
            title="Next match (Enter)">Next</button
          >
        </div>
        <p id="find-status" role="status" class="ka-help">
          {busy
            ? "Loading prose…"
            : query
              ? `${results.length} ${results.length === 1 ? "match" : "matches"}`
              : "Enter text to find."}
        </p>
      </div>
      <div class="find-options">
        <div class="ka-field od-field find-scope">
          <label for="find-scope">Search in</label>
          <select id="find-scope" bind:value={scope} onchange={reset}>
            <option value="scene" disabled={!sceneId}>Current scene</option>
            <option value="project">Entire project</option>
          </select>
        </div>
        <div class="ka-checks">
          <label class="ka-check"
            ><input type="checkbox" bind:checked={caseSensitive} onchange={reset} /> Match case</label
          >
          <label class="ka-check"
            ><input type="checkbox" bind:checked={wholeWord} onchange={reset} /> Whole words</label
          >
          <label class="ka-check"><input type="checkbox" bind:checked={replacing} /> Replace</label>
        </div>
      </div>
      {#if replacing}
        <div class="ka-field od-field">
          <label for="find-replacement">Replace with</label>
          <input
            id="find-replacement"
            type="text"
            bind:value={replacement}
            oninput={() => (confirming = false)}
          />
        </div>
      {/if}
      <p class="ka-help">
        Searches visible prose in Fixed scenes. Flexible, Undefined and archived scenes are
        excluded; locked scenes are searchable but cannot be replaced.
      </p>
      {#if active}
        <div class="ka-results find-result">
          <p class="find-location">
            {Math.min(selected + 1, results.length)} of {results.length} · {active.doc
              .chapter_title} /
            {active.doc.scene_title}{active.doc.beat_title !== null
              ? ` / ${active.doc.beat_title}`
              : ""}{active.doc.locked ? " · Locked" : active.readOnly ? " · Unsaved draft" : ""}
          </p>
          <p class="find-excerpt">
            {active.match.from > 100 ? "…" : ""}{active.text.slice(
              Math.max(0, active.match.from - 100),
              active.match.from
            )}<mark>{active.text.slice(active.match.from, active.match.to)}</mark
            >{active.text.slice(active.match.to, active.match.to + 160)}{active.text.length >
            active.match.to + 160
              ? "…"
              : ""}
          </p>
          {#if onOpenScene}<button
              type="button"
              class="ka-button ka-button--ghost find-open"
              onclick={openScene}>Open scene</button
            >{/if}
        </div>
      {:else if query && !busy && !error}
        <p class="ka-help">
          {results.length
            ? "No more matches ahead. Use Next or Previous to continue."
            : "No matches found."}
        </p>
      {/if}
      {#if replacing && confirming}
        <div class="ka-notice ka-notice--warning" role="status">
          <p>
            Replace {editableCount} matches in {scope === "project"
              ? "the entire project"
              : "the current scene"}? {results.length - editableCount} locked or unsaved matches will
            be skipped.
          </p>
          <div class="ka-row">
            <button type="button" class="ka-button" onclick={() => replace(true)}
              >Confirm replace all</button
            >
            <button
              type="button"
              class="ka-button ka-button--secondary"
              onclick={() => (confirming = false)}>Cancel</button
            >
          </div>
        </div>
      {/if}
    </fieldset>
    {#if message}<p role="status" class="ka-help">{message}</p>{/if}
    {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
    {#if loadFailed || pendingDrafts.length}
      <div class="ka-notice ka-notice--warning find-drafts">
        <button
          type="button"
          class="ka-button ka-button--secondary"
          disabled={busy}
          onclick={() => load(true)}>{loadFailed ? "Retry loading" : "Retry saving drafts"}</button
        >
        {#if pendingDrafts.length}
          <p>
            Unsaved drafts are retained for this session, including after closing the project.
            Locked or missing documents and unrecognized save errors are not retried automatically
            and do not block other scenes. You can retry after resolving the save error, or copy
            these drafts before discarding them. Matches in documents with unsaved drafts cannot be
            replaced.
          </p>
          {#each pendingDrafts as draft, index}
            <details class="ka-disclosure">
              <summary>Unsaved {draft.kind === "beat" ? "beat" : "scene"} {index + 1}</summary>
              <div class="ka-field od-field">
                <label for={`find-draft-${index}`}>Draft text (select to copy)</label>
                <textarea
                  id={`find-draft-${index}`}
                  readonly
                  rows="5"
                  value={proseText(draft.prose)}
                ></textarea>
              </div>
            </details>
          {/each}
          {#if onDiscardDrafts}
            {#if confirmingDiscard}
              <p>
                Discard these unsaved drafts? Their changes will be lost. Copy any text you want to
                keep first.
              </p>
              <div class="ka-row">
                <button
                  type="button"
                  class="ka-button ka-button--danger"
                  disabled={busy}
                  onclick={discardDrafts}>Confirm discard drafts</button
                >
                <button
                  type="button"
                  class="ka-button ka-button--secondary"
                  disabled={busy}
                  onclick={() => (confirmingDiscard = false)}>Keep drafts</button
                >
              </div>
            {:else}
              <button
                type="button"
                class="ka-button ka-button--ghost"
                disabled={busy}
                onclick={() => (confirmingDiscard = true)}>Discard unsaved drafts…</button
              >
            {/if}
          {/if}
        {/if}
      </div>
    {/if}
  </div>
  {#if replacing}
    <footer class="ka-dialog-footer">
      <button
        type="button"
        class="ka-button ka-button--ghost ka-dialog-footer-start"
        disabled={busy || !undo.length}
        onclick={() => apply(undo[undo.length - 1], true)}>Undo replacement</button
      >
      <button
        type="button"
        class="ka-button ka-button--secondary"
        disabled={busy || !editableCount || confirming}
        onclick={() => (confirming = true)}>Replace all</button
      >
      <button
        type="button"
        class="ka-button"
        disabled={busy || !active || active.readOnly || confirming}
        onclick={() => replace(false)}>Replace match</button
      >
    </footer>
  {/if}
</dialog>

<style>
  .find-dialog {
    max-height: calc(100dvh - 48px);
    margin: auto;
    padding: 0;
    overflow: hidden;
  }
  .find-dialog[open] {
    display: flex;
    flex-direction: column;
  }
  .find-dialog::backdrop {
    background: var(--color-overlay-scrim);
  }
  .find-body {
    display: grid;
    gap: var(--space-s);
  }
  .find-form {
    display: grid;
    gap: var(--space-s);
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
  .find-query {
    display: flex;
    gap: var(--space-2xs);
  }
  .find-query input {
    flex: 1;
    min-width: 0;
  }
  .find-options {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-2xs) var(--space-m);
  }
  .find-scope {
    width: auto;
    min-width: 200px;
  }
  .find-result {
    display: grid;
    gap: var(--space-2xs);
    padding-block: var(--space-s);
    border-top: var(--border-hair);
  }
  .find-location {
    margin: 0;
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
  .find-excerpt {
    margin: 0;
    max-width: var(--measure);
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .find-open {
    justify-self: start;
    margin-left: calc(-1 * var(--space-s));
  }
  .find-drafts {
    justify-items: start;
  }
</style>
