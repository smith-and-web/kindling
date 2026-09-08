<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ReviewProse from "./ReviewProse.svelte";
  import { proseText } from "../utils/proseSearch";
  import {
    acceptSuggestions,
    activeDocuments,
    diffText,
    draftOf,
    isAnchored,
    revisionStatuses,
    type SceneReview,
    type ReviewData,
    type ReviewDraft,
    type Annotation,
    type RevisionOverview,
  } from "../utils/revisions";
  let {
    sceneId,
    projectId,
    title,
    locked,
    onApplied,
    onClose,
  }: {
    sceneId: string;
    projectId: string;
    title: string;
    locked: boolean;
    onApplied: (review: SceneReview) => void;
    onClose: () => void;
  } = $props();
  let dialog: HTMLDialogElement;
  let review = $state.raw<SceneReview>();
  let overview = $state<RevisionOverview[]>([]);
  let busy = $state(false);
  let error = $state("");
  let tab = $state<"review" | "history" | "overview">("review");
  let documentId = $state("");
  let name = $state("");
  let author = $state("Writer");
  let comment = $state("");
  let replacement = $state("");
  let reply = $state("");
  let kind = $state<"comment" | "suggestion">("comment");
  let selection = $state<{ from: number; to: number; quote: string } | null>(null);
  let selectionIsExplicit = $state(false);
  let navigationVersion = $state(0);
  let selectedId = $state<string | null>(null);
  let before = $state(0);
  let after = $state(-1);
  let restoreIndex = $state<number | null>(null);
  const documents = $derived(review ? activeDocuments(review) : []);
  const source = $derived(documents.find((d) => d.id === documentId) ?? documents[0]);
  const visibleAnnotations = $derived(
    review?.data.annotations.filter((a) => documents.some((d) => d.id === a.document_id)) ?? []
  );
  const inactiveCount = $derived(
    (review?.data.annotations.length ?? 0) - visibleAnnotations.length
  );
  const selected = $derived(visibleAnnotations.find((a) => a.id === selectedId));
  const canReanchor = $derived(
    selectionIsExplicit &&
      !!selection &&
      !!selected &&
      (selected.from === selected.to
        ? selection.from === selection.to
        : selection.from < selection.to)
  );
  const pending = $derived(
    visibleAnnotations.filter((a) => a.state === "open" && a.replacement !== null)
  );
  const oldDraft = $derived(review?.data.drafts[before]);
  const newDraft = $derived(after < 0 ? review : review?.data.drafts[after]);

  onMount(() => {
    dialog.showModal();
    void load();
  });
  async function load() {
    busy = true;
    error = "";
    try {
      review = await invoke<SceneReview>("get_scene_review", { sceneId });
      overview = await invoke<RevisionOverview[]>("get_revision_overview", { projectId });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  async function save(data: ReviewData, next: ReviewDraft | null = null) {
    if (!review || busy || locked) return false;
    busy = true;
    error = "";
    try {
      review = await invoke<SceneReview>("save_scene_review", { expected: review, data, next });
      if (next) {
        selection = null;
        onApplied(review);
      }
      // Update the overview locally after the committed write, so a failed
      // auxiliary read cannot make a successful decision appear to have failed.
      overview = overview.map((row) =>
        row.scene_id === sceneId ? { ...row, status: data.status, drafts: data.drafts.length } : row
      );
      return true;
    } catch (e) {
      error = String(e);
      return false;
    } finally {
      busy = false;
    }
  }
  async function createDraft() {
    if (!review || !name.trim()) return;
    const data = window.structuredClone(review.data);
    data.drafts.push(draftOf(review, name));
    if (await save(data)) {
      before = data.drafts.length - 1;
      name = "";
    }
  }
  async function restore() {
    if (!review || restoreIndex === null) return;
    const next = review.data.drafts[restoreIndex];
    const data = window.structuredClone(review.data);
    data.drafts.push(draftOf(review, "Before restoring " + next.name));
    if (await save(data, next)) restoreIndex = null;
  }
  async function setStatus(status: string) {
    if (review) await save({ ...review.data, status });
  }
  async function addAnnotation() {
    if (!review || !source || !selection || !selectionIsExplicit || !author.trim()) return;
    if (kind === "comment" && (!comment.trim() || selection.from === selection.to)) return;
    if (kind === "suggestion" && !replacement && selection.from === selection.to) return;
    const a: Annotation = {
      id: window.crypto.randomUUID(),
      document_id: source.id,
      anchor_html: source.html,
      ...selection,
      replacement: kind === "suggestion" ? replacement : null,
      state: "open",
      messages: [
        { author: author.trim(), text: comment.trim(), created_at: new Date().toISOString() },
      ],
    };
    if (await save({ ...review.data, annotations: [...review.data.annotations, a] })) {
      selectedId = a.id;
      comment = "";
      replacement = "";
    }
  }
  async function changeAnnotations(ids: string[], state: Annotation["state"]) {
    if (!review) return;
    await save({
      ...review.data,
      annotations: review.data.annotations.map((a) => (ids.includes(a.id) ? { ...a, state } : a)),
    });
  }
  async function accept(ids: string[]) {
    if (!review) return;
    try {
      const { data, next } = acceptSuggestions(review, ids);
      await save(data, next);
    } catch (e) {
      error = String(e);
    }
  }
  async function addReply() {
    if (!review || !selected || !reply.trim() || !author.trim()) return;
    const data = window.structuredClone(review.data);
    data.annotations
      .find((a) => a.id === selectedId)!
      .messages.push({
        author: author.trim(),
        text: reply.trim(),
        created_at: new Date().toISOString(),
      });
    if (await save(data)) reply = "";
  }
  async function reanchor() {
    if (!review || !source || !selected || !selection || !canReanchor) return;
    const data = window.structuredClone(review.data);
    Object.assign(data.annotations.find((a) => a.id === selectedId)!, selection, {
      document_id: source.id,
      anchor_html: source.html,
    });
    await save(data);
  }
  function selectAnnotation(a: Annotation) {
    documentId = a.document_id;
    selectedId = a.id;
    navigationVersion++;
    selection = null;
    reply = "";
  }
  function step(direction: number) {
    const i = pending.findIndex((a) => a.id === selectedId);
    const a = pending[(i + direction + pending.length) % pending.length];
    if (a) selectAnnotation(a);
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="revisions-title"
  oncancel={(e) => {
    e.preventDefault();
    if (!busy) onClose();
  }}
  onkeydown={(e) => e.stopPropagation()}
>
  <header class="flex items-center justify-between gap-4 border-b border-press-border pb-3">
    <h2 id="revisions-title" class="font-heading text-press-h3">Revisions · {title}</h2>
    <button type="button" disabled={busy} onclick={onClose}>Close</button>
  </header>
  <nav aria-label="Revision views" class="flex gap-4 py-3">
    <button aria-pressed={tab === "review"} onclick={() => (tab = "review")}
      >Editorial review</button
    >
    <button aria-pressed={tab === "history"} onclick={() => (tab = "history")}>Draft history</button
    >
    <button aria-pressed={tab === "overview"} onclick={() => (tab = "overview")}>All scenes</button>
  </nav>
  {#if error}<p role="alert" class="text-press-error">{error}</p>{/if}
  {#if !review}
    <p>{busy ? "Loading revisions…" : "Could not load revisions."}</p>
    {#if !busy}<button onclick={load}>Retry</button>{/if}
  {:else}
    {#if locked}<p class="text-press-muted">
        This scene is locked. History and comments are read-only.
      </p>{/if}
    <fieldset disabled={busy || locked}>
      <label
        >Revision status
        <select value={review.data.status} onchange={(e) => setStatus(e.currentTarget.value)}>
          {#each Object.entries(revisionStatuses) as [value, label]}<option {value}>{label}</option
            >{/each}
        </select>
      </label>
    </fieldset>
    {#if tab === "history"}
      <fieldset disabled={busy || locked} class="flex gap-3 my-4">
        <label>Draft name <input bind:value={name} placeholder="Post-editor pass" /></label>
        <button disabled={!name.trim()} onclick={createDraft}>Save named draft</button>
      </fieldset>
      {#if review.data.drafts.length === 0}
        <p>No saved drafts yet. Save a named draft to keep this scene’s current prose.</p>
      {:else}
        <div class="flex flex-wrap gap-3 my-4">
          <label
            >Compare from <select bind:value={before}
              >{#each review.data.drafts as d, i}<option value={i}>Draft {i + 1} · {d.name}</option
                >{/each}</select
            ></label
          >
          <label
            >Compare to <select bind:value={after}
              ><option value={-1}>Current prose</option>{#each review.data.drafts as d, i}<option
                  value={i}>Draft {i + 1} · {d.name}</option
                >{/each}</select
            ></label
          >
          <button disabled={busy || locked} onclick={() => (restoreIndex = before)}
            >Restore selected draft</button
          >
        </div>
        {#if restoreIndex !== null}
          <div class="border-y border-press-border py-3">
            <p>
              Restore “{review.data.drafts[restoreIndex].name}”? Current prose will be preserved as
              another draft. Beat structure must still match.
            </p>
            <button disabled={busy || locked} onclick={restore}
              >Restore and preserve current prose</button
            >
            <button disabled={busy} onclick={() => (restoreIndex = null)}>Cancel restore</button>
          </div>
        {/if}
        {#if oldDraft && newDraft}
          <p class="text-press-muted my-3">
            Deleted text is struck through; inserted text is underlined. This comparison shows prose
            text, not formatting.
          </p>
          {#each [...new Set( [...oldDraft.documents.map((d) => d.id), ...newDraft.documents.map((d) => d.id)] )] as id}
            {@const oldDoc = oldDraft.documents.find((d) => d.id === id)}
            {@const newDoc = newDraft.documents.find((d) => d.id === id)}
            <h3 class="font-heading text-press-body-lg mt-4">{newDoc?.label ?? oldDoc?.label}</h3>
            <div class="diff-prose">
              {#each diffText(proseText(oldDoc?.html ?? ""), proseText(newDoc?.html ?? "")) as part}{#if part.kind === "delete"}<del
                    >{part.text}</del
                  >{:else if part.kind === "insert"}<ins>{part.text}</ins
                  >{:else}{part.text}{/if}{/each}
            </div>
          {/each}
        {/if}
      {/if}
    {:else if tab === "overview"}
      <table class="w-full my-4 text-left">
        <thead
          ><tr><th>Chapter</th><th>Scene</th><th>Revision status</th><th>Saved drafts</th></tr
          ></thead
        >
        <tbody
          >{#each overview as row}<tr
              ><td>{row.chapter}</td><td>{row.title}</td><td
                >{revisionStatuses[row.status as keyof typeof revisionStatuses]}</td
              ><td>{row.drafts}</td></tr
            >{/each}</tbody
        >
      </table>
    {:else if source}
      <p class="text-press-muted my-3">
        Select prose to comment or suggest a deletion/replacement. Place the cursor to suggest an
        insertion. Suggestions change the manuscript only when accepted.
      </p>
      <label
        >Prose source <select
          value={source.id}
          onchange={(e) => {
            documentId = e.currentTarget.value;
            selection = null;
            selectedId = null;
          }}
        >
          {#each documents as doc}<option value={doc.id}>{doc.label}</option>{/each}
        </select></label
      >
      <div class="review-columns mt-4">
        <div>
          {#key source.id}
            <ReviewProse
              {source}
              annotations={visibleAnnotations}
              {selectedId}
              {navigationVersion}
              onSelect={(from, to, quote, explicit) => {
                selection = { from, to, quote };
                selectionIsExplicit = explicit;
              }}
            />
          {/key}
          <fieldset disabled={busy || locked} class="mt-4 flex flex-col gap-3">
            <label>Your name <input bind:value={author} /></label>
            <label
              >Annotation <select bind:value={kind}
                ><option value="comment">Comment</option><option value="suggestion"
                  >Suggest change</option
                ></select
              ></label
            >
            <p class="text-press-muted">
              {selection
                ? selection.quote || "Insertion at cursor"
                : "Select text in the prose above."}
            </p>
            {#if kind === "suggestion"}<label
                >Suggested text (leave empty to delete)<textarea bind:value={replacement}
                ></textarea></label
              >{/if}
            <label
              >{kind === "comment" ? "Comment" : "Reason (optional)"}<textarea bind:value={comment}
              ></textarea></label
            >
            <button
              disabled={!selection ||
                !selectionIsExplicit ||
                !author.trim() ||
                (kind === "comment"
                  ? !comment.trim() || selection.from === selection.to
                  : !replacement && selection.from === selection.to)}
              onclick={addAnnotation}>Add {kind === "comment" ? "comment" : "suggestion"}</button
            >
          </fieldset>
        </div>
        <aside aria-label="Comments and suggestions">
          <div class="flex flex-wrap gap-3">
            <button disabled={!pending.length} onclick={() => step(-1)}>Previous change</button>
            <button disabled={!pending.length} onclick={() => step(1)}>Next change</button>
            <button
              disabled={busy || locked || !pending.length}
              onclick={() => accept(pending.map((a) => a.id))}>Accept all</button
            >
            <button
              disabled={busy || locked || !pending.length}
              onclick={() =>
                changeAnnotations(
                  pending.map((a) => a.id),
                  "rejected"
                )}>Reject all</button
            >
          </div>
          <p class="text-press-muted my-3">{pending.length} pending changes</p>
          {#if inactiveCount}<p class="text-press-muted">
              {inactiveCount}
              {inactiveCount === 1 ? "annotation belongs" : "annotations belong"} to inactive prose. Return
              to their original editing mode to review them.
            </p>{/if}
          {#each visibleAnnotations as a}
            <div class="border-t border-press-border py-3">
              <button
                class="text-left"
                aria-pressed={a.id === selectedId}
                onclick={() => selectAnnotation(a)}
                >{a.replacement === null ? "Comment" : "Suggestion"} · {a.state} · {a.quote ||
                  "Insertion"}</button
              >
              {#if a.id === selectedId}
                {#if a.replacement !== null}<p class="diff-prose">
                    <del>{a.quote}</del> <ins>{a.replacement}</ins>
                  </p>{/if}
                {#if !isAnchored(a, review.documents) && a.state === "open"}
                  <p class="text-press-warning">
                    Outdated anchor: prose changed. Select the intended text and re-anchor before
                    accepting.
                  </p>
                  <button disabled={busy || locked || !canReanchor} onclick={reanchor}
                    >Re-anchor to selection</button
                  >
                {/if}
                {#each a.messages as message}<p class="my-2">
                    <strong>{message.author}</strong>
                    <span class="text-press-muted"
                      >{new Date(message.created_at).toLocaleDateString()}</span
                    ><br />{message.text}
                  </p>{/each}
                <fieldset disabled={busy || locked} class="flex flex-col gap-2">
                  <label>Reply <textarea bind:value={reply}></textarea></label>
                  <button disabled={!reply.trim() || !author.trim()} onclick={addReply}
                    >Reply to thread</button
                  >
                  {#if a.state === "open"}
                    {#if a.replacement === null}<button
                        onclick={() => changeAnnotations([a.id], "resolved")}>Resolve thread</button
                      >
                    {:else}
                      <button
                        disabled={!isAnchored(a, review.documents)}
                        onclick={() => accept([a.id])}>Accept change</button
                      >
                      <button onclick={() => changeAnnotations([a.id], "rejected")}
                        >Reject change</button
                      >
                    {/if}
                  {:else if a.state === "resolved"}<button
                      onclick={() => changeAnnotations([a.id], "open")}>Reopen thread</button
                    >{/if}
                </fieldset>
              {/if}
            </div>
          {/each}
          {#if !visibleAnnotations.length}<p>No comments or suggestions yet.</p>{/if}
        </aside>
      </div>
    {/if}
  {/if}
  {#if busy}<p role="status">Saving or loading…</p>{/if}
</dialog>

<style>
  dialog {
    width: min(72rem, calc(100vw - var(--space-xl)));
    max-height: calc(100vh - var(--space-xl));
    margin: auto;
    padding: var(--space-l);
    background: var(--color-surface);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-m);
    box-shadow: var(--shadow-overlay);
    font-family: var(--font-ui);
    font-size: var(--text-ui);
    overflow: auto;
  }
  dialog::backdrop {
    background: var(--color-overlay-scrim);
  }
  .review-columns {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: var(--space-l);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }
  button {
    padding: var(--space-xs) var(--space-s);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
  }
  button[aria-pressed="true"] {
    background: var(--color-accent-wash);
  }
  button:disabled {
    color: var(--color-disabled-text);
    background: var(--color-disabled-bg);
  }
  input,
  select,
  textarea {
    font-size: var(--text-base);
    max-width: 100%;
  }
  .diff-prose {
    font-family: var(--font-body);
    font-size: var(--text-body);
    max-width: var(--measure);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  del {
    color: var(--color-error);
  }
  ins {
    color: var(--color-success);
  }
  td,
  th {
    padding: var(--space-s);
    border-bottom: 1px solid var(--color-border);
  }
  @media (max-width: 800px) {
    .review-columns {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
