<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { proseText } from "../utils/proseSearch";
  import {
    diffText,
    draftOf,
    revisionStatuses,
    type SceneReview,
    type ReviewData,
    type ReviewDraft,
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
  let tab = $state<"history" | "overview">("history");
  let name = $state("");
  let before = $state(0);
  let after = $state(-1);
  let restoreIndex = $state<number | null>(null);
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
    <h2 id="revisions-title" class="font-heading text-press-h3">Draft history · {title}</h2>
    <button type="button" disabled={busy} onclick={onClose}>Close</button>
  </header>
  <nav aria-label="Revision views" class="flex gap-4 py-3">
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
  select {
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
</style>
