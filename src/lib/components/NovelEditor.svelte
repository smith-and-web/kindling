<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Underline from "@tiptap/extension-underline";
  import TextAlign from "@tiptap/extension-text-align";
  import { trackEditorPosition } from "../utils/editorPosition";
  import { Loader2 } from "lucide-svelte";
  import ProseToolbar from "./ProseToolbar.svelte";

  interface Props {
    content: string;
    placeholder?: string;
    readonly?: boolean;
    saveStatus?: "idle" | "saving" | "error";
    onUpdate?: (html: string) => void;
    /** Called when editor is ready; pass to parent for split-at-cursor support */
    onEditorReady?: (editor: Editor) => void;
    projectId?: string;
    sceneId?: string;
    beatId?: string | null;
  }

  let {
    content,
    placeholder = "Write your prose...",
    readonly = false,
    saveStatus = "idle",
    onUpdate,
    onEditorReady,
    projectId,
    sceneId,
    beatId = null,
  }: Props = $props();

  let editorElement: HTMLElement;
  let scrollElement: HTMLDivElement;
  let editor: Editor | null = $state(null);
  let isInitialized = false;
  let isSettingContent = false;
  let lastExternalContent = "";
  let lastEmittedContent = "";

  $effect(() => {
    const instance = editor;
    const project = projectId;
    const scene = sceneId;
    const beat = beatId;
    if (instance && project && scene) {
      return untrack(() => trackEditorPosition(instance, scrollElement, project, scene, beat));
    }
  });

  // Word count
  let wordCount = $state(0);

  let toolbarRevision = $state(0);
  function updateToolbarState() {
    untrack(() => toolbarRevision++);
  }

  function updateWordCount() {
    if (!editor) return;
    const text = editor.getText();
    // Count words by splitting on whitespace and filtering empty strings
    const words = text
      .trim()
      .split(/\s+/)
      .filter((word) => word.length > 0);
    wordCount = words.length;
  }

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        StarterKit.configure({
          heading: false,
          bulletList: false,
          orderedList: false,
          listItem: false,
          codeBlock: false,
          horizontalRule: false,
        }),
        Underline,
        TextAlign.configure({
          types: ["paragraph"],
        }),
      ],
      content: content || "",
      editable: !readonly,
      editorProps: {
        attributes: {
          class: "novel-editor-content",
          "data-placeholder": placeholder,
        },
      },
      onUpdate: ({ editor }) => {
        updateToolbarState();
        updateWordCount();
        if (onUpdate && isInitialized && !isSettingContent) {
          const html = editor.getHTML();
          if (html !== lastEmittedContent) {
            lastEmittedContent = html;
            onUpdate(html);
          }
        }
      },
      onSelectionUpdate: () => {
        updateToolbarState();
      },
      onFocus: () => {
        updateToolbarState();
      },
    });

    updateToolbarState();
    updateWordCount();
    lastExternalContent = content || "";
    lastEmittedContent = content || "";

    onEditorReady?.(editor);

    setTimeout(() => {
      isInitialized = true;
    }, 0);
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });

  // Update content when prop changes
  $effect.pre(() => {
    const normalizedContent = content || "";

    untrack(() => {
      if (editor) {
        if (normalizedContent !== lastExternalContent && normalizedContent !== lastEmittedContent) {
          lastExternalContent = normalizedContent;
          lastEmittedContent = normalizedContent;
          isSettingContent = true;
          editor.commands.setContent(normalizedContent);
          isSettingContent = false;
          updateWordCount();
        } else {
          lastExternalContent = normalizedContent;
        }
      }
    });
  });

  // Update editable state when readonly prop changes
  $effect(() => {
    if (editor) {
      editor.setEditable(!readonly);
    }
  });

  // Toolbar actions
  function insertTab() {
    editor?.chain().focus().insertContent("\t").run();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      insertTab();
    }
  }

  /** Get the 0-based paragraph index for split_before_paragraph (which paragraph the cursor is in) */
  export function getSplitBeforeParagraph(): number | null {
    if (!editor) return null;
    const { from } = editor.state.selection;
    const resolved = editor.state.doc.resolve(from);
    if (resolved.depth < 1) return 0;
    return resolved.index(0) as number;
  }
</script>

<div class="novel-editor" class:readonly>
  <!-- Toolbar -->
  {#if !readonly}
    <ProseToolbar {editor} revision={toolbarRevision}>
      {#if saveStatus === "saving"}
        <div class="save-status saving" data-testid="save-indicator">
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>Saving...</span>
        </div>
      {:else if saveStatus === "error"}
        <div class="save-status error">
          <span>Error saving</span>
        </div>
      {/if}
      <div class="word-count">
        {wordCount}
        {wordCount === 1 ? "word" : "words"}
      </div>
    </ProseToolbar>
  {/if}

  <!-- Editor -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div bind:this={scrollElement} class="novel-pages-container" onkeydown={handleKeydown}>
    <div class="novel-page app-prose-sheet">
      <div
        bind:this={editorElement}
        class="editor-wrapper"
        data-testid="beat-prose-editor"
        data-source-id={beatId ?? sceneId}
      ></div>
    </div>
  </div>
</div>

<style>
  .novel-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    overflow: hidden;
  }

  .novel-editor.readonly {
    cursor: default;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: var(--text-eyebrow);
    padding: 0.25rem 0.5rem;
    margin-right: 0.5rem;
  }

  .save-status.saving {
    color: var(--color-text-muted);
  }

  .save-status.error {
    color: var(--color-error);
  }

  .word-count {
    font-size: var(--text-eyebrow);
    color: var(--color-text-muted);
    padding: 0.25rem 0.5rem;
    background: var(--color-surface);
    border-radius: var(--radius-xs);
  }

  /* Pages container - scrollable area */
  .novel-pages-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 1.5rem;
    background: var(--color-surface-sunken);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  /* Novel Page - the paper appearance */
  .novel-page {
    width: min(calc(100% - var(--space-l)), calc(var(--measure) + 2 * var(--space-xl)));
    min-height: 40rem;
    padding: var(--space-xl);
    flex-shrink: 0;
  }

  .editor-wrapper {
    width: 100%;
    max-width: var(--measure);
    min-height: 36rem;
    margin: 0 auto;
  }

  /* TipTap Editor Styles - Novel typography */
  :global(.novel-editor-content) {
    outline: none;
    font-family: var(--font-body);
    font-size: var(--text-body);
    line-height: var(--leading-relaxed);
    color: var(--color-prose-text);
    min-height: 36rem;
    tab-size: 4;
    white-space: pre-wrap;
    cursor: text;
  }

  /* Paragraph styles */
  :global(.novel-editor-content p) {
    margin: 0;
    text-indent: 1.5em;
  }

  :global(.novel-editor-content p:first-child),
  :global(.novel-editor-content p.is-editor-empty:first-child) {
    text-indent: 0;
  }

  :global(.novel-editor-content p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    float: left;
    color: var(--color-prose-placeholder);
    pointer-events: none;
    height: 0;
    font-style: italic;
  }

  :global(.novel-editor-content blockquote) {
    margin: 1em 0;
    padding: var(--space-s);
    background: var(--color-prose-callout-bg);
    border-left: 4px solid var(--color-prose-blockquote-border);
    border-radius: 0 var(--radius-m) var(--radius-m) 0;
    font-style: italic;
    color: var(--color-prose-blockquote-text);
  }

  :global(.novel-editor-content code) {
    font-family: var(--font-mono);
    background: var(--color-prose-code-bg);
    padding: 0.125em 0.25em;
    border-radius: var(--radius-xs);
    font-size: var(--text-ui);
  }

  :global(.novel-editor-content strong) {
    font-weight: 600;
  }

  :global(.novel-editor-content em) {
    font-style: italic;
  }

  :global(.novel-editor-content u) {
    text-decoration: underline;
  }
</style>
