<script lang="ts">
  import { onMount } from "svelte";
  import { Editor } from "@tiptap/core";
  import { Plugin, PluginKey, TextSelection } from "@tiptap/pm/state";
  import { Decoration, DecorationSet, type EditorView } from "@tiptap/pm/view";
  import {
    reviewExtensions,
    isAnchored,
    type Annotation,
    type ReviewDocument,
  } from "../utils/revisions";
  let {
    source,
    annotations,
    selectedId,
    navigationVersion = 0,
    onSelect,
  }: {
    source: ReviewDocument;
    annotations: Annotation[];
    selectedId: string | null;
    navigationVersion?: number;
    onSelect: (from: number, to: number, quote: string, explicit: boolean) => void;
  } = $props();
  let element: HTMLDivElement;
  let editor = $state.raw<Editor>();
  let appliedHtml = "";
  let programmaticSelection = false;
  let navigatedVersion = -1;
  let navigatedId: string | null = null;
  function captureDomSelection(view: EditorView) {
    const selection = window.getSelection();
    if (
      selection?.anchorNode &&
      selection.focusNode &&
      view.dom.contains(selection.anchorNode) &&
      view.dom.contains(selection.focusNode)
    ) {
      const anchor = view.posAtDOM(selection.anchorNode, selection.anchorOffset);
      const head = view.posAtDOM(selection.focusNode, selection.focusOffset);
      const { from, to } = TextSelection.between(
        view.state.doc.resolve(anchor),
        view.state.doc.resolve(head)
      );
      onSelect(from, to, view.state.doc.textBetween(from, to, "\n"), true);
    }
    return false;
  }
  const key = new PluginKey("editorial-annotations");
  onMount(() => {
    appliedHtml = source.html;
    const instance = new Editor({
      element,
      extensions: reviewExtensions,
      content: source.html,
      editable: false,
      editorProps: {
        handleDOMEvents: { mouseup: captureDomSelection, keyup: captureDomSelection },
        attributes: { class: "review-prose", "aria-label": "Prose for editorial review" },
      },
      onSelectionUpdate: ({ editor }) => {
        if (programmaticSelection) return;
        const { from, to } = editor.state.selection;
        onSelect(from, to, editor.state.doc.textBetween(from, to, "\n"), true);
      },
    });
    instance.registerPlugin(
      new Plugin({
        key,
        state: {
          init: () => DecorationSet.empty,
          apply: (tr, previous) => tr.getMeta(key) ?? previous.map(tr.mapping, tr.doc),
        },
        props: { decorations: (state) => key.getState(state) },
      })
    );
    editor = instance;
    onSelect(instance.state.selection.from, instance.state.selection.to, "", false);
    return () => instance.destroy();
  });
  $effect(() => {
    if (!editor) return;
    programmaticSelection = true;
    try {
      if (appliedHtml !== source.html) {
        editor.commands.setContent(source.html, { emitUpdate: false });
        appliedHtml = source.html;
      }
      const decorations: Decoration[] = [];
      for (const a of annotations) {
        if (a.state !== "open" || !isAnchored(a, [source], editor.state.doc)) continue;
        if (a.from < a.to)
          decorations.push(
            Decoration.inline(a.from, a.to, {
              class: a.replacement !== null ? "review-deletion" : "review-comment",
              ...(a.id === selectedId ? { "data-selected": "true" } : {}),
            })
          );
        if (a.replacement !== null)
          decorations.push(
            Decoration.widget(a.to, () => {
              const span = document.createElement("ins");
              span.className = "review-insertion";
              if (a.id === selectedId) span.dataset.selected = "true";
              span.textContent = a.replacement;
              return span;
            })
          );
      }
      editor.view.dispatch(
        editor.state.tr.setMeta(key, DecorationSet.create(editor.state.doc, decorations))
      );
      const selected = annotations.find((a) => a.id === selectedId);
      if (
        selected &&
        (selected.id !== navigatedId || navigationVersion !== navigatedVersion) &&
        isAnchored(selected, [source], editor.state.doc)
      ) {
        editor.commands.setTextSelection({ from: selected.from, to: selected.to });
        // A read-only editor may have no native selection when a navigation
        // button holds focus. Scroll the decoration directly in that case too.
        element
          .querySelector<HTMLElement>('[data-selected="true"]')
          ?.scrollIntoView({ block: "nearest" });
      }
      navigatedId = selectedId;
      navigatedVersion = navigationVersion;
    } finally {
      programmaticSelection = false;
    }
  });
</script>

<div class="app-prose-sheet p-6" bind:this={element}></div>

<style>
  div {
    max-width: var(--measure);
    width: 100%;
  }
  :global(.review-prose) {
    font-family: var(--font-body);
    font-size: var(--text-body);
    color: var(--color-text);
    white-space: pre-wrap;
    min-height: 12rem;
    outline: none;
  }
  :global(.review-deletion) {
    color: var(--color-error);
    text-decoration: line-through;
  }
  :global(.review-comment),
  :global(.review-prose [data-selected]) {
    background: var(--color-accent-wash);
    border-bottom: 1px solid var(--color-accent);
  }
  :global(.review-prose [data-selected]) {
    outline: 1px solid var(--color-accent);
  }
  :global(.review-insertion) {
    color: var(--color-success);
    text-decoration: underline;
  }
</style>
