<script lang="ts">
  import { onMount } from "svelte";
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
    changesBetween,
    projectedRange,
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
    onReadingPosition,
  }: {
    sources: EditorialSource[];
    initial: ReturnType<Node["toJSON"]>;
    changes?: EditorialChange[];
    readonly?: boolean;
    markup?: boolean;
    selected?: string | null;
    onChange: (doc: Node, edit: { before: Node; mapping: Mapping }) => void;
    onSelection: (from: number, to: number, explicit: boolean) => void;
    onComment: () => void;
    onError: (message: string) => void;
    onAnnotation: (id: string) => void;
    onReadingPosition: (position: number) => void;
  } = $props();
  let element: HTMLDivElement;
  let editor = $state.raw<Editor>();
  let revision = $state(0);
  let programmaticSelection = false;
  const format = $derived.by(() => {
    void revision;
    return {
      bold: editor?.isActive("bold") ?? false,
      italic: editor?.isActive("italic") ?? false,
      underline: editor?.isActive("underline") ?? false,
      blockquote: editor?.isActive("blockquote") ?? false,
    };
  });
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
  export function select(from: number, to = from, focus = true) {
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
    const results = searchManuscript(editor.state.doc, query);
    const result = results[((index % results.length) + results.length) % results.length];
    if (result) select(result.from, result.to, false);
    return results.length;
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
    select(view.from, view.to, view.focused);
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
          if (id) onAnnotation(id);
          return false;
        },
        attributes: {
          class: "editorial-prose",
          "aria-label": readonly ? "Current manuscript" : "Manuscript with suggested edits",
          spellcheck: "true",
        },
        handleKeyDown: (_view, event) => {
          if ((event.metaKey || event.ctrlKey) && event.altKey && event.key.toLowerCase() === "m") {
            event.preventDefault();
            onComment();
            return true;
          }
          return false;
        },
      },
      onUpdate: ({ editor: instance, transaction }) => {
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
    const seenScenes = new SvelteSet<string>();
    let chapter = "";
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
    if (markup) {
      for (const delta of changesBetween(base, doc)) {
        if (delta.fromB < delta.toB)
          decorations.push(
            Decoration.inline(delta.fromB, delta.toB, { class: "editorial-insertion" })
          );
        if (delta.fromA < delta.toA)
          decorations.push(
            Decoration.widget(
              delta.fromB,
              () => {
                const deleted = document.createElement("del");
                deleted.className = "editorial-deletion";
                deleted.append(
                  DOMSerializer.fromSchema(editorialSchema).serializeFragment(
                    base.slice(delta.fromA, delta.toA).content
                  )
                );
                return deleted;
              },
              { side: -1 }
            )
          );
      }
      for (const change of changes.filter((c) => c.state === "open")) {
        const range = projectedRange(base, doc, change);
        if (range.from < range.to)
          decorations.push(
            Decoration.inline(range.from, range.to, {
              class:
                change.id === selected
                  ? "editorial-selected"
                  : change.kind === "comment"
                    ? "editorial-comment"
                    : readonly
                      ? "editorial-deletion"
                      : "editorial-insertion",
              "data-review-id": change.id,
            })
          );
        if (readonly && change.kind === "suggestion") {
          decorations.push(
            Decoration.widget(
              range.to,
              () => {
                const inserted = document.createElement("ins");
                inserted.className = "editorial-insertion";
                inserted.dataset.reviewId = change.id;
                inserted.append(
                  DOMSerializer.fromSchema(editorialSchema).serializeFragment(
                    Slice.fromJSON(editorialSchema, change.after).content
                  )
                );
                return inserted;
              },
              { side: 1 }
            )
          );
        }
      }
    }
    editor.view.dispatch(
      editor.state.tr
        .setMeta(key, DecorationSet.create(editor.state.doc, decorations))
        .setMeta("editorialRevision", currentRevision)
    );
  });
</script>

{#if !readonly}
  <div class="editorial-format" role="toolbar" aria-label="Suggest formatting">
    <span>Suggesting</span>
    <button
      aria-label="Suggest bold"
      aria-pressed={format.bold}
      onmousedown={(e) => e.preventDefault()}
      onclick={() => editor?.chain().focus().toggleBold().run()}><strong>B</strong></button
    >
    <button
      aria-label="Suggest italic"
      aria-pressed={format.italic}
      onmousedown={(e) => e.preventDefault()}
      onclick={() => editor?.chain().focus().toggleItalic().run()}><em>I</em></button
    >
    <button
      aria-label="Suggest underline"
      aria-pressed={format.underline}
      onmousedown={(e) => e.preventDefault()}
      onclick={() => editor?.chain().focus().toggleUnderline().run()}>U̲</button
    >
    <button
      aria-label="Suggest blockquote"
      aria-pressed={format.blockquote}
      onmousedown={(e) => e.preventDefault()}
      onclick={() => editor?.chain().focus().toggleBlockquote().run()}>Quote</button
    >
    <select
      aria-label="Suggest paragraph alignment"
      onchange={(e) => editor?.chain().focus().setTextAlign(e.currentTarget.value).run()}
    >
      <option value="left">Align left</option><option value="center">Center</option><option
        value="right">Align right</option
      ><option value="justify">Justify</option>
    </select>
    <button aria-label="Undo suggested edit" onclick={() => editor?.chain().focus().undo().run()}
      >Undo</button
    >
    <button aria-label="Redo suggested edit" onclick={() => editor?.chain().focus().redo().run()}
      >Redo</button
    >
    <button onclick={onComment}>Comment <kbd>⌘/Ctrl Alt M</kbd></button>
  </div>
{/if}
<div class="app-prose-sheet editorial-sheet" bind:this={element}></div>

<style>
  .editorial-format {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3xs);
    padding-block: var(--space-2xs);
    border-bottom: 1px solid var(--color-border);
    position: sticky;
    top: 0;
    background: var(--color-bg);
    z-index: var(--z-sticky);
  }
  .editorial-format span,
  kbd {
    font-family: var(--font-ui);
    font-size: var(--text-small);
    color: var(--color-text-muted);
  }
  .editorial-format button {
    padding: var(--space-2xs);
  }
  .editorial-format select {
    width: auto;
    max-width: 9rem;
    padding: var(--space-2xs);
    font-size: var(--text-small);
  }
  .editorial-sheet {
    margin-block: var(--space-l);
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
    margin-block: var(--space-m);
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
