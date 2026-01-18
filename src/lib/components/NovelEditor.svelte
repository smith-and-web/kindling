<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
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
    ChevronLeft,
    ChevronRight,
  } from "lucide-svelte";

  interface Props {
    content: string;
    placeholder?: string;
    readonly?: boolean;
    saving?: boolean;
    saveError?: boolean;
    onUpdate?: (html: string) => void;
  }

  let {
    content,
    placeholder = "Write your prose...",
    readonly = false,
    saving = false,
    saveError = false,
    onUpdate,
  }: Props = $props();

  let editorElement: HTMLElement;
  let pagesContainer: HTMLElement;
  let editor: Editor | null = $state(null);
  let isInitialized = false;

  // Pagination state
  let currentPage = $state(1);
  let totalPages = $state(1);

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

  // Page dimensions - must match CSS values
  // Column width: 28rem = 448px, Gap: 4rem = 64px
  const PAGE_WIDTH = 448; // 28rem in pixels
  const PAGE_GAP = 64; // 4rem in pixels
  const PAGE_STRIDE = PAGE_WIDTH + PAGE_GAP; // Total width per page including gap

  // Calculate total pages based on content width
  function updatePageCount() {
    if (!pagesContainer || !editorElement) return;
    const contentWidth = editorElement.scrollWidth;
    // First page is just PAGE_WIDTH, subsequent pages add PAGE_STRIDE
    if (contentWidth <= PAGE_WIDTH) {
      totalPages = 1;
    } else {
      totalPages = 1 + Math.ceil((contentWidth - PAGE_WIDTH) / PAGE_STRIDE);
    }
  }

  // Navigate to a specific page
  function goToPage(page: number) {
    if (page < 1 || page > totalPages) return;
    currentPage = page;
    if (pagesContainer) {
      // Scroll to the start of the requested page
      pagesContainer.scrollLeft = (page - 1) * PAGE_STRIDE;
    }
  }

  function previousPage() {
    goToPage(currentPage - 1);
  }

  function nextPage() {
    goToPage(currentPage + 1);
  }

  // Update page count when content changes
  async function onContentUpdate() {
    await tick();
    updatePageCount();
  }

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        StarterKit.configure({
          // Disable features we don't need for novel writing
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
        onContentUpdate();
        // Skip the initial update that fires when content is first set
        if (onUpdate && isInitialized) {
          onUpdate(editor.getHTML());
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
    // Mark as initialized after a tick to allow initial content to settle
    setTimeout(() => {
      isInitialized = true;
      updatePageCount();
    }, 0);
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });

  // Update content when prop changes (e.g., switching beats)
  $effect(() => {
    if (editor && content !== editor.getHTML()) {
      editor.commands.setContent(content || "");
    }
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
    // Insert a tab character for indentation
    editor?.chain().focus().insertContent("\t").run();
  }

  function indent() {
    insertTab();
  }

  function outdent() {
    // For outdent, we'd need to remove leading whitespace - this is complex
    // For now, this is a no-op since we're using inline tabs
  }

  function handleKeydown(e: KeyboardEvent) {
    // Capture Tab key to insert indentation instead of moving focus
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      insertTab();
    }
    // Page navigation with Ctrl/Cmd + Arrow keys
    if ((e.ctrlKey || e.metaKey) && e.key === "ArrowLeft") {
      e.preventDefault();
      previousPage();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "ArrowRight") {
      e.preventDefault();
      nextPage();
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

      <!-- Save status indicator -->
      <div class="toolbar-spacer"></div>
      {#if saving}
        <div class="save-indicator">
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>Saving...</span>
        </div>
      {:else if saveError}
        <div class="save-indicator error">
          <span>Error saving</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Novel Page with Pagination -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="novel-page-wrapper" onkeydown={handleKeydown}>
    <div class="novel-page">
      <div bind:this={pagesContainer} class="pages-container">
        <div bind:this={editorElement} class="editor-wrapper"></div>
      </div>
    </div>

    <!-- Page Navigation -->
    <div class="page-navigation">
      <button
        type="button"
        class="page-nav-btn"
        onclick={previousPage}
        disabled={currentPage <= 1}
        aria-label="Previous page"
      >
        <ChevronLeft class="w-4 h-4" />
      </button>
      <span class="page-indicator">
        Page {currentPage} of {totalPages}
      </span>
      <button
        type="button"
        class="page-nav-btn"
        onclick={nextPage}
        disabled={currentPage >= totalPages}
        aria-label="Next page"
      >
        <ChevronRight class="w-4 h-4" />
      </button>
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

  .save-indicator {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .save-indicator.error {
    color: #ef4444;
  }

  /* Novel Page Wrapper */
  .novel-page-wrapper {
    flex: 1;
    overflow: hidden;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    background: var(--color-bg-panel);
  }

  /* Novel Page - mimics a trade paperback (5.5" x 8.5") */
  .novel-page {
    width: 100%;
    max-width: 32rem; /* ~512px, approximately 5.5" at 96dpi */
    height: 45rem; /* Fixed height for pagination - content area */
    background: #faf9f7; /* Warm white paper color */
    border-radius: 0.125rem;
    box-shadow:
      0 1px 3px rgba(0, 0, 0, 0.2),
      0 4px 12px rgba(0, 0, 0, 0.15),
      inset 0 0 0 1px rgba(0, 0, 0, 0.05);
    padding: 2.5rem 2rem;
    overflow: hidden;
  }

  /* Pages container with CSS columns for pagination */
  .pages-container {
    height: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-behavior: smooth;
    scroll-snap-type: x mandatory;
    /* Hide scrollbar but allow scrolling */
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .pages-container::-webkit-scrollbar {
    display: none;
  }

  .editor-wrapper {
    height: 100%;
    column-width: 28rem; /* Page content width (32rem - 4rem padding) */
    column-gap: 4rem;
    column-fill: auto;
  }

  /* Page Navigation */
  .page-navigation {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: var(--color-bg-card);
    border-radius: 0.5rem;
  }

  .page-nav-btn {
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

  .page-nav-btn:hover:not(:disabled) {
    background: var(--color-bg-panel);
    color: var(--color-text-primary);
  }

  .page-nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .page-indicator {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    min-width: 6rem;
    text-align: center;
  }

  /* TipTap Editor Styles - Novel typography */
  :global(.novel-editor-content) {
    outline: none;
    font-family: "Lora", Georgia, serif;
    font-size: 0.8125rem; /* 13px - typical novel font size */
    line-height: 1.6;
    color: #1a1a1a;
    height: 100%;
    tab-size: 4;
    white-space: pre-wrap;
  }

  :global(.novel-editor-content p) {
    margin: 0;
    text-indent: 1.5em;
  }

  :global(.novel-editor-content p:first-child) {
    text-indent: 0;
  }

  :global(.novel-editor-content p + p) {
    margin-top: 0;
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

  /* Text alignment */
  :global(.novel-editor-content p[style*="text-align: center"]) {
    text-indent: 0;
  }

  :global(.novel-editor-content p[style*="text-align: right"]) {
    text-indent: 0;
  }

  :global(.novel-editor-content p[style*="text-align: justify"]) {
    text-align: justify;
  }
</style>
