<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { Node } from "@tiptap/pm/model";
  import type { Mapping } from "@tiptap/pm/transform";
  import EditorialManuscript from "./EditorialManuscript.svelte";
  import { EditorialSaves } from "../utils/editorialSaves";
  import {
    manuscript,
    editorialSchema,
    trackChanges,
    changesBetween,
    changeMap,
    projectedRange,
    message,
    sliceText,
    sliceHtml,
    validateEditorialPackage,
    restoreTracking,
    locateChange,
    prepareAcceptance,
    withdrawSuggestion,
    type EditorialRound,
    type EditorialPackage,
    type EditorialSession,
    type EditorialSource,
    type EditorialFeedback,
    type EditorialChange,
    type FeedbackEntry,
  } from "../utils/editorial";

  let {
    prepareWriting,
    onManuscriptChanged,
  }: { prepareWriting: () => Promise<void>; onManuscriptChanged: () => Promise<void> } = $props();
  let dialog: HTMLDialogElement;
  let prose = $state<ReturnType<typeof EditorialManuscript>>();
  let active = $state(false);
  let screen = $state<"export" | "review" | "preview" | "feedback">("export");
  let busy = $state(false),
    error = $state(""),
    notice = $state("");
  let round = $state<EditorialRound | null>(null);
  let session = $state<EditorialSession | null>(null);
  let received = $state<EditorialPackage | null>(null);
  let feedback = $state<EditorialFeedback | null>(null);
  let exportSources = $state<EditorialSource[]>([]),
    rounds = $state<EditorialRound[]>([]);
  let projectId = $state(""),
    roundName = $state("Editorial pass"),
    brief = $state("");
  let selectedChapters = $state<string[]>([]);
  let selection = $state({ from: 1, to: 1 });
  let reanchorReady = $state(false);
  let selectedId = $state<string | null>(null);
  let commentText = $state(""),
    replyText = $state(""),
    writerName = $state("Writer");
  let showComment = $state(false),
    markup = $state(true),
    filter = $state("open");
  let replyReviewer = $state("");
  const reviewers = $derived(
    feedback?.entries
      .filter(
        (entry, i, all) =>
          all.findIndex((e) => e.key.split("/")[0] === entry.key.split("/")[0]) === i
      )
      .map((entry) => ({ id: entry.key.split("/")[0], name: entry.reviewer })) ?? []
  );
  let search = $state(""),
    searchIndex = $state(0),
    searchCount = $state(0);
  let manuscriptVersion = $state(0);
  let savedState = $state("Saved locally");
  let saves: EditorialSaves | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const incoming: string[] = [];
  let draining = false;
  let drainAgain = false;

  const sources = $derived(
    screen === "feedback" ? (feedback?.sources ?? []) : (round?.sources ?? [])
  );
  const baseline = $derived(round ? manuscript(round.sources) : null);
  const navigation = $derived(
    sources.filter((s, i) => !sources.slice(0, i).some((p) => p.scene_id === s.scene_id))
  );
  const visibleChanges = $derived(
    session?.changes.filter((c) => filter === "all" || (c.writer_decision ?? c.state) === filter) ??
      []
  );
  const visibleEntries = $derived(
    feedback?.entries.filter((e) => filter === "all" || e.decision === filter) ?? []
  );
  const selectedChange = $derived(session?.changes.find((c) => c.id === selectedId));
  const selectedEntry = $derived(feedback?.entries.find((e) => e.key === selectedId));
  const manuscriptAnnotations = $derived.by(() => {
    if (session) return session.changes;
    if (!feedback) return [];
    const base = manuscript(feedback.round.sources),
      current = manuscript(feedback.sources);
    return feedback.entries
      .filter((e) => e.decision === "open")
      .map((entry) => {
        const location = locateChange(base, current, entry.change);
        return {
          ...entry.change,
          id: entry.key,
          from: location.from,
          to: Math.max(location.from, location.to),
          state: "open" as const,
        };
      });
  });
  function stepAnnotation(direction: number) {
    if (session && visibleChanges.length) {
      const index = visibleChanges.findIndex((c) => c.id === selectedId);
      selectChange(
        visibleChanges[(index + direction + visibleChanges.length) % visibleChanges.length]
      );
    } else if (feedback && visibleEntries.length) {
      const index = visibleEntries.findIndex((e) => e.key === selectedId);
      selectEntry(
        visibleEntries[(index + direction + visibleEntries.length) % visibleEntries.length]
      );
    }
  }

  export function isOpen() {
    return active;
  }
  export function closeWorkspace() {
    return close();
  }
  export function focusSearch() {
    dialog.querySelector<HTMLInputElement>('input[type="search"]')?.focus();
  }
  export function exportFeedback() {
    if (session) return returnFeedback();
  }
  export function discardForQuit() {
    clearTimeout(timer);
    saves?.discard();
    saves = null;
    session = null;
  }
  export async function flush() {
    clearTimeout(timer);
    if (saves && session) {
      try {
        const generation = await saves.flush();
        if (generation !== null) session.generation = generation;
        savedState = "Saved locally";
      } catch (e) {
        savedState = "Not saved — retry or export a recovery copy";
        error = String(e);
        throw e;
      }
    }
  }
  function stage() {
    if (!saves || !session) return;
    try {
      saves.stage($state.snapshot(session));
      savedState = "Saving…";
    } catch (e) {
      error = `Recovery journal: ${String(e)}. Keep this window open until saving succeeds.`;
    }
    clearTimeout(timer);
    timer = setTimeout(() => {
      void flush().catch(() => {});
    }, 350);
  }
  async function show() {
    active = true;
    await tick();
    if (!dialog.open) dialog.showModal();
  }
  async function close() {
    try {
      await flush();
      dialog.close();
      active = false;
    } catch {
      /* Keep all local work visible and recoverable. */
    }
  }
  async function action(work: () => Promise<void>) {
    if (busy) return;
    busy = true;
    error = "";
    notice = "";
    try {
      await work();
    } catch (e) {
      error = String(e);
      // File opening can fail before the workspace has ever been shown.
      await show();
    } finally {
      busy = false;
    }
  }

  export async function openFile(path?: string) {
    await action(async () => {
      await flush();
      await prepareWriting();
      const selected =
        path ??
        (await open({
          title: "Open a Kindling review or feedback file",
          multiple: false,
          filters: [
            {
              name: "Kindling editorial files",
              extensions: ["kindling-review", "kindling-feedback"],
            },
          ],
        }));
      if (!selected || typeof selected !== "string") return;
      const packageData = await invoke<EditorialPackage & { saved_generation?: number | null }>(
        "open_editorial_package",
        {
          path: selected,
        }
      );
      validateEditorialPackage(packageData);
      let nextSaves: EditorialSaves | null = null;
      let nextSession: EditorialSession | null = null;
      if (packageData.kind === "review") {
        nextSaves = new EditorialSaves(
          packageData.round,
          packageData.session,
          packageData.saved_generation
        );
        nextSession = nextSaves.recover(packageData.session) ?? {
          reviewer_id: window.crypto.randomUUID(),
          name: "",
          generation: 0,
          document: manuscript(packageData.round.sources).toJSON(),
          changes: [],
          position: 1,
          reading_position: 1,
        };
        validateEditorialPackage({ ...packageData, session: nextSession });
      }
      // Commit workspace state only after the complete incoming/recovered review validates.
      round = packageData.round;
      if (nextSession)
        restoreTracking(
          manuscript(round.sources),
          Node.fromJSON(editorialSchema, nextSession.document),
          nextSession.changes
        );
      received = packageData;
      selectedId = null;
      feedback = null;
      saves = nextSaves;
      session = nextSession;
      screen = session ? "review" : "preview";
      filter = "open";
      if (session) stage();
      manuscriptVersion++;
      await show();
      await tick();
      if (session) {
        prose?.select(session.position, session.position, false);
        prose?.restoreReadingPosition(session.reading_position ?? session.position);
      }
    });
    if (error && !active) await show();
  }

  export async function openProject(id: string) {
    await action(async () => {
      await flush();
      await prepareWriting();
      projectId = id;
      [exportSources, rounds] = await Promise.all([
        invoke<EditorialSource[]>("editorial_sources", { projectId: id }),
        invoke<EditorialRound[]>("list_editorial_rounds", { projectId: id }),
      ]);
      saves = null;
      session = null;
      feedback = null;
      round = null;
      screen = "export";
      selectedChapters = [];
      await show();
    });
    if (error && !active) await show();
  }

  async function exportReview() {
    await action(async () => {
      const path = await save({
        title: "Export for editorial review",
        defaultPath: `${roundName}.kindling-review`,
        filters: [{ name: "Kindling review", extensions: ["kindling-review"] }],
      });
      if (!path) return;
      const created = await invoke<EditorialRound>("export_editorial_review", {
        projectId,
        name: roundName,
        brief,
        chapterIds: selectedChapters,
        path,
      });
      rounds = [created, ...rounds];
      notice = `Review package saved to ${path}. Give this file to your editor.`;
    });
  }

  function updateDocument(doc: Node, edit: { before: Node; mapping: Mapping }) {
    if (!session || !round) return;
    session.changes = trackChanges(baseline!, doc, $state.snapshot(session.changes), edit);
    session.document = doc.toJSON();
    stage();
  }
  function updateSelection(from: number, to: number, explicit: boolean) {
    selection = { from, to };
    reanchorReady = explicit;
    if (session) {
      session.position = from;
      stage();
    }
  }
  function updateReadingPosition(position: number) {
    if (session && session.reading_position !== position) {
      session.reading_position = position;
      stage();
    }
  }
  function composeComment() {
    showComment = true;
    void tick().then(() =>
      dialog.querySelector<HTMLTextAreaElement>(".comment-compose textarea")?.focus()
    );
  }
  function chooseAnnotation(id: string) {
    selectedId = id;
    reanchorReady = false;
    replyText = "";
  }
  function addComment() {
    if (!session || !round || !commentText.trim() || !session.name.trim()) return;
    const base = manuscript(round.sources),
      proposed = Node.fromJSON(editorialSchema, session.document);
    const deltas = changesBetween(base, proposed);
    const inverse = changeMap(deltas).invert();
    const containing = deltas.find(
      (d) => d.fromB <= selection.from && d.toB >= selection.to && d.fromB < d.toB
    );
    const from = inverse.map(selection.from, -1),
      to = inverse.map(selection.to, 1);
    session.changes.push({
      id: window.crypto.randomUUID(),
      revision: 1,
      kind: "comment",
      from,
      to,
      before: base.slice(from, to).toJSON(),
      after: containing ? proposed.slice(selection.from, selection.to).toJSON() : null,
      anchor_offset: containing
        ? [selection.from - containing.fromB, selection.to - containing.fromB]
        : null,
      state: "open",
      messages: [message(session.name, commentText)],
    });
    selectedId = session.changes[session.changes.length - 1].id;
    commentText = "";
    showComment = false;
    stage();
  }
  function selectChange(change: EditorialChange) {
    if (!round || !session) return;
    selectedId = change.id;
    replyText = "";
    const range = projectedRange(
      manuscript(round.sources),
      Node.fromJSON(editorialSchema, session.document),
      change
    );
    prose?.select(range.from, range.to);
  }
  function selectEntry(entry: FeedbackEntry) {
    if (!feedback) return;
    selectedId = entry.key;
    reanchorReady = false;
    replyText = "";
    const range = locateChange(
      manuscript(feedback.round.sources),
      manuscript(feedback.sources),
      entry.change
    );
    prose?.select(range.from, Math.max(range.from, range.to));
  }
  function withdraw(change: EditorialChange) {
    if (!session || !round) return;
    if (change.kind === "suggestion")
      prose?.replaceDocument(
        withdrawSuggestion(
          manuscript(round.sources),
          Node.fromJSON(editorialSchema, session.document),
          change
        )
      );
    else {
      change.state = "withdrawn";
      stage();
    }
    selectedId = null;
  }
  function reply() {
    if (!replyText.trim()) return;
    if (selectedChange && session?.name.trim()) {
      selectedChange.messages.push(message(session.name, replyText));
      stage();
      replyText = "";
    } else if (selectedEntry && feedback && writerName.trim()) {
      void action(async () => {
        feedback = await invoke<EditorialFeedback>("reply_editorial_feedback", {
          roundId: feedback!.round.id,
          version: feedback!.version,
          key: selectedEntry!.key,
          message: message(writerName, replyText),
        });
        replyText = "";
      });
    }
  }
  async function returnFeedback(recovery = false) {
    await action(async () => {
      if (!round || !session) return;
      if (!recovery) await flush();
      const path = await save({
        title: recovery ? "Export recovery copy" : "Return editorial feedback",
        defaultPath: `${round.title} — ${round.name} — ${session.name || "review"}${recovery ? " — recovery.kindling-review" : ".kindling-feedback"}`,
        filters: [
          {
            name: recovery ? "Kindling recovery review" : "Kindling feedback",
            extensions: [recovery ? "kindling-review" : "kindling-feedback"],
          },
        ],
      });
      if (!path) return;
      await invoke(recovery ? "export_editorial_recovery" : "export_editorial_feedback", {
        round: $state.snapshot(round),
        session: {
          ...$state.snapshot(session),
          generation: recovery ? saves!.recoveryGeneration() : session.generation,
        },
        path,
      });
      if (recovery) {
        notice = `Recovery review saved to ${path}. Open it in Kindling to resume your work, then export feedback normally.`;
        return;
      }
      notice = `Feedback saved to ${path}. Return this file to the writer. You can continue reviewing and export another response later.`;
    });
  }
  async function sendWriterReply() {
    await action(async () => {
      if (!feedback) return;
      const reviewerId = replyReviewer || reviewers[0]?.id;
      const reviewer = reviewers.find((r) => r.id === reviewerId);
      if (!reviewer) return;
      const path = await save({
        title: "Send replies and decisions to your editor",
        defaultPath: `${round!.title} — ${round!.name} — reply to ${reviewer.name}.kindling-review`,
        filters: [{ name: "Kindling review response", extensions: ["kindling-review"] }],
      });
      if (!path) return;
      await invoke("export_editorial_reply", { roundId: feedback.round.id, reviewerId, path });
      notice = `Response saved to ${path}. Your editor opens it to receive your replies and decisions while keeping their ongoing review.`;
    });
  }
  async function importFeedback() {
    await action(async () => {
      feedback = await invoke<EditorialFeedback>("import_editorial_feedback", {
        package: $state.snapshot(received),
      });
      screen = "feedback";
      manuscriptVersion++;
      selectedId = null;
      notice = "Feedback imported. Your manuscript has not changed. Review the suggestions below.";
    });
  }
  function restoreMappedView(
    view: ReturnType<NonNullable<typeof prose>["captureView"]>,
    previous: Node
  ) {
    if (!feedback) return;
    const map = changeMap(changesBetween(previous, manuscript(feedback.sources)));
    const from = map.map(view.from, 1),
      to = Math.max(from, map.map(view.to, -1));
    prose?.restoreView({
      ...view,
      from,
      to,
      reading: view.reading
        ? { ...view.reading, position: map.map(view.reading.position, 1) }
        : null,
    });
  }
  async function openRound(id: string) {
    await action(async () => {
      const view = prose?.captureView();
      const previous = feedback ? manuscript(feedback.sources) : null;
      feedback = await invoke<EditorialFeedback>("get_editorial_feedback", { roundId: id });
      round = feedback.round;
      screen = "feedback";
      manuscriptVersion++;
      selectedId = null;
      await tick();
      if (view && previous) restoreMappedView(view, previous);
    });
  }
  async function decide(entries: FeedbackEntry[], decision: string, reanchor = false) {
    await action(async () => {
      if (!feedback) return;
      await prepareWriting();
      const view = prose?.captureView();
      const previous = manuscript(feedback.sources);
      const replacements =
        decision === "accepted"
          ? prepareAcceptance($state.snapshot(feedback), entries, reanchor ? selection : undefined)
          : [];
      feedback = await invoke<EditorialFeedback>("decide_editorial_feedback", {
        roundId: feedback.round.id,
        version: feedback.version,
        keys: entries.map((e) => e.key),
        decision,
        replacements,
      });
      if (replacements.length) {
        manuscriptVersion++;
        await tick();
        if (view) restoreMappedView(view, previous);
      }
      await onManuscriptChanged();
      notice =
        decision === "accepted"
          ? "Changes accepted. Previous prose is preserved in scene draft history."
          : "Review decision saved.";
    });
  }
  function findNext(direction: number) {
    searchIndex += direction;
    searchCount = prose?.find(search, searchIndex) ?? 0;
  }

  async function drainFiles() {
    if (draining) {
      drainAgain = true;
      return;
    }
    draining = true;
    try {
      incoming.push(...(await invoke<string[]>("take_editorial_open_files")));
      while (incoming.length) {
        if (busy) {
          setTimeout(() => {
            void drainFiles();
          }, 100);
          break;
        }
        await openFile(incoming.shift()!);
      }
    } catch (e) {
      error = String(e);
    } finally {
      draining = false;
      if (drainAgain) {
        drainAgain = false;
        void drainFiles();
      }
    }
  }
  onMount(() => {
    const unlisten = listen("editorial-open", () => {
      void drainFiles();
    });
    // Listen before draining: native events during startup remain in the queue.
    void unlisten
      .then(() => drainFiles())
      .catch((e) => {
        error = String(e);
      });
    return () => {
      clearTimeout(timer);
      void unlisten.then((stop) => stop());
    };
  });
</script>

<dialog
  bind:this={dialog}
  class="editorial-workspace"
  aria-label="Editorial workspace"
  oncancel={(e) => {
    e.preventDefault();
    void close();
  }}
  onkeydown={(e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      focusSearch();
    }
    e.stopPropagation();
  }}
>
  <header>
    <div>
      <span class="workspace-label">Kindling / Editorial</span>
      <h1>{round?.title ?? "Editorial review"}</h1>
      {#if round}<p>{round.name}</p>{/if}
    </div>
    <div class="header-actions">
      {#if screen === "review"}<span role="status">{savedState}</span><button
          disabled={busy || !session?.name.trim()}
          onclick={() => returnFeedback()}>Export feedback</button
        >{/if}
      <button disabled={busy} onclick={close}>Back to Kindling</button>
    </div>
  </header>
  {#if error}<div role="alert" class="workspace-error">
      <p>{error}</p>
      {#if session}<button onclick={() => flush().catch(() => {})}>Retry saving</button><button
          onclick={() => returnFeedback(true)}>Export recovery copy</button
        >{/if}
    </div>{/if}
  {#if notice}<p role="status" class="workspace-notice">{notice}</p>{/if}

  {#if screen === "export"}
    <section class="workspace-intro">
      <h2>Send a manuscript for review</h2>
      <p>
        Your editor opens the package in Kindling and works offline. Returned feedback comes back
        here for your decisions.
      </p>
      <label
        >Review round<input
          bind:value={roundName}
          placeholder="Developmental edit — September"
        /></label
      >
      <label
        >Brief for your editor <span>(optional)</span><textarea
          bind:value={brief}
          placeholder="What would you like your editor to focus on?"
          rows="3"
        ></textarea></label
      >
      <fieldset>
        <legend>Chapters <span>(none selected means the whole manuscript)</span></legend>
        {#each exportSources.filter((s, i) => !exportSources
              .slice(0, i)
              .some((p) => p.chapter_id === s.chapter_id)) as chapter}
          <label class="checkbox"
            ><input
              type="checkbox"
              value={chapter.chapter_id}
              bind:group={selectedChapters}
            />{chapter.chapter}</label
          >
        {/each}
      </fieldset>
      <button disabled={busy || !roundName.trim() || !exportSources.length} onclick={exportReview}
        >Export review package</button
      >
      <button disabled={busy} onclick={() => openFile()}>Open review or feedback file…</button>
      {#if rounds.length}<h2 class="mt-8">Review rounds</h2>
        <ul>
          {#each rounds as item}<li>
              <button onclick={() => openRound(item.id)}>{item.name}</button><span
                >{new Date(item.created_at).toLocaleDateString()}</span
              >
            </li>{/each}
        </ul>{/if}
    </section>
  {:else if screen === "preview" && received}
    <section class="workspace-intro">
      <h2>Feedback from {received.session?.name}</h2>
      <p>
        {new Set(received.round.sources.map((s) => s.scene_id)).size} scenes · {received.session?.changes.filter(
          (c) => c.kind === "suggestion"
        ).length} suggestions · {received.session?.changes.filter((c) => c.kind === "comment")
          .length} comments
      </p>
      <p>
        Import adds feedback to the original review round. You decide which suggestions to accept,
        including where the manuscript has changed since it was sent.
      </p>
      <button disabled={busy} onclick={importFeedback}>Import and review feedback</button>
    </section>
  {:else if round && (session || feedback)}
    <div class="workspace-layout">
      <nav aria-label="Manuscript navigation">
        {#if round.brief}<details open>
            <summary>Writer’s brief</summary>
            <p class="brief">{round.brief}</p>
          </details>{/if}
        <label
          >Find in manuscript<input
            type="search"
            bind:value={search}
            oninput={() => {
              searchIndex = 0;
              searchCount = prose?.find(search, 0) ?? 0;
            }}
            onkeydown={(e) => {
              if (e.key === "Enter") findNext(e.shiftKey ? -1 : 1);
            }}
          /></label
        >
        {#if search}<div class="inline-actions">
            <button onclick={() => findNext(-1)} aria-label="Previous search match">Previous</button
            ><button onclick={() => findNext(1)} aria-label="Next search match">Next</button><span
              >{searchCount} matches</span
            >
          </div>{/if}
        {#each navigation as source, index}
          {#if index === 0 || source.chapter_id !== navigation[index - 1].chapter_id}<h2>
              {source.chapter}
            </h2>{/if}
          <button class="scene-link" onclick={() => prose?.navigate(source.id)}
            >{source.scene}</button
          >
        {/each}
      </nav>
      <section class="manuscript-column" aria-label="Manuscript">
        {#key `${round.id}:${manuscriptVersion}`}
          <EditorialManuscript
            bind:this={prose}
            {sources}
            initial={screen === "review" ? session!.document : manuscript(sources).toJSON()}
            changes={manuscriptAnnotations}
            readonly={screen === "feedback"}
            {markup}
            selected={selectedId}
            onChange={updateDocument}
            onSelection={updateSelection}
            onComment={composeComment}
            onReadingPosition={updateReadingPosition}
            onError={(e) => (error = e)}
            onAnnotation={chooseAnnotation}
          />
        {/key}
      </section>
      <aside aria-label="Editorial feedback">
        <h2>{screen === "review" ? "Your review" : "Returned feedback"}</h2>
        {#if session}<label
            >Your name<input
              bind:value={session.name}
              oninput={stage}
              placeholder="Name shown with your feedback"
            /></label
          >{:else}<label>Your name<input bind:value={writerName} /></label>{/if}
        <label class="checkbox"><input type="checkbox" bind:checked={markup} />Show markup</label>
        <div class="inline-actions">
          <button
            disabled={session ? !visibleChanges.length : !visibleEntries.length}
            onclick={() => stepAnnotation(-1)}>Previous annotation</button
          ><button
            disabled={session ? !visibleChanges.length : !visibleEntries.length}
            onclick={() => stepAnnotation(1)}>Next annotation</button
          >
        </div>
        <label
          >Show<select bind:value={filter}
            ><option value="open">Pending feedback</option><option value="all">All feedback</option
            ><option value="resolved">Resolved comments</option><option value="accepted"
              >Accepted</option
            ><option value="rejected">Rejected</option></select
          ></label
        >
        {#if session}
          <button onclick={composeComment}>Add comment</button>
          {#if showComment}<div class="comment-compose">
              <p>
                {selection.from === selection.to
                  ? "General feedback at this reading position"
                  : "Comment on the selected passage"}
              </p>
              <label>Comment<textarea bind:value={commentText} rows="4"></textarea></label><button
                disabled={!commentText.trim() || !session.name.trim()}
                onclick={addComment}>Save comment</button
              >
            </div>{/if}
          {#if !visibleChanges.length}<p class="empty-state">
              Read and edit the manuscript naturally. Your edits become suggestions; the writer’s
              original stays preserved.
            </p>{/if}
          {#each visibleChanges as change}<article>
              <button
                class="annotation-title"
                aria-pressed={change.id === selectedId}
                onclick={() => selectChange(change)}
                >{change.kind === "comment"
                  ? "Comment"
                  : !sliceText(change.before)
                    ? "Insertion"
                    : !sliceText(change.after)
                      ? "Deletion"
                      : "Change"} · {change.writer_decision
                  ? `Writer ${change.writer_decision}`
                  : change.state}</button
              >
              <p class="excerpt">
                {sliceText(change.before).slice(0, 120) ||
                  sliceText(change.after).slice(0, 120) ||
                  "Formatting or general feedback"}
              </p>
              {#if change.id === selectedId}
                {#if change.kind === "suggestion"}<div class="comparison">
                    <div class="original-prose">
                      {@html sliceHtml(change.before) || "Insertion point"}
                    </div>
                    <div class="suggested-prose">
                      {@html sliceHtml(change.after) || "Delete passage"}
                    </div>
                    <p class="hint">Edit the suggestion directly in the manuscript to refine it.</p>
                  </div>{/if}
                {#each change.messages as note}<p class="thread">
                    <strong>{note.author}</strong><br />{note.text}
                  </p>{/each}
                <label>Reply<textarea bind:value={replyText}></textarea></label><button
                  disabled={!replyText.trim() || !session.name.trim()}
                  onclick={reply}>Reply</button
                >
                {#if change.kind === "comment"}<button
                    onclick={() => {
                      change.state =
                        (change.writer_decision ?? change.state) === "resolved"
                          ? "open"
                          : "resolved";
                      if (change.writer_decision) {
                        change.revision++;
                        change.writer_decision = undefined;
                      }
                      stage();
                    }}>{change.state === "resolved" ? "Reopen" : "Resolve"}</button
                  >{/if}
                <button onclick={() => withdraw(change)}>Withdraw {change.kind}</button>
              {/if}
            </article>{/each}
        {:else if feedback}
          {#if reviewers.length}
            <details>
              <summary>Send replies to your editor</summary>
              <label
                >Editor<select bind:value={replyReviewer}>
                  {#each reviewers as reviewer}<option value={reviewer.id}>{reviewer.name}</option
                    >{/each}
                </select></label
              >
              <button disabled={busy} onclick={sendWriterReply}>Export replies and decisions</button
              >
            </details>
          {/if}
          <div class="inline-actions">
            <button
              disabled={busy ||
                !visibleEntries.some(
                  (e) => e.change.kind === "suggestion" && e.decision === "open"
                )}
              onclick={() =>
                decide(
                  visibleEntries.filter(
                    (e) => e.change.kind === "suggestion" && e.decision === "open"
                  ),
                  "accepted"
                )}>Accept visible suggestions</button
            ><button
              disabled={busy ||
                !visibleEntries.some(
                  (e) => e.change.kind === "suggestion" && e.decision === "open"
                )}
              onclick={() =>
                decide(
                  visibleEntries.filter(
                    (e) => e.change.kind === "suggestion" && e.decision === "open"
                  ),
                  "rejected"
                )}>Reject visible suggestions</button
            >
          </div>
          <button disabled={busy} onclick={() => openRound(feedback!.round.id)}
            >Refresh manuscript</button
          >
          {#if !visibleEntries.length}<p class="empty-state">
              No feedback in this view. You can open another returned file or choose All feedback.
            </p>{/if}
          {#each visibleEntries as entry}<article>
              <button
                class="annotation-title"
                aria-pressed={entry.key === selectedId}
                onclick={() => selectEntry(entry)}
                >{entry.reviewer} · {entry.change.kind} · {entry.decision}</button
              >
              <p class="excerpt">
                {sliceText(entry.change.before).slice(0, 120) ||
                  sliceText(entry.change.after).slice(0, 120) ||
                  "Formatting or general feedback"}
              </p>
              {#if entry.key === selectedId}
                {@const location = locateChange(
                  manuscript(feedback.round.sources),
                  manuscript(feedback.sources),
                  entry.change
                )}
                <div class="comparison">
                  <h3>Original passage</h3>
                  <div class="original-prose">
                    {@html sliceHtml(entry.change.before) || "Insertion point"}
                  </div>
                  <h3>Current passage</h3>
                  <p>
                    {manuscript(feedback.sources).textBetween(
                      location.from,
                      Math.max(location.from, location.to),
                      "\n"
                    ) || "Empty passage"}
                  </p>
                  {#if entry.change.kind === "suggestion"}<h3>Suggested passage</h3>
                    <div class="suggested-prose">
                      {@html sliceHtml(entry.change.after) || "Delete passage"}
                    </div>{/if}
                </div>
                {#if location.conflict && entry.decision === "open"}<p class="conflict">
                    This passage changed after export. Select the intended passage in the current
                    manuscript before applying this suggestion.
                  </p>{/if}
                {#each entry.change.messages as note}<p class="thread">
                    <strong>{note.author}</strong><br />{note.text}
                  </p>{/each}
                <label>Reply<textarea bind:value={replyText}></textarea></label><button
                  disabled={busy || !replyText.trim() || !writerName.trim()}
                  onclick={reply}>Reply</button
                >
                {#if entry.decision === "open" && entry.change.kind === "suggestion"}<button
                    disabled={busy || location.conflict}
                    onclick={() => decide([entry], "accepted")}>Accept</button
                  ><button disabled={busy} onclick={() => decide([entry], "rejected")}
                    >Reject</button
                  >
                  {#if location.conflict}<button
                      disabled={busy || !reanchorReady}
                      onclick={() => decide([entry], "accepted", true)}
                      >Apply to selected passage</button
                    >{/if}
                {:else if entry.change.kind === "comment"}<button
                    disabled={busy}
                    onclick={() =>
                      decide([entry], entry.decision === "resolved" ? "open" : "resolved")}
                    >{entry.decision === "resolved" ? "Reopen" : "Resolve"}</button
                  >{/if}
              {/if}
            </article>{/each}
        {/if}
      </aside>
    </div>
  {/if}
</dialog>

<style>
  .editorial-workspace {
    position: fixed;
    inset: 0;
    margin: 0;
    width: 100vw;
    height: 100vh;
    max-width: none;
    max-height: none;
    border: none;
    border-radius: 0;
    padding: 0;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: var(--font-ui);
  }
  .editorial-workspace[open] {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-l);
    padding: var(--space-s) var(--space-l);
    border-bottom: 1px solid var(--color-border);
  }
  h1 {
    font-family: var(--font-display);
    font-size: var(--text-h2);
  }
  h2 {
    font-family: var(--font-display);
    font-size: var(--text-h3);
    margin-block: var(--space-m);
  }
  h3,
  .workspace-label {
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  header p,
  .header-actions span,
  .hint,
  .inline-actions span,
  legend span,
  label span {
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .header-actions,
  .inline-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2xs);
  }
  .workspace-layout {
    display: grid;
    grid-template-columns: minmax(10rem, 1fr) minmax(0, 3fr) minmax(17rem, 1.4fr);
    flex: 1;
    min-height: 0;
  }
  nav,
  aside,
  .manuscript-column {
    overflow-y: auto;
    padding: var(--space-m);
  }
  nav {
    border-right: 1px solid var(--color-border);
  }
  aside {
    border-left: 1px solid var(--color-border);
  }
  .manuscript-column {
    max-width: calc(var(--measure) + 6rem);
    width: 100%;
    margin-inline: auto;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-block: var(--space-m);
  }
  label.checkbox {
    flex-direction: row;
    align-items: center;
    gap: var(--space-s);
  }
  input,
  textarea,
  select {
    max-width: 100%;
  }
  button {
    padding: var(--space-xs) var(--space-s);
  }
  .scene-link {
    display: block;
    text-align: left;
    width: 100%;
    padding-block: var(--space-s);
  }
  .workspace-intro {
    overflow-y: auto;
    width: 100%;
    padding: var(--space-xl);
    max-width: var(--measure);
    margin-inline: auto;
  }
  .workspace-intro > p,
  .brief,
  .empty-state,
  .excerpt,
  .thread,
  .comparison > p {
    font-family: var(--font-body);
    font-size: var(--text-body);
    max-width: var(--measure);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .workspace-intro li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    padding-block: var(--space-s);
  }
  .workspace-intro li span {
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  article {
    border-top: 1px solid var(--color-border);
    padding-block: var(--space-m);
  }
  .annotation-title {
    text-align: left;
    font-size: var(--text-ui);
    padding-inline: 0;
  }
  .annotation-title[aria-pressed="true"] {
    color: var(--color-accent-text);
  }
  .thread {
    margin-block: var(--space-m);
  }
  .thread strong {
    font-family: var(--font-ui);
    font-size: var(--text-small);
  }
  .comparison {
    padding-block: var(--space-s);
  }
  .comparison h3 {
    margin-top: var(--space-m);
  }
  .original-prose,
  .suggested-prose {
    font-family: var(--font-body);
    font-size: var(--text-body);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .original-prose {
    border-left: 2px solid var(--color-error);
    padding-left: var(--space-xs);
  }
  .suggested-prose {
    border-left: 2px solid var(--color-success);
    padding-left: var(--space-xs);
    margin-block: var(--space-s);
  }
  .conflict {
    border-left: 2px solid var(--color-warning);
    padding: var(--space-s);
    color: var(--color-warning);
  }
  .workspace-error,
  .workspace-notice {
    padding: var(--space-s) var(--space-xl);
    border-bottom: 1px solid var(--color-border);
  }
  .workspace-error {
    color: var(--color-error);
  }
  .workspace-notice {
    color: var(--color-text);
  }
</style>
