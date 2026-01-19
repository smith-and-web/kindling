<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Underline from "@tiptap/extension-underline";
  import TextAlign from "@tiptap/extension-text-align";
  import {
    Bold,
    Italic,
    Underline as UnderlineIcon,
    Code,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    Quote,
    IndentIncrease,
    IndentDecrease,
    Loader2,
  } from "lucide-svelte";

  interface Props {
    content: string;
    placeholder?: string;
    readonly?: boolean;
    saveStatus?: "idle" | "saving" | "error";
    onUpdate?: (html: string) => void;
  }

  let {
    content,
    placeholder = "Write your prose...",
    readonly = false,
    saveStatus = "idle",
    onUpdate,
  }: Props = $props();

  let editorElement: HTMLElement;
  let editor: Editor | null = $state(null);
  let isInitialized = false;
  let isSettingContent = false;
  let lastExternalContent = "";
  let lastEmittedContent = "";

  // Word count
  let wordCount = $state(0);

  // Track editor state for toolbar reactivity
  let isBold = $state(false);
  let isItalic = $state(false);
  let isUnderline = $state(false);
  let isCode = $state(false);
  let isBlockquote = $state(false);
  let textAlign = $state<"left" | "center" | "right" | "justify">("left");

  function updateToolbarState() {
    if (!editor) return;
    isBold = editor.isActive("bold");
    isItalic = editor.isActive("italic");
    isUnderline = editor.isActive("underline");
    isCode = editor.isActive("code");
    isBlockquote = editor.isActive("blockquote");
    textAlign = editor.isActive({ textAlign: "center" })
      ? "center"
      : editor.isActive({ textAlign: "right" })
        ? "right"
        : editor.isActive({ textAlign: "justify" })
          ? "justify"
          : "left";
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
  function toggleBold() {
    editor?.chain().focus().toggleBold().run();
  }

  function toggleItalic() {
    editor?.chain().focus().toggleItalic().run();
  }

  function toggleUnderline() {
    editor?.chain().focus().toggleUnderline().run();
  }

  function toggleCode() {
    editor?.chain().focus().toggleCode().run();
  }

  function toggleBlockquote() {
    editor?.chain().focus().toggleBlockquote().run();
  }

  function setAlignment(align: "left" | "center" | "right" | "justify") {
    editor?.chain().focus().setTextAlign(align).run();
  }

  function insertTab() {
    editor?.chain().focus().insertContent("\t").run();
  }

  function indent() {
    insertTab();
  }

  function outdent() {
    // No-op for now
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      insertTab();
    }
  }
</script>

<div class="novel-editor" class:readonly>
  <!-- Toolbar -->
  {#if !readonly}
    <div class="novel-editor-toolbar">
      <div class="toolbar-group">
        <button
          type="button"
          class="toolbar-btn"
          class:active={isBold}
          onclick={toggleBold}
          title="Bold (Ctrl+B)"
        >
          <Bold class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={isItalic}
          onclick={toggleItalic}
          title="Italic (Ctrl+I)"
        >
          <Italic class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={isUnderline}
          onclick={toggleUnderline}
          title="Underline (Ctrl+U)"
        >
          <UnderlineIcon class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={isCode}
          onclick={toggleCode}
          title="Monospace"
        >
          <Code class="w-4 h-4" />
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-group">
        <button
          type="button"
          class="toolbar-btn"
          class:active={textAlign === "left"}
          onclick={() => setAlignment("left")}
          title="Align Left"
        >
          <AlignLeft class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={textAlign === "center"}
          onclick={() => setAlignment("center")}
          title="Align Center"
        >
          <AlignCenter class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={textAlign === "right"}
          onclick={() => setAlignment("right")}
          title="Align Right"
        >
          <AlignRight class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="toolbar-btn"
          class:active={textAlign === "justify"}
          onclick={() => setAlignment("justify")}
          title="Justify"
        >
          <AlignJustify class="w-4 h-4" />
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-group">
        <button
          type="button"
          class="toolbar-btn"
          class:active={isBlockquote}
          onclick={toggleBlockquote}
          title="Block Quote"
        >
          <Quote class="w-4 h-4" />
        </button>
        <button type="button" class="toolbar-btn" onclick={indent} title="Indent">
          <IndentIncrease class="w-4 h-4" />
        </button>
        <button type="button" class="toolbar-btn" onclick={outdent} title="Outdent">
          <IndentDecrease class="w-4 h-4" />
        </button>
      </div>

      <!-- Save status and word count -->
      <div class="toolbar-spacer"></div>
      {#if saveStatus === "saving"}
        <div class="save-status saving">
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
    </div>
  {/if}

  <!-- Editor -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="novel-pages-container" onkeydown={handleKeydown}>
    <div class="novel-page">
      <div bind:this={editorElement} class="editor-wrapper" data-testid="beat-prose-editor"></div>
    </div>
  </div>
</div>

<style>
  .novel-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-panel);
    overflow: hidden;
  }

  .novel-editor.readonly {
    opacity: 0.7;
  }

  /* Toolbar */
  .novel-editor-toolbar {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.5rem 0.75rem;
    background: var(--color-bg-card);
    border-bottom: 1px solid var(--color-bg-primary);
    flex-wrap: wrap;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 0.125rem;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 0.375rem;
    color: var(--color-text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .toolbar-btn:hover {
    background: var(--color-bg-panel);
    color: var(--color-text-primary);
  }

  .toolbar-btn.active {
    background: var(--color-accent);
    color: white;
  }

  .toolbar-divider {
    width: 1px;
    height: 1.5rem;
    background: var(--color-bg-panel);
    margin: 0 0.5rem;
  }

  .toolbar-spacer {
    flex: 1;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    margin-right: 0.5rem;
  }

  .save-status.saving {
    color: var(--color-text-secondary);
    opacity: 0.7;
  }

  .save-status.error {
    color: #ef4444;
  }

  .word-count {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    padding: 0.25rem 0.5rem;
    background: var(--color-bg-panel);
    border-radius: 0.25rem;
  }

  /* Pages container - scrollable area */
  .novel-pages-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 1.5rem;
    background: var(--color-bg-panel);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  /* Novel Page - the paper appearance */
  .novel-page {
    width: 26rem;
    min-height: 40rem;
    background: #faf9f7;
    border-radius: 0.125rem;
    box-shadow:
      0 1px 3px rgba(0, 0, 0, 0.2),
      0 4px 12px rgba(0, 0, 0, 0.15),
      inset 0 0 0 1px rgba(0, 0, 0, 0.05);
    padding: 2rem 1.5rem;
    flex-shrink: 0;
  }

  .editor-wrapper {
    width: 100%;
    min-height: 36rem;
  }

  /* TipTap Editor Styles - Novel typography */
  :global(.novel-editor-content) {
    outline: none;
    font-family: "Lora", Georgia, serif;
    font-size: 0.8125rem;
    line-height: 1.6;
    color: #1a1a1a;
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
    color: #9ca3af;
    pointer-events: none;
    height: 0;
    font-style: italic;
  }

  :global(.novel-editor-content blockquote) {
    margin: 1em 0;
    padding-left: 1em;
    border-left: 2px solid #d1d5db;
    font-style: italic;
    color: #4b5563;
  }

  :global(.novel-editor-content code) {
    font-family: "Courier New", Courier, monospace;
    background: #f3f4f6;
    padding: 0.125em 0.25em;
    border-radius: 0.25em;
    font-size: 0.9em;
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
