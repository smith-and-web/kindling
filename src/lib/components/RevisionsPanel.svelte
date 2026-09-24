<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { History, Info, Loader2, Lock, Plus, RotateCcw, TriangleAlert } from "lucide-svelte";
  import { proseText } from "../utils/proseSearch";
  import {
    activeDocuments,
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
  // Set when a save pruned a draft the writer was comparing.
  let prunedNotice = $state("");
  let tab = $state<"history" | "overview">("history");
  let name = $state("");
  let before = $state(0);
  let after = $state(-1);
  let restoreIndex = $state<number | null>(null);
  // Which write is in flight, so its button can say what it is doing.
  let action = $state<"draft" | "restore" | null>(null);
  const oldDraft = $derived(review?.data.drafts[before]);
  const newDraft = $derived(after < 0 ? review : review?.data.drafts[after]);
  const comparison = $derived(
    oldDraft && newDraft
      ? diffText(
          activeDocuments({ ...oldDraft, scene_id: sceneId })
            .map((d) => proseText(d.html).trimEnd())
            .join("\n"),
          activeDocuments({ ...newDraft, scene_id: sceneId })
            .map((d) => proseText(d.html).trimEnd())
            .join("\n")
        )
      : []
  );
  const hasChanges = $derived(comparison.some((part) => part.kind !== "same"));
  function dateLabel(value: string) {
    return new Date(value).toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }

  onMount(() => {
    dialog.showModal();
    void load();
  });
  async function load() {
    busy = true;
    error = "";
    try {
      review = await invoke<SceneReview>("get_scene_review", { sceneId });
      before = Math.max(0, review.data.drafts.length - 1);
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
    prunedNotice = "";
    try {
      const previous = review.data.drafts;
      review = await invoke<SceneReview>("save_scene_review", { expected: review, data, next });
      // Saving can prune old automatic drafts, so keep the comparison on the same drafts.
      // If one was pruned, move that side to its nearest survivor (older for the saved draft,
      // newer for the comparison) without collapsing both sides onto one draft, and say so.
      const drafts = review.data.drafts;
      const find = (draft: ReviewDraft) =>
        drafts.findIndex((d) => d.created_at === draft.created_at && d.name === draft.name);
      const keep = (index: number) =>
        index < 0 || !previous[index]
          ? index
          : find(previous[index]) >= 0
            ? find(previous[index])
            : null;
      const nearest = (index: number, newerFirst: boolean, avoid: number | null) => {
        const older = previous.slice(0, index).reverse();
        const newer = previous.slice(index + 1);
        return (newerFirst ? [...newer, ...older] : [...older, ...newer])
          .map(find)
          .find((i) => i >= 0 && i !== avoid);
      };
      let nextBefore = keep(before);
      let nextAfter = keep(after);
      const removed = [
        ...new Set(
          [nextBefore === null && before, nextAfter === null && after]
            .filter((index): index is number => index !== false)
            .map((index) => `“${previous[index].name}”`)
        ),
      ];
      nextBefore ??= nearest(before, false, nextAfter) ?? nearest(before, false, null) ?? 0;
      nextAfter ??= nearest(after, true, nextBefore) ?? -1;
      // Current prose is never a saved draft, so it keeps two different drafts apart.
      if (nextAfter === nextBefore && before !== after) nextAfter = -1;
      before = nextBefore;
      after = nextAfter;
      const label = (index: number) =>
        index < 0 ? "the current prose" : `“${drafts[index].name}”`;
      prunedNotice = removed.length
        ? `${removed.join(" and ")} ${removed.length > 1 ? "were" : "was"} removed because only the newest automatic drafts are kept.` +
          (drafts[before] ? ` Now comparing ${label(before)} with ${label(after)}.` : "")
        : "";
      if (next) {
        onApplied(review);
      }
      // Update the overview locally after the committed write, so a failed
      // auxiliary read cannot make a successful decision appear to have failed.
      overview = overview.map((row) =>
        row.scene_id === sceneId
          ? { ...row, status: data.status, drafts: review!.data.drafts.length }
          : row
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
    action = "draft";
    const saved = await save(data);
    action = null;
    if (saved) {
      before = review.data.drafts.length - 1;
      name = "";
    }
  }
  async function restore() {
    if (!review || restoreIndex === null) return;
    const next = review.data.drafts[restoreIndex];
    const data = window.structuredClone(review.data);
    data.drafts.push(draftOf(review, "Before restoring " + next.name, true));
    action = "restore";
    const saved = await save(data, next);
    action = null;
    if (saved) restoreIndex = null;
  }
  async function setStatus(status: string) {
    if (review) await save({ ...review.data, status });
  }
</script>

<dialog
  bind:this={dialog}
  class="app-dialog-surface revisions-dialog"
  aria-labelledby="revisions-title"
  oncancel={(e) => {
    e.preventDefault();
    if (!busy) onClose();
  }}
  onkeydown={(e) => e.stopPropagation()}
>
  <DialogHeader
    title="Draft history"
    titleId="revisions-title"
    subtitle={title}
    {onClose}
    disabled={busy}
  >
    {#if review}<fieldset disabled={busy || locked} class="status-field">
        <label class="status-label" for="revision-status">Revision status</label>
        <select
          id="revision-status"
          value={review.data.status}
          onchange={(e) => setStatus(e.currentTarget.value)}
          >{#each Object.entries(revisionStatuses) as [value, label]}<option {value}>{label}</option
            >{/each}</select
        >
      </fieldset>{/if}
  </DialogHeader>
  <nav aria-label="Revision views" class="ka-tablist history-tabs">
    <button type="button" aria-pressed={tab === "history"} onclick={() => (tab = "history")}
      >Draft history</button
    >
    <button type="button" aria-pressed={tab === "overview"} onclick={() => (tab = "overview")}
      >All scenes</button
    >
  </nav>
  {#if error}<p role="alert" class="ka-error history-error">{error}</p>{/if}
  {#if !review}
    <div class="ka-empty od-stack history-empty">
      {#if busy}
        <p class="history-loading">
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />Loading revisions…
        </p>
      {:else}
        <p>Could not load revisions.</p>
        <button type="button" class="ka-button ka-button--secondary" onclick={load}>Retry</button>
      {/if}
    </div>
  {:else}
    {#if locked}
      <div class="ka-notice ka-notice--warning od-row-top locked-notice" role="status">
        <Lock class="w-5 h-5" aria-hidden="true" />
        <p>This scene is locked. History is read-only.</p>
      </div>
    {/if}
    {#if prunedNotice}
      <div class="ka-notice od-row-top locked-notice" role="status">
        <Info class="w-5 h-5" aria-hidden="true" />
        <p>{prunedNotice}</p>
      </div>
    {/if}
    {#if tab === "history"}
      <div class="history-layout">
        <aside class="draft-list" aria-label="Saved drafts">
          <h3 class="draft-list-title">
            Saved drafts <span class="ka-badge">{review.data.drafts.length}</span>
          </h3>
          <fieldset disabled={busy || locked} class="save-draft">
            <div class="ka-field od-field">
              <label for="revision-draft-name">Draft name</label>
              <input id="revision-draft-name" bind:value={name} placeholder="Post-editor pass" />
            </div>
            <button
              type="button"
              class="ka-button ka-button--secondary"
              disabled={!name.trim()}
              aria-busy={action === "draft" || undefined}
              onclick={createDraft}
              >{#if action === "draft"}<Loader2
                  class="w-5 h-5 animate-spin"
                  aria-hidden="true"
                />{:else}<Plus class="w-5 h-5" aria-hidden="true" />{/if}Save named draft</button
            >
          </fieldset>
          <div class="draft-entries">
            {#each review.data.drafts.map((draft, index) => ({ draft, index })).reverse() as item}
              <button
                type="button"
                class="draft-entry"
                aria-pressed={before === item.index}
                onclick={() => {
                  before = item.index;
                  restoreIndex = null;
                }}
                ><span class="draft-number">Draft {item.index + 1}</span><strong
                  >{item.draft.name}</strong
                ><time datetime={item.draft.created_at}>{dateLabel(item.draft.created_at)}</time
                ></button
              >
            {/each}
          </div>
        </aside>
        <section class="draft-detail" aria-label="Draft comparison">
          {#if !review.data.drafts.length}
            <div class="ka-empty od-stack history-empty">
              <History class="w-7 h-7" aria-hidden="true" />
              <h3>Keep a version of this scene</h3>
              <p>No saved drafts yet. Save a named draft to keep this scene’s current prose.</p>
            </div>
          {:else if oldDraft && newDraft}
            <div class="comparison-toolbar">
              <div class="ka-field od-field compare-field">
                <label for="revision-compare">Compare with</label>
                <select id="revision-compare" bind:value={after}
                  ><option value={-1}>Current prose</option
                  >{#each review.data.drafts as d, i}<option value={i}
                      >Draft {i + 1} · {d.name}</option
                    >{/each}</select
                >
              </div>
              <button
                type="button"
                class="ka-button ka-button--secondary"
                disabled={busy || locked}
                aria-expanded={restoreIndex !== null}
                onclick={() => (restoreIndex = before)}
                ><RotateCcw class="w-5 h-5" aria-hidden="true" />Restore selected draft</button
              >
            </div>
            {#if restoreIndex !== null}
              <div
                class="ka-notice ka-notice--warning od-row-top restore-confirm"
                role="region"
                aria-label="Confirm draft restore"
              >
                <TriangleAlert class="w-5 h-5" aria-hidden="true" />
                <div class="od-field od-fill restore-confirm-body">
                  <strong>Restore “{review.data.drafts[restoreIndex].name}”?</strong>
                  <p>
                    Current prose will be preserved as another draft. Beat structure must still
                    match.
                  </p>
                  <div class="restore-actions">
                    <button
                      type="button"
                      class="ka-button ka-button--ghost"
                      disabled={busy}
                      onclick={() => (restoreIndex = null)}>Cancel restore</button
                    ><button
                      type="button"
                      class="ka-button ka-button--danger"
                      disabled={busy || locked}
                      aria-busy={action === "restore" || undefined}
                      onclick={restore}
                      >{#if action === "restore"}<Loader2
                          class="w-5 h-5 animate-spin"
                          aria-hidden="true"
                        />{/if}Restore and preserve current prose</button
                    >
                  </div>
                </div>
              </div>
            {/if}
            <p class="ka-help comparison-note">
              {hasChanges
                ? "Removed text is marked on the left; added text is marked on the right."
                : "No prose text changes between these versions."} Formatting is not compared.
            </p>
            <div class="comparison-pages">
              <section class="comparison-version" aria-label="Saved draft">
                <header>
                  <span class="version-kind"
                    >Saved draft · {oldDraft.mode === "page" ? "Page prose" : "Beat prose"}</span
                  >
                  <h3>{oldDraft.name}</h3>
                  <time datetime={oldDraft.created_at}>{dateLabel(oldDraft.created_at)}</time>
                </header>
                <div class="app-prose-sheet diff-sheet">
                  <div class="diff-prose">
                    {#each comparison as part}{#if part.kind === "delete"}<del>{part.text}</del
                        >{:else if part.kind !== "insert"}{part.text}{/if}{/each}
                  </div>
                </div>
              </section>
              <section class="comparison-version" aria-label="Comparison version">
                <header>
                  <span class="version-kind"
                    >{after < 0 ? "Working manuscript" : "Saved draft"} · {newDraft.mode === "page"
                      ? "Page prose"
                      : "Beat prose"}</span
                  >
                  <h3>{after < 0 ? "Current prose" : review.data.drafts[after].name}</h3>
                  {#if after >= 0}<time datetime={review.data.drafts[after].created_at}
                      >{dateLabel(review.data.drafts[after].created_at)}</time
                    >{/if}
                </header>
                <div class="app-prose-sheet diff-sheet">
                  <div class="diff-prose">
                    {#each comparison as part}{#if part.kind === "insert"}<ins>{part.text}</ins
                        >{:else if part.kind !== "delete"}{part.text}{/if}{/each}
                  </div>
                </div>
              </section>
            </div>
          {/if}
        </section>
      </div>
    {:else}
      <div class="overview-scroll">
        <table>
          <thead
            ><tr><th>Chapter</th><th>Scene</th><th>Revision status</th><th>Saved drafts</th></tr
            ></thead
          ><tbody
            >{#each overview as row}<tr
                ><td>{row.chapter}</td><td>{row.title}</td><td
                  >{revisionStatuses[row.status as keyof typeof revisionStatuses]}</td
                ><td>{row.drafts}</td></tr
              >{/each}</tbody
          >
        </table>
      </div>
    {/if}
  {/if}
  {#if busy}<p role="status" class="ka-help busy-notice">Saving or loading…</p>{/if}
</dialog>

<style>
  .revisions-dialog {
    width: min(80rem, calc(100vw - var(--space-xl)));
    height: min(56rem, calc(100vh - var(--space-xl)));
    max-height: calc(100vh - var(--space-xl));
    margin: auto;
    padding: 0;
    font: var(--text-ui) / 1.5 var(--font-ui);
    overflow: hidden;
  }
  .revisions-dialog[open] {
    display: flex;
    flex-direction: column;
  }
  .revisions-dialog::backdrop {
    background: var(--color-overlay-scrim);
  }

  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
    min-width: 0;
  }
  .status-field {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .status-label {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
    white-space: nowrap;
  }
  .status-field select {
    width: auto;
    min-width: 11rem;
  }

  .history-tabs {
    flex: none;
    padding-inline: var(--space-m);
  }
  .history-tabs > button {
    margin-bottom: -1px;
  }
  .history-tabs > button[aria-pressed="true"] {
    border-bottom-color: var(--color-accent-text);
    color: var(--color-accent-text);
  }
  .history-error {
    margin: 0;
    padding: var(--space-xs) var(--space-m) 0;
  }
  .locked-notice {
    flex: none;
    margin: var(--space-s) var(--space-m) 0;
  }
  .locked-notice p {
    margin: 0;
  }

  .history-layout {
    display: grid;
    grid-template-columns: 18rem minmax(0, 1fr);
    min-height: 0;
    flex: 1;
  }
  .draft-list {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: var(--border-hair);
    background: var(--color-bg);
  }
  .draft-list-title {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    margin: var(--space-s) var(--space-s) var(--space-xs);
    font: 600 var(--text-ui) / 1.5 var(--font-ui);
    color: var(--color-text);
  }
  .save-draft {
    display: grid;
    gap: var(--space-2xs);
    margin: 0 var(--space-s) var(--space-s);
  }
  .draft-entries {
    overflow: auto;
    min-height: 0;
    border-top: var(--border-hair);
  }
  .draft-entry {
    display: flex;
    flex-direction: column;
    align-items: start;
    gap: var(--space-3xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-xs) var(--space-s);
    border: 0;
    border-bottom: var(--border-hair);
    border-left: 3px solid transparent;
    border-radius: 0;
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.4 var(--font-ui);
    text-align: left;
    overflow-wrap: anywhere;
    cursor: pointer;
  }
  .draft-entry:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: -3px;
  }
  @media (hover: hover) {
    .draft-entry:not([aria-pressed="true"]):hover {
      background: var(--color-surface-sunken);
    }
  }
  .draft-entry[aria-pressed="true"] {
    background: var(--color-accent-wash);
    border-left-color: var(--color-accent-text);
  }
  .draft-entry strong {
    font-weight: 500;
  }
  .draft-number,
  .version-kind,
  time {
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }

  .draft-detail {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
  }
  .comparison-toolbar {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--space-s);
    padding: var(--space-s) var(--space-m);
  }
  .compare-field {
    flex: 1 1 16rem;
    max-width: var(--measure);
  }
  .restore-confirm {
    margin: 0 var(--space-m) var(--space-s);
  }
  .restore-confirm-body {
    gap: var(--space-xs);
  }
  .restore-confirm-body p {
    margin: 0;
  }
  .restore-actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: var(--space-2xs);
  }
  .comparison-note {
    max-width: none;
    margin: 0;
    padding: 0 var(--space-m) var(--space-s);
    border-bottom: var(--border-hair);
  }
  .comparison-pages {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    min-height: 0;
    overflow: auto;
    flex: 1;
  }
  .comparison-version {
    padding: var(--space-m);
    min-width: 0;
  }
  .comparison-version + .comparison-version {
    border-left: var(--border-hair);
  }
  .comparison-version header {
    display: grid;
    gap: var(--space-3xs);
    border-bottom: var(--border-hair);
    padding-bottom: var(--space-s);
    margin-bottom: var(--space-m);
  }
  .comparison-version h3 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
    overflow-wrap: anywhere;
  }
  /* Prose is compared on manuscript paper, which stays light in both themes. */
  .diff-sheet {
    padding: var(--space-m) var(--space-l);
  }
  .diff-prose {
    font: var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-prose-text);
    max-width: var(--measure);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  /* Changed words keep manuscript ink: the chrome's status text colours are
     tuned for the chrome and fall below 4.5:1 on paper in the dark theme
     (success measured 4.33:1). The wash and the strike or underline carry the
     change, as the success badge and notice adapters in app.css do. */
  del,
  ins {
    color: var(--color-prose-text);
  }
  del {
    background: var(--color-error-wash);
    text-decoration-line: line-through;
  }
  ins {
    background: var(--color-success-wash);
    text-decoration-line: underline;
  }

  .history-empty {
    margin: auto;
    padding: var(--space-l);
    max-width: var(--measure);
  }
  .history-empty h3 {
    margin: 0;
    font: 550 var(--text-h3) / 1.25 var(--font-display);
    letter-spacing: var(--tracking-tight);
    color: var(--color-text);
  }
  .history-empty p {
    margin: 0;
  }
  .history-loading {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
  }
  .busy-notice {
    flex: none;
    max-width: none;
    margin: 0;
    padding: var(--space-2xs) var(--space-m);
    border-top: var(--border-hair);
  }

  .overview-scroll {
    overflow: auto;
    min-height: 0;
    flex: 1;
  }
  table {
    width: 100%;
    text-align: left;
    border-collapse: collapse;
    font: var(--text-ui) / 1.5 var(--font-ui);
  }
  td,
  th {
    height: var(--control-target);
    padding: var(--space-xs) var(--space-m);
    border-bottom: var(--border-hair);
  }
  th {
    font: 600 var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text-muted);
  }
</style>
