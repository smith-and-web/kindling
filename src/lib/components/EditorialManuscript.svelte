<script lang="ts">
  import { onMount, tick } from "svelte";
  import ProseToolbar from "./ProseToolbar.svelte";
  import { MessageSquare } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { Editor } from "@tiptap/core";
  import type { Mapping } from "@tiptap/pm/transform";
  import { Plugin, PluginKey, TextSelection } from "@tiptap/pm/state";
  import { Decoration, DecorationSet } from "@tiptap/pm/view";
  import { DOMSerializer, Node, Slice } from "@tiptap/pm/model";
  import {
    editorialExtensions,
    editorialSchema,
    manuscript,
    normalizeOwnership,
    projectedRange,
    lockedProseChanged,
    searchManuscript,
    type EditorialSource,
    type EditorialChange,
  } from "../utils/editorial";

  let {
    sources,
    initial,
    changes = [],
    readonly = false,
    markup = true,
    selected = null,
    onChange,
    onSelection,
    onComment,
    onError,
    onAnnotation,
    onActivate,
    onReadingPosition,
    toolbar,
    showSections = true,
    protectLocked = false,
    canComment = true,
    lockSources,
    onSearchResults,
    initialSearch,
  }: {
    sources: EditorialSource[];
    initial: ReturnType<Node["toJSON"]>;
    changes?: (EditorialChange & { unapplied?: boolean })[];
    readonly?: boolean;
    markup?: boolean;
    selected?: string | null;
    onChange: (doc: Node, edit: { before: Node; mapping: Mapping }) => void;
    onSelection: (from: number, to: number, explicit: boolean) => void;
    onComment: () => void;
    onError: (message: string) => void;
    onAnnotation: (id: string) => void;
    onActivate?: (id: string) => void;
    onReadingPosition: (position: number) => void;
    toolbar?: Snippet;
    showSections?: boolean;
    protectLocked?: boolean;
    canComment?: boolean;
    lockSources?: EditorialSource[];
    onSearchResults?: (count: number) => void;
    initialSearch?: { query: string; index: number };
  } = $props();
  let element: HTMLDivElement;
  let editor = $state.raw<Editor>();
  let revision = $state(0);
  let programmaticSelection = false;
  let navigationRequest = 0;
  let searchMatch = $state<{ from: number; to: number } | null>(null);
  let searchQuery = "";
  let searchIndex = 0;
  function refreshSearch(doc: Node) {
    const results = searchManuscript(doc, searchQuery);
    searchMatch =
      results[((searchIndex % results.length) + results.length) % results.length] ?? null;
    onSearchResults?.(results.length);
    return results.length;
  }
  async function revealSelection(request: number) {
    await tick();
    if (!editor || editor.isDestroyed || request !== navigationRequest) return;
    const container = element.parentElement!;
    const bounds = container.getBoundingClientRect();
    const toolbar = container.querySelector(".editorial-format")?.getBoundingClientRect();
    const gutter =
      parseFloat(window.getComputedStyle(container).getPropertyValue("--space-xs")) || 12;
    const top = Math.max(bounds.top, toolbar?.bottom ?? bounds.top) + gutter;
    const bottom = bounds.bottom - gutter;
    const start = editor.view.coordsAtPos(editor.state.selection.from);
    const end = editor.view.coordsAtPos(editor.state.selection.to);
    if (start.top < top || end.bottom > bottom) {
      // Scroll this pane explicitly: read-only or unfocused ProseMirror views
      // need not honor a transaction's scrollIntoView request.
      container.scrollTop += start.top - top;
    }
  }
  const key = new PluginKey("manuscript-markup");
  const base = $derived(manuscript(sources));

  export function currentDocument() {
    return editor
      ? normalizeOwnership(editor.state.doc, sources)
      : Node.fromJSON(editorialSchema, initial);
  }
  export function replaceDocument(doc: Node) {
    if (!editor) return;
    doc = Node.fromJSON(editor.schema, doc.toJSON());
    const before = normalizeOwnership(editor.state.doc, sources);
    const start = before.content.findDiffStart(doc.content);
    if (start === null) return;
    const end = before.content.findDiffEnd(doc.content)!;
    const overlap = Math.max(0, start - Math.min(end.a, end.b));
    // A precise inverse edit keeps unrelated comments and native undo mappings intact.
    editor.view.dispatch(
      editor.state.tr.replace(start, end.a + overlap, doc.slice(start, end.b + overlap))
    );
  }
  export function select(from: number, to = from, focus = true, reveal = true) {
    if (!editor) return;
    const doc = editor.state.doc;
    const selection = TextSelection.between(
      doc.resolve(Math.max(0, Math.min(from, doc.content.size))),
      doc.resolve(Math.max(0, Math.min(to, doc.content.size)))
    );
    programmaticSelection = true;
    try {
      editor.view.dispatch(editor.state.tr.setSelection(selection).scrollIntoView());
    } finally {
      programmaticSelection = false;
    }
    if (focus) editor.view.focus();
    const request = ++navigationRequest;
    if (reveal) void revealSelection(request);
  }
  export function navigate(sourceId: string) {
    let position: number | undefined;
    editor?.state.doc.forEach((node, offset) => {
      if (position === undefined && node.attrs.source === sourceId) position = offset + 1;
    });
    if (position !== undefined) select(position);
  }
  export function find(query: string, index: number) {
    if (!editor) return 0;
    searchQuery = query;
    searchIndex = index;
    const count = refreshSearch(editor.state.doc);
    if (searchMatch) select(searchMatch.from, searchMatch.to, false);
    return count;
  }

  function readingAnchor() {
    if (!editor || !element?.parentElement) return null;
    const container = element.parentElement.getBoundingClientRect();
    const bounds = editor.view.dom.getBoundingClientRect();
    const toolbar =
      element.parentElement.querySelector(".editorial-format")?.getBoundingClientRect().height ?? 0;
    const point = editor.view.posAtCoords({
      left: bounds.left + 8,
      top: Math.max(bounds.top + 2, container.top + toolbar + 20),
    });
    return point
      ? { position: point.pos, offset: editor.view.coordsAtPos(point.pos).top - container.top }
      : null;
  }
  export function captureView() {
    return {
      from: editor?.state.selection.from ?? 1,
      to: editor?.state.selection.to ?? 1,
      scroll: element?.parentElement?.scrollTop ?? 0,
      focused: editor?.view.hasFocus() ?? false,
      reading: readingAnchor(),
    };
  }
  export function restoreView(view: ReturnType<typeof captureView>) {
    select(view.from, view.to, view.focused, false);
    if (element?.parentElement) {
      const container = element.parentElement;
      if (view.reading && editor) {
        const pos = Math.max(0, Math.min(view.reading.position, editor.state.doc.content.size));
        container.scrollTop +=
          editor.view.coordsAtPos(pos).top -
          container.getBoundingClientRect().top -
          view.reading.offset;
      } else container.scrollTop = view.scroll;
    }
  }
  export function restoreReadingPosition(position: number) {
    navigationRequest++;
    if (!editor || !element.parentElement) return;
    const point = editor.view.coordsAtPos(
      Math.max(1, Math.min(position, editor.state.doc.content.size - 1))
    );
    const container = element.parentElement;
    const toolbar =
      container.querySelector(".editorial-format")?.getBoundingClientRect().height ?? 0;
    container.scrollTop += point.top - container.getBoundingClientRect().top - toolbar - 16;
  }

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: editorialExtensions,
      content: initial,
      editable: !readonly,
      editorProps: {
        handleDOMEvents: {
          mouseup: (view) => {
            const { from, to } = view.state.selection;
            onSelection(from, to, true);
            return false;
          },
        },
        handleClick: (_view, _position, event) => {
          const id = (event.target as HTMLElement).closest<HTMLElement>("[data-review-id]")?.dataset
            .reviewId;
          if (id) (onActivate ?? onAnnotation)(id);
          return false;
        },
        attributes: {
          class: "editorial-prose",
          "aria-label": readonly ? "Current manuscript" : "Manuscript with suggested edits",
          spellcheck: "true",
        },
        handleKeyDown: (_view, event) => {
          if (
            canComment &&
            (event.metaKey || event.ctrlKey) &&
            event.altKey &&
            event.key.toLowerCase() === "m"
          ) {
            event.preventDefault();
            onComment();
            return true;
          }
          return false;
        },
      },
      onUpdate: ({ editor: instance, transaction }) => {
        refreshSearch(instance.state.doc);
        try {
          onChange(normalizeOwnership(instance.state.doc, sources), {
            before: normalizeOwnership(transaction.before, sources),
            mapping: transaction.mapping,
          });
        } catch (error) {
          onError(String(error));
        }
        revision++;
      },
      onSelectionUpdate: ({ editor: instance }) => {
        const { from, to } = instance.state.selection;
        onSelection(from, to, !programmaticSelection);
        revision++;
      },
    });
    instance.registerPlugin(
      new Plugin({
        key,
        filterTransaction: (tr) => {
          if (!tr.docChanged) return true;
          try {
            normalizeOwnership(tr.doc, sources);
            if (protectLocked && lockedProseChanged(tr.before, tr.doc, lockSources ?? sources)) {
              onError("Unlock this scene before suggesting changes to its prose.");
              return false;
            }
            return true;
          } catch {
            onError("To move this passage, cut and paste it at the destination.");
            return false;
          }
        },
        state: {
          init: () => DecorationSet.empty,
          apply: (tr, previous) => tr.getMeta(key) ?? previous.map(tr.mapping, tr.doc),
        },
        props: { decorations: (state) => key.getState(state) },
      })
    );
    editor = instance;
    searchQuery = initialSearch?.query ?? "";
    searchIndex = initialSearch?.index ?? 0;
    refreshSearch(instance.state.doc);
    const container = element.parentElement!;
    let readingTimer: ReturnType<typeof setTimeout>;
    const recordReading = () => {
      clearTimeout(readingTimer);
      readingTimer = setTimeout(() => {
        const bounds = container.getBoundingClientRect();
        const proseBounds = instance.view.dom.getBoundingClientRect();
        const toolbar =
          container.querySelector(".editorial-format")?.getBoundingClientRect().height ?? 0;
        const point = instance.view.posAtCoords({
          left: proseBounds.left + 8,
          top: Math.max(proseBounds.top + 2, bounds.top + toolbar + 20),
        });
        if (point) onReadingPosition(point.pos);
      }, 200);
    };
    container.addEventListener("scroll", recordReading, { passive: true });
    return () => {
      clearTimeout(readingTimer);
      container.removeEventListener("scroll", recordReading);
      instance.destroy();
    };
  });

  $effect(() => {
    const currentRevision = revision;
    if (!editor) return;
    const doc = sources.length ? normalizeOwnership(editor.state.doc, sources) : editor.state.doc;
    const decorations: Decoration[] = [];
    if (searchMatch && searchMatch.to <= doc.content.size)
      decorations.push(
        Decoration.inline(searchMatch.from, searchMatch.to, { class: "editorial-search-match" })
      );
    const seenScenes = new SvelteSet<string>();
    let chapter = "";
    if (showSections)
      doc.forEach((node, offset) => {
        const source = sources.find((s) => s.id === node.attrs.source);
        if (!source || seenScenes.has(source.scene_id)) return;
        seenScenes.add(source.scene_id);
        const chapterTitle = source.chapter_id !== chapter ? source.chapter : "";
        chapter = source.chapter_id;
        decorations.push(
          Decoration.widget(
            offset,
            () => {
              const header = document.createElement("div");
              header.className = "editorial-section";
              header.contentEditable = "false";
              if (chapterTitle) {
                const title = document.createElement("h2");
                title.textContent = chapterTitle;
                header.append(title);
              }
              const scene = document.createElement("p");
              scene.textContent = source.scene;
              header.append(scene);
              return header;
            },
            { side: -1, key: source.scene_id }
          )
        );
      });
    // A transient decoration index; never retained as reactive state.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const markers = new Map<number, EditorialChange[]>();
    for (const change of changes.filter(
      (c) => c.state === "open" && (!c.writer_decision || c.writer_decision === "open")
    )) {
      const preview = readonly || change.unapplied;
      const range = projectedRange(base, doc, change);
      const from = Math.max(0, Math.min(range.from, doc.content.size));
      const to = Math.max(from, Math.min(range.to, doc.content.size));
      const resolved = doc.resolve(from);
      const at = resolved.depth ? resolved.start() : from;
      markers.set(at, [...(markers.get(at) ?? []), change]);
      if (!markup && change.id !== selected) continue;
      if (from < to)
        decorations.push(
          Decoration.inline(from, to, {
            class:
              change.id === selected
                ? "editorial-selected"
                : change.kind === "comment"
                  ? "editorial-comment"
                  : preview
                    ? "editorial-deletion"
                    : "editorial-insertion",
            "data-review-id": change.id,
          })
        );
      if (change.kind === "suggestion") {
        const slice = Slice.fromJSON(editorialSchema, preview ? change.after : change.before);
        if (slice.size)
          decorations.push(
            Decoration.widget(
              preview ? to : from,
              () => {
                const node = document.createElement(preview ? "ins" : "del");
                node.className = preview ? "editorial-insertion" : "editorial-deletion";
                node.dataset.reviewId = change.id;
                node.append(
                  DOMSerializer.fromSchema(editorialSchema).serializeFragment(slice.content)
                );
                return node;
              },
              {
                side: preview ? 1 : -1,
                key: `change:${change.id}:${change.revision}:${preview ? "proposed" : "original"}`,
              }
            )
          );
      }
    }
    for (const [position, annotations] of markers)
      decorations.push(
        Decoration.widget(
          position,
          () => {
            const marker = document.createElement("button");
            marker.className = "editorial-margin-marker";
            marker.type = "button";
            marker.contentEditable = "false";
            marker.textContent = String(annotations.length);
            marker.title = `${annotations.length} comments or changes in this paragraph`;
            marker.setAttribute("aria-label", marker.title);
            marker.setAttribute("aria-pressed", String(annotations.some((c) => c.id === selected)));
            marker.onmousedown = (event) => event.preventDefault();
            marker.onclick = (event) => {
              event.preventDefault();
              event.stopPropagation();
              const index = annotations.findIndex((c) => c.id === selected);
              onAnnotation(annotations[(index + 1) % annotations.length].id);
            };
            return marker;
          },
          {
            side: -1,
            key: `margin:${JSON.stringify(annotations.map((c) => c.id))}:${annotations.some((c) => c.id === selected) ? selected : ""}`,
          }
        )
      );
    editor.view.dispatch(
      editor.state.tr
        .setMeta(key, DecorationSet.create(editor.state.doc, decorations))
        .setMeta("editorialRevision", currentRevision)
    );
  });
</script>

<div class="editorial-format">
  <ProseToolbar {editor} {revision} {readonly}>
    {@render toolbar?.()}
    {#if canComment}<button class="comment-action" onclick={onComment}
        ><MessageSquare size={16} /> Comment</button
      >{/if}
  </ProseToolbar>
</div>
<div class="app-prose-sheet editorial-sheet" bind:this={element}></div>

<style>
  .editorial-format {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky);
  }
  .comment-action {
    display: flex;
    align-items: center;
    gap: var(--space-2xs);
    background: transparent;
    border: 0;
    color: var(--color-text);
    padding: var(--space-2xs);
    font-family: var(--font-ui);
    font-size: var(--text-small);
  }
  :global(.editorial-margin-marker) {
    position: absolute;
    transform: translateX(calc(-1 * var(--space-l)));
    padding: var(--space-3xs);
    min-width: var(--space-m);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-s);
    font-family: var(--font-ui);
    font-size: var(--text-eyebrow);
    color: var(--color-prose-text);
    background: var(--color-accent-wash);
    cursor: pointer;
  }
  :global(.editorial-margin-marker[aria-pressed="true"]) {
    border-color: var(--color-accent);
  }
  .editorial-sheet {
    margin: var(--space-l) auto;
    width: 100%;
    max-width: calc(var(--measure) + var(--space-xl) * 2);
    box-sizing: border-box;
    padding: var(--space-xl);
  }
  :global(.editorial-prose) {
    font-family: var(--font-body);
    font-size: var(--text-body);
    line-height: var(--leading-relaxed);
    color: var(--color-prose-text);
    max-width: var(--measure);
    min-height: 60vh;
    outline: none;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }
  :global(.editorial-prose p) {
    margin: 0;
    text-indent: 1.5em;
  }
  :global(.editorial-prose p:first-child),
  :global(.editorial-section + p),
  :global(.editorial-section p) {
    text-indent: 0;
  }
  :global(.editorial-prose blockquote) {
    margin-block: 1em;
    padding: var(--space-s);
    background: var(--color-prose-callout-bg);
    border-left: 4px solid var(--color-prose-blockquote-border);
    border-radius: 0 var(--radius-m) var(--radius-m) 0;
    font-style: italic;
    color: var(--color-prose-blockquote-text);
  }
  :global(.editorial-search-match) {
    background: var(--color-accent-wash);
    outline: 1px solid var(--color-accent);
  }
  :global(.editorial-section) {
    padding-block: var(--space-l) var(--space-s);
    border-bottom: 1px solid var(--color-border);
    white-space: normal;
  }
  :global(.editorial-section h2) {
    font-family: var(--font-display);
    font-size: var(--text-h2);
  }
  :global(.editorial-section p) {
    font-family: var(--font-ui);
    font-size: var(--text-small);
    color: var(--color-prose-placeholder);
  }
  :global(.editorial-insertion) {
    color: var(--color-prose-text);
    text-decoration: underline;
  }
  :global(.editorial-deletion) {
    color: var(--color-prose-text);
    text-decoration: line-through;
    white-space: pre-wrap;
  }
  :global(.editorial-deletion p) {
    display: inline;
  }
  :global(.editorial-comment),
  :global(.editorial-selected) {
    background: var(--color-accent-wash);
    border-bottom: 1px solid var(--color-accent);
  }
  :global(.editorial-selected) {
    outline: 1px solid var(--color-accent);
  }
</style>
