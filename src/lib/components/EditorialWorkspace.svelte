<script lang="ts">
  import { onMount, tick, type Snippet } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { Node, Slice, Fragment } from "@tiptap/pm/model";
  import type { Mapping } from "@tiptap/pm/transform";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";
  import ReviewSidebar from "./ReviewSidebar.svelte";
  import BrandWordmark from "./BrandWordmark.svelte";
  import RevisionsPanel from "./RevisionsPanel.svelte";
  import { localAnnotations, localSelection } from "../utils/localEditorial";
  import {
    acceptSuggestions,
    activeDocuments,
    revisionStatuses,
    type SceneReview,
  } from "../utils/revisions";
  import type { ReviewItem } from "../utils/reviewItems";
  import {
    Search,
    ChevronLeft,
    ChevronDown,
    ChevronRight,
    MoreHorizontal,
    PanelLeftClose,
    PanelLeftOpen,
    Check,
    X,
    RefreshCw,
    Download,
    History,
    FolderOpen,
    FileOutput,
    RotateCcw,
  } from "lucide-svelte";
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
    references,
  }: {
    prepareWriting: () => Promise<void>;
    onManuscriptChanged: () => Promise<void>;
    references?: Snippet<[string]>;
  } = $props();
  let dialog: HTMLElement;
  let local = $state(false);
  let focusedScene = $state("");
  let sceneReviews = $state<SceneReview[]>([]);
  let localSources = $state<EditorialSource[]>([]);
  let showHistory = $state(false);
  let showSearch = $state(false);
  let showNavigation = $state(true);
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
  let exportScope = $state("all");
  let menuPosition = $state<{ x: number; y: number } | null>(null);
  let menuTrigger = $state<HTMLElement>();
  let packageReturn = $state<{
    screen: "review" | "feedback";
    view: ReturnType<NonNullable<typeof prose>["captureView"]> | undefined;
  } | null>(null);
  const manuscriptMenu = $derived<MenuItem[]>([
    ...(local
      ? [
          {
            label: "Draft history",
            icon: History,
            action: async () => {
              if (!session || (await reviewDecisions())) showHistory = true;
            },
          },
          {
            label: "Start suggestions from current prose",
            icon: RotateCcw,
            action: async () => {
              await suggest(true);
            },
          },
          { label: "", divider: true, action: () => {} },
          {
            label: "Review packages and rounds…",
            icon: FileOutput,
            action: () => openProject(projectId),
          },
        ]
      : []),
    ...(session
      ? [
          {
            label: "Export feedback",
            icon: FileOutput,
            disabled: !session.name.trim() || busy,
            action: () => returnFeedback(),
          },
          { label: "Export recovery copy", icon: Download, action: () => returnFeedback(true) },
        ]
      : []),
    { label: "Open review or feedback file…", icon: FolderOpen, action: () => openFile() },
  ]);
  function dismissManuscriptMenu(event: MouseEvent) {
    if (!menuPosition || !(event.target instanceof Element)) return;
    if (
      !menuTrigger?.contains(event.target) &&
      !event.target.closest('[data-testid="context-menu"]')
    )
      menuPosition = null;
  }
  let selectedChapters = $state<string[]>([]);
  let selection = $state({ from: 1, to: 1 });
  let reanchorReady = $state(false);
  let selectedId = $state<string | null>(null);
  let commentText = $state(""),
    writerName = $state("Writer");
  let showComment = $state(false),
    markup = $state(false),
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
  const lockSources = $derived(
    local
      ? sources.map((s) => ({
          ...s,
          locked: localSources.find((current) => current.id === s.id)?.locked ?? true,
        }))
      : sources
  );
  const legacy = $derived(local ? localAnnotations(sceneReviews, lockSources) : []);
  const inactive = $derived(
    local
      ? sceneReviews.flatMap((review) =>
          review.data.annotations
            .filter((a) => !activeDocuments(review).some((d) => d.id === a.document_id))
            .map((annotation) => ({ review, annotation }))
        )
      : []
  );
  const focusedReview = $derived(sceneReviews.find((r) => r.scene_id === focusedScene));
  const focusedSource = $derived(lockSources.find((s) => s.scene_id === focusedScene));

  const baseline = $derived(round ? manuscript(round.sources) : null);
  const navigation = $derived(
    sources.filter((s, i) => !sources.slice(0, i).some((p) => p.scene_id === s.scene_id))
  );
  const visibleEntries = $derived(
    feedback?.entries.filter((e) => filter === "all" || e.decision === filter) ?? []
  );
  const manuscriptAnnotations = $derived.by(() => {
    if (session)
      return [
        ...session.changes,
        ...legacy
          .filter((a) => a.annotation.state === "open")
          .map((a) => ({ ...a.change, from: a.from, to: a.to, unapplied: true })),
      ];
    if (!feedback) return [];
    const base = manuscript(feedback.round.sources),
      current = manuscript(feedback.sources);
    return [
      ...legacy
        .filter((a) => a.annotation.state === "open")
        .map((a) => ({ ...a.change, from: a.from, to: a.to, unapplied: true })),
      ...feedback.entries
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
        }),
    ];
  });
  const items = $derived<ReviewItem[]>([
    ...legacy.map((a) => ({
      id: a.key,
      kind: a.change.kind,
      state: a.annotation.state,
      author: a.annotation.messages[0]?.author || "Writer",
      excerpt: a.annotation.quote || a.annotation.replacement || "General feedback",
      messages: a.change.messages,
      before: a.annotation.quote
        ? sliceHtml(
            new Slice(Fragment.from(editorialSchema.text(a.annotation.quote)), 0, 0).toJSON()
          )
        : "",
      after: sliceHtml(a.change.after),
      current: a.current,
      conflict: a.conflict,
      locked: a.locked,
      reanchor: !session,
      decide: !session,
      resolve: !session && a.change.kind === "comment",
    })),
    ...(session?.changes.map((c) => ({
      id: c.id,
      kind: c.kind,
      state: c.writer_decision ?? c.state,
      author: c.messages[0]?.author || session!.name,
      excerpt: sliceText(c.before) || sliceText(c.after),
      messages: c.messages,
      before: sliceHtml(c.before),
      after: sliceHtml(c.after),
      withdraw: !c.writer_decision || c.writer_decision === "open",
      resolve: c.kind === "comment",
    })) ?? []),
    ...(!session
      ? (feedback?.entries.map((e) => {
          const location = locateChange(
            manuscript(feedback!.round.sources),
            manuscript(feedback!.sources),
            e.change
          );
          return {
            id: e.key,
            kind: e.change.kind,
            state: e.decision,
            author: e.reviewer,
            excerpt: sliceText(e.change.before) || sliceText(e.change.after),
            messages: e.change.messages,
            before: sliceHtml(e.change.before),
            after: sliceHtml(e.change.after),
            current: manuscript(feedback!.sources).textBetween(
              location.from,
              Math.max(location.from, location.to),
              "\n"
            ),
            conflict: location.conflict,
            decide: true,
            resolve: e.change.kind === "comment",
          };
        }) ?? [])
      : []),
    ...inactive.map(({ review, annotation: a }) => ({
      id: `inactive:${review.scene_id}:${a.id}`,
      kind: a.replacement === null ? ("comment" as const) : ("suggestion" as const),
      state: a.state,
      author: a.messages[0]?.author || "Writer",
      excerpt: a.quote || a.replacement || "General feedback",
      messages: a.messages.map((m, i) => ({ ...m, id: `${a.id}:${i}` })),
      before: a.quote
        ? sliceHtml(new Slice(Fragment.from(editorialSchema.text(a.quote)), 0, 0).toJSON())
        : "",
      after: a.replacement
        ? sliceHtml(new Slice(Fragment.from(editorialSchema.text(a.replacement)), 0, 0).toJSON())
        : "",
      unavailable: `Saved on inactive ${review.mode === "page" ? "beat" : "page"} prose in “${sources.find((s) => s.scene_id === review.scene_id)?.scene || "this scene"}”. Return to Writing and switch that scene to ${review.mode === "page" ? "Beat" : "Page"} mode to act on this feedback.`,
    })),
  ]);
  function stepAnnotation(direction: number) {
    const visible = items.filter((i) => filter === "all" || i.state === filter);
    if (!visible.length) return;
    selectItem(
      visible[
        (visible.findIndex((i) => i.id === selectedId) + direction + visible.length) %
          visible.length
      ].id
    );
  }
  function selectItem(id: string) {
    const hidden = inactive.find(
      ({ review, annotation }) => `inactive:${review.scene_id}:${annotation.id}` === id
    );
    if (hidden) {
      chooseAnnotation(id);
      navigateScene(hidden.review.scene_id);
      return;
    }
    const old = legacy.find((a) => a.key === id);
    if (old) {
      chooseAnnotation(id);
      focusedScene = old.review.scene_id;
      const range =
        session && baseline
          ? projectedRange(baseline, Node.fromJSON(editorialSchema, session.document), {
              ...old.change,
              from: old.from,
              to: old.to,
            })
          : old;
      prose?.select(range.from, range.to);
    } else if (session) {
      const change = session.changes.find((c) => c.id === id);
      if (change) selectChange(change);
    } else {
      const entry = feedback?.entries.find((e) => e.key === id);
      if (entry) selectEntry(entry);
    }
  }
  function rememberName(name: string) {
    writerName = name;
    localStorage.setItem("kindling.editorial.name", name);
    if (session) {
      session.name = name;
      stage();
    }
  }
  let localLoad = 0;
  async function loadLocalScenes() {
    if (!local) return;
    const id = projectId,
      request = ++localLoad;
    const current = await invoke<EditorialSource[]>("editorial_sources", { projectId: id });
    const reviews = await invoke<SceneReview[]>("get_project_scene_reviews", { projectId: id });
    if (!local || projectId !== id || request !== localLoad) return;
    localSources = current;
    if (feedback) feedback.sources = current;
    sceneReviews = reviews;
  }
  export async function refreshLocalContext() {
    if (!local || !active) return;
    try {
      await loadLocalScenes();
      await tick();
    } catch (e) {
      error = String(e);
    }
  }
  export function isLocal() {
    return local && active;
  }
  export function navigateScene(sceneId: string) {
    focusedScene = sceneId;
    const source = sources.find((s) => s.scene_id === sceneId);
    if (source) prose?.navigate(source.id);
  }
  export async function openLocal(
    id: string,
    sceneId: string,
    cursor?: { sourceId: string; from: number; to: number }
  ) {
    await action(async () => {
      await flush();
      await prepareWriting();
      const data = await invoke<EditorialPackage>("open_local_editorial_review", {
        projectId: id,
        fresh: false,
      });
      validateEditorialPackage(data);
      const nextFeedback = await invoke<EditorialFeedback>("get_editorial_feedback", {
        roundId: data.round.id,
      });
      const current = await invoke<EditorialSource[]>("editorial_sources", { projectId: id });
      const reviews = await invoke<SceneReview[]>("get_project_scene_reviews", { projectId: id });
      local = true;
      packageReturn = null;
      projectId = id;
      round = data.round;
      focusedScene = sceneId;
      session = null;
      saves = null;
      feedback = { ...nextFeedback, sources: current };
      sceneReviews = reviews;
      localSources = current;
      screen = "feedback";
      filter = "open";
      selectedId = null;
      resetSearch();
      showComment = false;
      writerName = localStorage.getItem("kindling.editorial.name") || "Writer";
      manuscriptVersion++;
      await show();
      await tick();
      if (cursor) {
        let offset = 0;
        for (const source of sources) {
          if (source.id === cursor.sourceId) {
            prose?.select(offset + cursor.from, offset + cursor.to, false);
            return;
          }
          offset += manuscript([source]).content.size;
        }
      }
      navigateScene(sceneId);
    });
  }
  async function suggest(fresh = false) {
    return action(async () => {
      await flush();
      const data = await invoke<EditorialPackage & { saved_generation?: number | null }>(
        "open_local_editorial_review",
        { projectId, fresh }
      );
      validateEditorialPackage(data);
      await loadLocalScenes();
      const nextSaves = new EditorialSaves(data.round, data.session, data.saved_generation);
      const nextSession = nextSaves.recover(data.session) ?? {
        reviewer_id: window.crypto.randomUUID(),
        name: writerName,
        generation: 0,
        document: manuscript(data.round.sources).toJSON(),
        changes: [],
        position: 1,
      };
      validateEditorialPackage({ ...data, session: nextSession });
      round = data.round;
      saves = nextSaves;
      session = nextSession;
      restoreTracking(
        manuscript(round.sources),
        Node.fromJSON(editorialSchema, session.document),
        $state.snapshot(session.changes)
      );
      const view = prose?.captureView();
      const previous = prose?.currentDocument();
      screen = "review";
      selectedId = null;
      manuscriptVersion++;
      stage();
      await tick();
      if (view && previous)
        restoreMappedView(view, previous, Node.fromJSON(editorialSchema, session.document));
    });
  }
  async function reviewDecisions() {
    return action(async () => {
      await flush();
      const view = prose?.captureView();
      const previous = prose?.currentDocument();
      const next = await invoke<EditorialFeedback>("get_editorial_feedback", {
        roundId: round!.id,
      });
      session = null;
      saves = null;
      feedback = next;
      screen = "feedback";
      selectedId = null;
      await loadLocalScenes();
      manuscriptVersion++;
      await tick();
      if (view && previous) restoreMappedView(view, previous);
    });
  }
  async function legacyDecision(id: string, decision: string, reanchor = false) {
    const old = legacy.find((a) => a.key === id);
    if (!old) return;
    await action(async () => {
      if (old.locked) throw new Error("Unlock this scene before changing its review.");
      const expected = $state.snapshot(old.review);
      let data = window.structuredClone(expected.data);
      let next = null;
      if (reanchor) {
        const range = localSelection(sources, selection.from, selection.to);
        if (range.source.scene_id !== old.review.scene_id)
          throw new Error("Select a passage in the original scene.");
        Object.assign(data.annotations.find((a) => a.id === old.annotation.id)!, {
          document_id: range.source.id,
          anchor_html: range.source.html,
          from: range.from,
          to: range.to,
          quote: range.quote,
        });
      }
      if (decision === "accepted")
        ({ data, next } = acceptSuggestions({ ...expected, data }, [old.annotation.id]));
      else if (decision !== "reanchor")
        data.annotations.find((a) => a.id === old.annotation.id)!.state = decision as
          | "open"
          | "resolved"
          | "rejected";
      const view = prose?.captureView();
      const previous = prose?.currentDocument();
      await invoke("save_scene_review", { expected, data, next });
      await loadLocalScenes();
      if (next) {
        manuscriptVersion++;
        await tick();
        if (view && previous) restoreMappedView(view, previous);
        await onManuscriptChanged();
      }
    });
  }
  function decideItem(id: string, decision: string, reanchor = false) {
    if (legacy.some((a) => a.key === id)) void legacyDecision(id, decision, reanchor);
    else {
      const entry = feedback?.entries.find((e) => e.key === id);
      if (entry) void decide([entry], decision, reanchor);
    }
  }
  async function replyItem(id: string, text: string): Promise<boolean> {
    return action(async () => {
      const old = legacy.find((a) => a.key === id);
      if (old) {
        if (old.locked) throw new Error("Unlock this scene before changing its review.");
        const expected = $state.snapshot(old.review);
        const data = window.structuredClone(expected.data);
        data.annotations
          .find((a) => a.id === old.annotation.id)!
          .messages.push({ author: writerName, text, created_at: new Date().toISOString() });
        await invoke("save_scene_review", { expected, data, next: null });
        await loadLocalScenes();
      } else if (session) {
        const change = session.changes.find((c) => c.id === id);
        if (!change) throw new Error("This conversation is no longer available.");
        change.messages.push(message(session.name, text));
        stage();
      } else if (feedback) {
        feedback = await invoke<EditorialFeedback>("reply_editorial_feedback", {
          roundId: feedback.round.id,
          version: feedback.version,
          key: id,
          message: message(writerName, text),
        });
      }
    });
  }
  function resolveItem(id: string) {
    const item = items.find((i) => i.id === id);
    if (!item) return;
    if (session) {
      const change = session.changes.find((c) => c.id === id)!;
      change.state = item.state === "resolved" ? "open" : "resolved";
      if (change.writer_decision) {
        change.revision++;
        change.writer_decision = undefined;
      }
      stage();
    } else decideItem(id, item.state === "resolved" ? "open" : "resolved");
  }
  function withdrawItem(id: string) {
    const change = session?.changes.find((c) => c.id === id);
    if (change) withdraw(change);
  }
  async function bulkLegacy(decision: string) {
    await action(async () => {
      const targets = legacy.filter(
        (a) =>
          a.annotation.state === "open" &&
          a.change.kind === "suggestion" &&
          (filter === "all" || filter === "open")
      );
      if (targets.some((a) => a.locked))
        throw new Error("Unlock the selected scenes before deciding on their suggestions.");
      const updates = [...new Set(targets.map((a) => a.review.scene_id))].map((id) => {
        const expected = $state.snapshot(sceneReviews.find((r) => r.scene_id === id)!);
        const ids = targets.filter((a) => a.review.scene_id === id).map((a) => a.annotation.id);
        if (decision === "accepted") return { expected, ...acceptSuggestions(expected, ids) };
        return {
          expected,
          data: {
            ...expected.data,
            annotations: expected.data.annotations.map((a) =>
              ids.includes(a.id) ? { ...a, state: "rejected" } : a
            ),
          },
          next: null,
        };
      });
      const view = prose?.captureView();
      const previous = prose?.currentDocument();
      await invoke("save_scene_review_batch", { updates });
      await loadLocalScenes();
      if (decision === "accepted") {
        manuscriptVersion++;
        await tick();
        if (view && previous) restoreMappedView(view, previous);
        await onManuscriptChanged();
      }
    });
  }
  async function setLocalStatus(status: string) {
    if (!focusedReview) return;
    await action(async () => {
      await invoke("save_scene_review", {
        expected: $state.snapshot(focusedReview),
        data: { ...$state.snapshot(focusedReview.data), status },
        next: null,
      });
      await loadLocalScenes();
      await onManuscriptChanged();
    });
  }

  export function isOpen() {
    return active;
  }
  export async function closeWorkspace() {
    menuPosition = null;
    await flush();
    packageReturn = null;
    active = false;
    showHistory = false;
  }
  export function focusSearch() {
    showSearch = true;
    void tick().then(() => {
      dialog.querySelector<HTMLInputElement>('input[type="search"]')?.focus();
      searchCount = prose?.find(search, searchIndex) ?? 0;
    });
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
        const queue = saves;
        const generation = await queue.flush();
        if (generation !== null) session.generation = generation;
        const committed = queue.savedSession();
        if (local && round && committed)
          await invoke("import_editorial_feedback", {
            package: {
              format: "kindling-editorial",
              version: 1,
              kind: "feedback",
              round: $state.snapshot(round),
              session: committed,
            },
          });
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
    const alreadyOpen = active;
    active = true;
    await tick();
    if (!alreadyOpen) dialog.focus();
  }
  async function close() {
    try {
      await closeWorkspace();
    } catch {
      /* Keep all local work visible and recoverable. */
    }
  }
  async function back() {
    if (screen === "export" && packageReturn) {
      const previous = packageReturn;
      await action(async () => {
        packageReturn = null;
        screen = previous.screen;
        await tick();
        if (showSearch) prose?.find(search, searchIndex);
        if (previous.view) prose?.restoreView(previous.view);
      });
    } else await close();
  }
  async function action(work: () => Promise<void>) {
    if (busy) return false;
    busy = true;
    error = "";
    notice = "";
    try {
      await work();
      return true;
    } catch (e) {
      error = String(e);
      // File opening can fail before the workspace has ever been shown.
      await show();
      return false;
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
          name: localStorage.getItem("kindling.editorial.name") || "",
          generation: 0,
          document: manuscript(packageData.round.sources).toJSON(),
          changes: [],
          position: 1,
          reading_position: 1,
        };
        validateEditorialPackage({ ...packageData, session: nextSession });
      }
      // Commit workspace state only after the complete incoming/recovered review validates.
      local = false;
      focusedScene = "";
      packageReturn = null;
      sceneReviews = [];
      showComment = false;
      resetSearch();
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
        focusSceneAt(session.position);
      }
    });
    if (error && !active) await show();
  }

  export async function openProject(id: string) {
    await action(async () => {
      await flush();
      await prepareWriting();
      const previous =
        active && local && projectId === id
          ? screen === "review" || screen === "feedback"
            ? { screen, view: prose?.captureView() }
            : packageReturn
          : null;
      const [nextSources, nextRounds] = await Promise.all([
        invoke<EditorialSource[]>("editorial_sources", { projectId: id }),
        invoke<EditorialRound[]>("list_editorial_rounds", { projectId: id }),
      ]);
      projectId = id;
      exportSources = nextSources;
      rounds = nextRounds;
      packageReturn = previous;
      if (!previous) {
        saves = null;
        session = null;
        feedback = null;
        round = null;
      }
      screen = "export";
      selectedChapters = [];
      exportScope = "all";
      await show();
    });
    if (error && !active) await show();
  }

  async function exportReview() {
    if (exportScope === "selected" && !selectedChapters.length) return;
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
        chapterIds: exportScope === "selected" ? selectedChapters : [],
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
  function focusSceneAt(from: number) {
    const doc = prose?.currentDocument();
    focusedScene = sources[0]?.scene_id ?? "";
    if (doc) {
      const cursor = doc.resolve(Math.min(from, doc.content.size));
      for (let depth = cursor.depth; depth > 0; depth--) {
        const source = sources.find((s) => s.id === cursor.node(depth).attrs.source);
        if (source) {
          focusedScene = source.scene_id;
          break;
        }
      }
    }
  }
  function updateSelection(from: number, to: number, explicit: boolean) {
    selection = { from, to };
    focusSceneAt(from);
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
    if (local && !session) {
      void suggest().then((opened) => {
        if (opened) showComment = true;
      });
      return;
    }
    showComment = true;
    void tick().then(() =>
      dialog.querySelector<HTMLTextAreaElement>(".comment-compose textarea")?.focus()
    );
  }
  function chooseAnnotation(id: string) {
    selectedId = id;
    reanchorReady = false;
  }
  function addComment() {
    if (!session || !round || !commentText.trim() || !session.name.trim()) return;
    const base = manuscript(round.sources),
      proposed = Node.fromJSON(editorialSchema, session.document);
    if (local) {
      let locked = false;
      proposed.nodesBetween(selection.from, Math.max(selection.from, selection.to), (node) => {
        if (lockSources.some((s) => s.id === node.attrs.source && s.locked)) locked = true;
      });
      if (locked) {
        error = "Unlock this scene before adding feedback.";
        return;
      }
    }
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
      await tick();
      focusSceneAt(1);
    });
  }
  function restoreMappedView(
    view: ReturnType<NonNullable<typeof prose>["captureView"]>,
    previous: Node,
    current?: Node
  ) {
    if (!current && !feedback) return;
    const map = changeMap(changesBetween(previous, current ?? manuscript(feedback!.sources)));
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
      await flush();
      const view = prose?.captureView();
      const previous = feedback ? manuscript(feedback.sources) : null;
      feedback = await invoke<EditorialFeedback>("get_editorial_feedback", { roundId: id });
      packageReturn = null;
      round = feedback.round;
      session = null;
      saves = null;
      if (local) await loadLocalScenes();
      screen = "feedback";
      manuscriptVersion++;
      selectedId = null;
      await tick();
      if (view && previous) restoreMappedView(view, previous);
      focusSceneAt(prose?.captureView().from ?? 1);
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
      if (local) await loadLocalScenes();
      await onManuscriptChanged();
      notice =
        decision === "accepted"
          ? "Changes accepted. Previous prose is preserved in scene draft history."
          : "Review decision saved.";
    });
  }
  function closeSearch() {
    showSearch = false;
    prose?.find("", 0);
  }
  function findNext(direction: number) {
    searchIndex += direction;
    searchCount = prose?.find(search, searchIndex) ?? 0;
  }
  function resetSearch() {
    search = "";
    searchIndex = 0;
    searchCount = 0;
    showSearch = false;
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

<svelte:window onpointerdowncapture={dismissManuscriptMenu} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section
  bind:this={dialog}
  class="editorial-workspace"
  class:active
  hidden={!active}
  aria-label="Editorial workspace"
  tabindex="-1"
  onkeydown={(e) => {
    if (e.key === "Escape" && menuPosition) {
      e.preventDefault();
      menuPosition = null;
      menuTrigger?.focus();
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      focusSearch();
    }
    e.stopPropagation();
  }}
>
  <header class="workspace-header">
    <div class="heading">
      <button
        class="icon"
        aria-label={packageReturn
          ? "Return to revisions"
          : local
            ? "Return to writing"
            : "Close review"}
        title={packageReturn ? "Return to revisions" : local ? "Return to writing" : "Close review"}
        disabled={busy}
        onclick={back}><ChevronLeft size={18} /></button
      >
      <div>
        {#if round?.title && (screen === "export" || focusedSource?.scene)}
          <span class="eyebrow">{round.title}</span>
        {/if}
        <h1>
          {screen === "export"
            ? "Review packages"
            : focusedSource?.scene || round?.title || "Editorial review"}
        </h1>
      </div>
    </div>
    <div class="workspace-actions">
      {#if local && focusedReview && screen !== "export"}<span class="compact-select"
          ><select
            aria-label="Revision status"
            disabled={busy || focusedSource?.locked}
            value={focusedReview.data.status}
            onchange={(e) => setLocalStatus(e.currentTarget.value)}
            >{#each Object.entries(revisionStatuses) as [value, label]}<option {value}
                >{label}</option
              >{/each}</select
          ><ChevronDown size={14} /></span
        >{/if}
      {#if session}<span class="save-state" role="status">{savedState}</span>{/if}
      {#if screen === "review" || screen === "feedback"}
        {#if local}<span class="compact-select"
            ><select
              aria-label="Editor mode"
              value={screen}
              disabled={busy}
              onchange={(e) => {
                if (e.currentTarget.value === "review") void suggest();
                else if (e.currentTarget.value === "writing") void close();
                else void reviewDecisions();
              }}
              ><option value="writing">Writing</option><option value="feedback">Reviewing</option
              ><option value="review">Suggesting</option></select
            ><ChevronDown size={14} /></span
          >{:else}<span class="mode">{session ? "Suggesting" : "Reviewing feedback"}</span>{/if}
        <button
          class="icon"
          title="Find in manuscript"
          aria-label="Find in manuscript"
          onclick={focusSearch}><Search size={18} /></button
        >
        <button
          bind:this={menuTrigger}
          class="icon"
          aria-label="Manuscript actions"
          aria-haspopup="menu"
          aria-expanded={!!menuPosition}
          onclick={(event) => {
            // The menu mounts before a native click reaches window. Keep this
            // opening click out of ContextMenu's outside-click handler.
            event.stopPropagation();
            const rect = menuTrigger!.getBoundingClientRect();
            menuPosition = menuPosition ? null : { x: rect.right, y: rect.bottom };
          }}><MoreHorizontal size={20} /></button
        >
        {#if active && menuPosition}
          <ContextMenu
            items={manuscriptMenu}
            x={menuPosition.x}
            y={menuPosition.y}
            onClose={() => (menuPosition = null)}
          />
        {/if}
      {/if}
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
    <div class="package-layout">
      <section class="package-form" aria-labelledby="package-title">
        <h2 id="package-title">Send a manuscript for review</h2>
        <p class="package-description">
          Create a file your editor can open in Kindling. They can read, suggest edits, and return
          their feedback without an account.
        </p>
        <label
          >Review round<input
            bind:value={roundName}
            placeholder="Developmental edit — September"
          /></label
        >
        <label
          ><span>Brief for your editor <span class="optional">Optional</span></span><textarea
            bind:value={brief}
            placeholder="What would you like your editor to focus on?"
            rows="4"
          ></textarea></label
        >
        <fieldset class="package-scope">
          <legend>Manuscript to include</legend>
          <label class="scope-choice"
            ><input
              type="radio"
              name="editorial-package-scope"
              bind:group={exportScope}
              value="all"
            />Entire manuscript</label
          >
          <label class="scope-choice"
            ><input
              type="radio"
              name="editorial-package-scope"
              bind:group={exportScope}
              value="selected"
            />Selected chapters</label
          >
          {#if exportScope === "selected"}
            <div class="chapter-choices">
              {#each exportSources.filter((s, i) => !exportSources
                    .slice(0, i)
                    .some((p) => p.chapter_id === s.chapter_id)) as chapter}
                <label class="scope-choice"
                  ><input
                    type="checkbox"
                    value={chapter.chapter_id}
                    bind:group={selectedChapters}
                  />{chapter.chapter}</label
                >
              {/each}
              {#if !selectedChapters.length}<p class="scope-hint">
                  Choose at least one chapter.
                </p>{/if}
            </div>
          {/if}
        </fieldset>
        <div class="package-export">
          <button
            class="primary-action"
            disabled={busy ||
              !roundName.trim() ||
              !exportSources.length ||
              (exportScope === "selected" && !selectedChapters.length)}
            onclick={exportReview}><FileOutput size={16} />Export review package</button
          >
          <p class="scope-hint">Save the file, then share it with your editor.</p>
        </div>
      </section>
      <aside class="package-rounds" aria-label="Review rounds">
        <h2>Continue a review</h2>
        <button class="open-package" disabled={busy} onclick={() => openFile()}
          ><FolderOpen size={16} />Open review or feedback file…</button
        >
        <p class="scope-hint">
          Open returned feedback to review suggestions and send your decisions back.
        </p>
        <h3>Review rounds</h3>
        {#if rounds.length}
          <ul>
            {#each rounds as item}<li>
                <button
                  class="round-link"
                  aria-label={item.name}
                  disabled={busy}
                  onclick={() => openRound(item.id)}
                  ><span>{item.name}</span><time datetime={item.created_at}
                    >{new Date(item.created_at).toLocaleString(undefined, {
                      month: "short",
                      day: "numeric",
                      year: "numeric",
                      hour: "numeric",
                      minute: "2-digit",
                      second: "2-digit",
                    })}</time
                  ><ChevronRight size={16} /></button
                >
              </li>{/each}
          </ul>
        {:else}<p class="scope-hint">
            Your review rounds will appear here after you export a package.
          </p>{/if}
      </aside>
    </div>
  {:else if screen === "preview" && received}
    {@const sceneCount = new Set(received.round.sources.map((s) => s.scene_id)).size}
    {@const suggestionCount = received.session!.changes.filter(
      (c) => c.kind === "suggestion"
    ).length}
    {@const commentCount = received.session!.changes.filter((c) => c.kind === "comment").length}
    <div class="package-layout feedback-preview">
      <section class="feedback-summary" aria-labelledby="feedback-heading">
        <h2 id="feedback-heading">Feedback from {received.session!.name}</h2>
        <p class="package-description">
          Bring your editor’s feedback into the original review round, then read it alongside your
          manuscript.
        </p>
        <ul class="feedback-counts" aria-label="Feedback in this file">
          <li>
            <strong>{sceneCount}</strong>{sceneCount === 1 ? "scene included" : "scenes included"}
          </li>
          <li>
            <strong>{suggestionCount}</strong>{suggestionCount === 1 ? "suggestion" : "suggestions"}
          </li>
          <li><strong>{commentCount}</strong>{commentCount === 1 ? "comment" : "comments"}</li>
        </ul>
        <div class="feedback-next">
          <h3>What happens next</h3>
          <p>
            Read comments, reply to your editor, and accept or reject suggestions. If you’ve
            rewritten a passage, Kindling helps you place its feedback before applying an edit.
          </p>
          <p>
            You can work through the feedback at your own pace and export your replies and decisions
            when you’re ready.
          </p>
        </div>
        <div class="package-export">
          <button class="primary-action" disabled={busy} onclick={importFeedback}
            ><FolderOpen size={16} />Import and review feedback</button
          >
          <p class="scope-hint">Your manuscript changes only when you accept a suggestion.</p>
        </div>
      </section>
      <aside class="package-rounds feedback-details" aria-label="Review details">
        <h2>Review details</h2>
        <dl>
          <dt>Manuscript</dt>
          <dd>{received.round.title}</dd>
          <dt>Review round</dt>
          <dd>{received.round.name}</dd>
        </dl>
        {#if received.round.brief.trim()}
          <h3>Your original brief</h3>
          <p class="brief">{received.round.brief}</p>
        {/if}
      </aside>
    </div>
  {:else if round && (session || feedback)}
    <div class="workspace-layout">
      {#if !local && showNavigation}<nav class="manuscript-nav" aria-label="Manuscript navigation">
          <div class="nav-title">
            <BrandWordmark />
            <button
              class="icon"
              aria-label="Hide manuscript navigation"
              onclick={() => (showNavigation = false)}><PanelLeftClose size={16} /></button
            >
          </div>
          <div class="nav-context">
            <h2>{round.title}</h2>
            <p class="round-name">{round.name}</p>
          </div>
          {#if round.brief}<details>
              <summary>Writer’s brief</summary>
              <p class="brief">{round.brief}</p>
            </details>{/if}
          {#each navigation as source, index}{#if index === 0 || source.chapter_id !== navigation[index - 1].chapter_id}<h3
              >
                {source.chapter}
              </h3>{/if}<button
              class="scene-link"
              aria-current={focusedScene === source.scene_id ? "location" : undefined}
              onclick={() => {
                focusedScene = source.scene_id;
                prose?.navigate(source.id);
              }}>{source.scene}</button
            >{/each}
        </nav>{/if}
      <section class="manuscript-region" aria-label="Manuscript">
        {#if showSearch}<div class="search-bar">
            <Search size={16} /><input
              type="search"
              aria-label="Find in manuscript"
              bind:value={search}
              placeholder="Find in manuscript"
              oninput={() => {
                searchIndex = 0;
                searchCount = prose?.find(search, 0) ?? 0;
              }}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  findNext(e.shiftKey ? -1 : 1);
                }
                if (e.key === "Escape") {
                  e.preventDefault();
                  closeSearch();
                }
              }}
            /><span aria-live="polite"
              >{search
                ? searchCount
                  ? `${(((searchIndex % searchCount) + searchCount) % searchCount) + 1} of ${searchCount}`
                  : "No matches"
                : ""}</span
            ><button disabled={!searchCount} onclick={() => findNext(-1)}>Previous</button><button
              disabled={!searchCount}
              onclick={() => findNext(1)}>Next</button
            ><button onclick={closeSearch}>Done</button>
          </div>{/if}
        <div class="manuscript-column">
          {#key `${round.id}:${manuscriptVersion}`}
            <EditorialManuscript
              bind:this={prose}
              {sources}
              initial={screen === "review" ? session!.document : manuscript(sources).toJSON()}
              changes={manuscriptAnnotations}
              protectLocked={local}
              {lockSources}
              canComment={local || !!session}
              readonly={screen === "feedback"}
              {markup}
              selected={selectedId}
              onChange={updateDocument}
              onSelection={updateSelection}
              onComment={composeComment}
              onReadingPosition={updateReadingPosition}
              onSearchResults={(count) => (searchCount = count)}
              initialSearch={{ query: showSearch ? search : "", index: searchIndex }}
              onError={(e) => (error = e)}
              onAnnotation={selectItem}
              onActivate={chooseAnnotation}
            >
              {#snippet toolbar()}
                {#if !local && !showNavigation}<button
                    class="icon"
                    aria-label="Show manuscript navigation"
                    onclick={() => (showNavigation = true)}><PanelLeftOpen size={16} /></button
                  >{/if}
                <span class="compact-select"
                  ><select
                    class="compact-control"
                    aria-label="Markup view"
                    value={markup ? "all" : "simple"}
                    onchange={(e) => (markup = e.currentTarget.value === "all")}
                    ><option value="simple">Simple markup</option><option value="all"
                      >All markup</option
                    ></select
                  ><ChevronDown size={14} /></span
                >
              {/snippet}
            </EditorialManuscript>
          {/key}
        </div>
      </section>
      {#snippet contextualReferences()}{@render references?.(focusedScene)}{/snippet}
      <ReviewSidebar
        {items}
        selected={selectedId}
        bind:filter
        name={session?.name ?? writerName}
        onName={rememberName}
        onSelect={selectItem}
        onStep={stepAnnotation}
        onReply={replyItem}
        onDecide={decideItem}
        onWithdraw={withdrawItem}
        onResolve={resolveItem}
        canReanchor={reanchorReady}
        {busy}
        composing={showComment}
        bind:comment={commentText}
        onComment={addComment}
        onCancelComment={() => (showComment = false)}
        references={local && references ? contextualReferences : undefined}
      >
        {#snippet options()}
          {#if feedback && !session}
            {@const suggestions = visibleEntries.filter(
              (e) => e.decision === "open" && e.change.kind === "suggestion"
            )}
            {#if suggestions.length}
              <div class="review-menu-group" role="group" aria-label="Current review round">
                <p class="review-menu-caption">Review round · {feedback.round.name}</p>
                <button
                  class="accept-decision"
                  aria-label="Accept visible suggestions in this round"
                  disabled={busy}
                  onclick={() => decide(suggestions, "accepted")}
                  ><Check size={16} />Accept visible suggestions</button
                >
                <button
                  class="reject-decision"
                  aria-label="Reject visible suggestions in this round"
                  disabled={busy}
                  onclick={() => decide(suggestions, "rejected")}
                  ><X size={16} />Reject visible suggestions</button
                >
              </div>
            {/if}
            {#if (filter === "all" || filter === "open") && legacy.some((a) => a.annotation.state === "open" && a.change.kind === "suggestion")}
              <div class="review-menu-group" role="group" aria-label="Active scene prose">
                <p class="review-menu-caption">Active scene prose</p>
                <button
                  class="accept-decision"
                  aria-label="Accept visible suggestions on active scene prose"
                  disabled={busy}
                  onclick={() => bulkLegacy("accepted")}
                  ><Check size={16} />Accept visible suggestions</button
                ><button
                  class="reject-decision"
                  aria-label="Reject visible suggestions on active scene prose"
                  disabled={busy}
                  onclick={() => bulkLegacy("rejected")}
                  ><X size={16} />Reject visible suggestions</button
                >
              </div>
            {/if}
            <div class="review-menu-group">
              <button disabled={busy} onclick={() => openRound(feedback!.round.id)}
                ><RefreshCw size={16} />Refresh manuscript</button
              >
            </div>
            {#if reviewers.length}
              <div class="review-menu-group">
                <label class="review-menu-field"
                  >Send to editor<span class="compact-select"
                    ><select class="compact-control" bind:value={replyReviewer}
                      >{#each reviewers as reviewer}<option value={reviewer.id}
                          >{reviewer.name}</option
                        >{/each}</select
                    ><ChevronDown size={14} /></span
                  ></label
                ><button disabled={busy} onclick={sendWriterReply}
                  ><Download size={16} />Export replies and decisions</button
                >
              </div>
            {/if}
          {/if}
        {/snippet}
      </ReviewSidebar>
    </div>
  {/if}
  {#if showHistory && focusedSource}<RevisionsPanel
      sceneId={focusedScene}
      {projectId}
      title={focusedSource.scene}
      locked={focusedSource.locked}
      onApplied={async () => {
        await action(async () => {
          const view = prose?.captureView();
          const previous = prose?.currentDocument();
          await loadLocalScenes();
          manuscriptVersion++;
          await tick();
          if (view && previous) restoreMappedView(view, previous);
          await onManuscriptChanged();
        });
      }}
      onClose={() => {
        void action(async () => {
          await loadLocalScenes();
          showHistory = false;
        });
      }}
    />{/if}
</section>

<style>
  .editorial-workspace {
    display: none;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    height: 100%;
    color: var(--color-text);
    background: var(--color-bg);
    font-family: var(--font-ui);
    font-size: var(--text-small);
  }
  .editorial-workspace.active {
    display: flex;
  }
  .workspace-header {
    display: flex;
    flex-shrink: 0;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-s);
    padding: var(--space-s) var(--space-m);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }
  .heading,
  .workspace-actions {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
  .heading {
    min-width: 0;
  }
  .eyebrow,
  .save-state,
  .round-name {
    color: var(--color-text-muted);
    font-size: var(--text-eyebrow);
  }
  h1 {
    font-family: var(--font-display);
    font-size: var(--text-h3);
    margin: 0;
  }
  h2,
  h3 {
    font-size: var(--text-small);
    margin: var(--space-s) 0;
  }
  button {
    padding: var(--space-2xs) var(--space-xs);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }
  button:hover {
    background: var(--color-surface-sunken);
  }
  button:disabled {
    color: var(--color-disabled-text);
    background: var(--color-disabled-bg);
    cursor: default;
  }
  .icon {
    border: 0;
    display: flex;
    padding: var(--space-2xs);
  }
  input,
  textarea,
  select {
    font-size: var(--text-base);
    max-width: 100%;
    box-sizing: border-box;
  }
  .workspace-actions select {
    width: auto;
  }
  .workspace-actions :global([data-testid="context-menu"]) {
    width: 22rem;
    max-width: calc(100vw - var(--space-s));
  }
  .workspace-actions :global([data-testid="context-menu"] svg) {
    flex-shrink: 0;
  }
  .compact-select {
    display: inline-grid;
    align-items: center;
  }
  .compact-select select {
    grid-area: 1 / 1;
    appearance: none;
  }
  .compact-select :global(svg) {
    grid-area: 1 / 1;
    justify-self: end;
    margin-right: var(--space-2xs);
    pointer-events: none;
    color: var(--color-text-muted);
  }
  /* Compact desktop review controls, as requested; full setup forms retain their scale. */
  .workspace-actions select,
  .search-bar input,
  .compact-control {
    font-size: var(--text-small);
    line-height: var(--leading);
    padding: var(--space-3xs) var(--space-2xs);
  }
  .compact-select select {
    padding-right: var(--space-l);
  }
  .workspace-layout {
    display: flex;
    flex: 1;
    min-height: 0;
    min-width: 0;
  }
  .manuscript-nav {
    width: 20rem;
    flex-shrink: 0;
    padding: var(--space-s);
    box-sizing: border-box;
    overflow: auto;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
  }
  .nav-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .scene-link {
    display: block;
    width: 100%;
    text-align: left;
    border: 0;
    margin-block: var(--space-3xs);
  }
  .nav-context {
    padding-block: var(--space-s);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: var(--space-s);
  }
  .nav-context h2,
  .manuscript-nav h3 {
    font-family: var(--font-ui);
    font-weight: 600;
    font-size: var(--text-ui);
    line-height: var(--leading-tight);
    margin: 0;
  }
  .nav-context .round-name {
    margin: var(--space-2xs) 0 0;
  }
  .nav-context h2 {
    font-size: var(--text-base);
  }
  .manuscript-nav h3 {
    margin-top: var(--space-m);
    margin-bottom: var(--space-2xs);
  }
  .manuscript-nav .scene-link {
    padding-left: var(--space-m);
    color: var(--color-text-muted);
  }
  .manuscript-nav .scene-link[aria-current="location"] {
    color: var(--color-accent-text);
    background: var(--color-accent-wash);
  }
  .manuscript-region {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-inline: var(--space-m);
  }
  .manuscript-column {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .search-bar {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--space-2xs);
    padding: var(--space-xs);
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }
  .search-bar input {
    flex: 1;
    min-width: 0;
  }
  .search-bar span {
    white-space: nowrap;
  }
  .accept-decision:not(:disabled) {
    color: var(--color-success);
  }
  .reject-decision:not(:disabled) {
    color: var(--color-error);
  }
  .workspace-error {
    padding: var(--space-xs) var(--space-m);
    border-bottom: 1px solid var(--color-error);
    background: var(--color-surface);
  }
  .workspace-error p {
    margin: 0 0 var(--space-2xs);
  }
  .workspace-notice {
    padding: var(--space-2xs) var(--space-m);
    margin: 0;
    border-bottom: 1px solid var(--color-border);
  }
  .package-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(15rem, 19rem);
    gap: var(--space-xl);
    width: min(68rem, 100%);
    padding: var(--space-xl);
    margin-inline: auto;
    overflow: auto;
  }
  .package-form h2,
  .feedback-summary h2 {
    font-family: var(--font-display);
    font-size: var(--text-h2);
    margin: 0 0 var(--space-xs);
  }
  .package-description {
    color: var(--color-text-muted);
    line-height: var(--leading);
    margin-bottom: var(--space-l);
  }
  .package-form > label {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xs);
    margin-bottom: var(--space-m);
  }
  .optional {
    color: var(--color-text-muted);
    font-size: var(--text-eyebrow);
    margin-left: var(--space-2xs);
  }
  .package-form input:not([type]),
  .package-form textarea {
    width: 100%;
    padding: var(--space-2xs) var(--space-xs);
    font-size: var(--text-small);
    line-height: var(--leading);
  }
  .package-scope {
    border: 0;
    padding: 0;
    margin-bottom: var(--space-m);
  }
  .package-scope legend {
    margin-bottom: var(--space-2xs);
  }
  .scope-choice {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    padding-block: var(--space-2xs);
    cursor: pointer;
  }
  .chapter-choices {
    padding-left: var(--space-m);
    margin-left: var(--space-2xs);
    border-left: 1px solid var(--color-border);
  }
  .scope-hint {
    color: var(--color-text-muted);
    font-size: var(--text-eyebrow);
    line-height: var(--leading);
    margin-block: var(--space-2xs);
  }
  .package-export {
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-m);
  }
  .primary-action,
  .open-package {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    padding: var(--space-xs) var(--space-s);
  }
  .primary-action {
    background: var(--color-accent);
    color: var(--color-on-accent);
    border-color: var(--color-accent);
  }
  .primary-action:hover {
    background: var(--color-accent-text);
  }
  .package-rounds {
    border-left: 1px solid var(--color-border);
    padding-left: var(--space-l);
  }
  .package-rounds h2,
  .package-rounds h3 {
    font-family: var(--font-display);
    font-size: var(--text-body-lg);
    margin-top: 0;
  }
  .package-rounds h3 {
    margin-top: var(--space-l);
  }
  .open-package {
    width: 100%;
    text-align: left;
  }
  .package-rounds ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .round-link {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-3xs) var(--space-2xs);
    text-align: left;
    border: 0;
    border-bottom: 1px solid var(--color-border);
    border-radius: 0;
    width: 100%;
    padding: var(--space-xs) 0;
    overflow-wrap: anywhere;
  }
  .round-link time {
    grid-column: 1;
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
  }
  .round-link :global(svg) {
    grid-column: 2;
    grid-row: 1 / 3;
    align-self: center;
  }
  @media (max-width: 1100px) {
    .package-layout {
      gap: var(--space-l);
      padding: var(--space-l);
    }
  }
  .feedback-preview {
    align-items: start;
    overflow-wrap: anywhere;
  }
  .feedback-counts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-s);
    list-style: none;
    padding: var(--space-m) 0;
    margin: 0;
    border-block: 1px solid var(--color-border);
  }
  .feedback-counts strong {
    display: block;
    font-family: var(--font-display);
    font-size: var(--text-h2);
    font-weight: 500;
  }
  .feedback-next {
    margin-block: var(--space-l);
    line-height: var(--leading);
  }
  .feedback-next h3 {
    font-family: var(--font-display);
    font-size: var(--text-body-lg);
    margin-bottom: var(--space-xs);
  }
  .feedback-next p + p {
    margin-top: var(--space-xs);
  }
  .feedback-details dt {
    color: var(--color-text-muted);
    font-size: var(--text-eyebrow);
    margin-top: var(--space-s);
  }
  .feedback-details dd {
    margin: var(--space-2xs) 0 0;
  }
  @media (max-width: 800px) {
    .feedback-preview {
      grid-template-columns: minmax(0, 1fr);
    }
    .feedback-details {
      border-left: 0;
      border-top: 1px solid var(--color-border);
      padding: var(--space-m) 0 0;
    }
  }
  .brief {
    white-space: pre-wrap;
    line-height: var(--leading-relaxed);
  }
</style>
