<script lang="ts">
  import { KeyboardFormatting } from "../utils/keyboardFormatting";
  import { onMount, onDestroy, untrack } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Underline from "@tiptap/extension-underline";
  import TextAlign from "@tiptap/extension-text-align";
  import { trackEditorPosition } from "../utils/editorPosition";
  import { Loader2 } from "lucide-svelte";
  import ProseToolbar from "./ProseToolbar.svelte";
  import { ScreenplayFormatting } from "../utils/screenplayFormatting";
  import { currentProject } from "../stores/project.svelte";
  import { t } from "../i18n";

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
    placeholder = t("editor.novelEditor.placeholder"),
    readonly = false,
    saveStatus = "idle",
    onUpdate,
    onEditorReady,
    projectId,
    sceneId,
    beatId = null,
  }: Props = $props();

  // Screenplays lay out cues, dialogue and transitions on the page (display only).
  const screenplay = currentProject.value?.project_type === "screenplay";

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

  let toolbarRevision = $state(0);
  function updateToolbarState() {
    untrack(() => toolbarRevision++);
  }

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        KeyboardFormatting,
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
        ...(screenplay ? [ScreenplayFormatting] : []),
      ],
      content: content || "",
      editable: !readonly,
      editorProps: {
        // Re-read on every view update, so the name and read-only state follow
        // the props (a scene locking, a beat editor becoming the page).
        attributes: () => ({
          class: "novel-editor-content",
          "data-placeholder": placeholder,
          role: "textbox",
          "aria-multiline": "true",
          "aria-label": beatId
            ? t("editor.novelEditor.beatProse")
            : t("editor.novelEditor.sceneProse"),
          "aria-readonly": readonly ? "true" : "false",
        }),
      },
      onUpdate: ({ editor }) => {
        updateToolbarState();
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
        } else {
          lastExternalContent = normalizedContent;
        }
      }
    });
  });

  // Update editable state when readonly prop changes
  $effect(() => {
    if (editor) {
      editor.setEditable(!readonly, false);
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

<div class="novel-editor" class:readonly class:beat-mode={!!beatId} class:screenplay>
  <!-- Toolbar -->
  {#if !readonly}
    <ProseToolbar {editor} revision={toolbarRevision}>
      <!-- A failed save is a lasting state the writer has to read, so it takes room
           in the row like any toolbar content. -->
      {#if saveStatus === "error"}
        <div class="save-status error" role="alert">
          <span>{t("editor.novelEditor.saveError")}</span>
        </div>
      {/if}
      <!-- The slot is always mounted and asks the row for no width: it fills whatever
           space the buttons leave and the transient "Saving…" indicator is drawn over
           it, so a save never changes how the toolbar wraps. -->
      <div class="save-slot" data-testid="save-slot">
        {#if saveStatus === "saving"}
          <div class="save-status saving" data-testid="save-indicator" role="status">
            <Loader2 class="w-4 h-4 animate-spin" aria-hidden="true" />
            <span class="save-label">{t("editor.novelEditor.saving")}</span>
          </div>
        {/if}
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
    background: var(--color-surface-sunken);
    overflow: hidden;
  }

  .novel-editor.readonly {
    cursor: default;
  }

  /* The toolbar's trailing area takes no room of its own: no padding, and a negative
     start margin that cancels the toolbar gap before it. It can then always share the
     last row with the buttons, never opening a row of its own, and grows into whatever
     that row has left. */
  .novel-editor :global(.prose-toolbar .extra) {
    flex: 1 1 0;
    justify-content: flex-end;
    margin-left: calc(-1 * var(--space-2xs));
    padding-inline: 0;
  }

  /* No width of its own either (basis 0, min-width 0, absolutely positioned content),
     so a save never changes how the toolbar wraps. */
  .save-slot {
    position: relative;
    flex: 1 1 0;
    min-width: 0;
    align-self: stretch;
    container-type: inline-size;
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
    font-size: var(--text-small);
  }

  /* Drawn over the slot, one line tall, a toolbar gap clear of the last button. When
     the row has no room for the label it wraps onto a clipped second line, leaving only
     the spinner; the label stays in the status for assistive tech. */
  .save-status.saving {
    position: absolute;
    inset: 0;
    height: calc(var(--text-small) * 1.5);
    margin-block: auto;
    padding-left: var(--space-2xs);
    flex-wrap: wrap;
    justify-content: flex-end;
    overflow: hidden;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  /* Every item is a full line tall, so a wrapped label starts exactly at the clip edge
     and the spinner stays centred in the line it keeps. */
  .save-status.saving > :global(*) {
    height: 100%;
  }

  /* Too little room for even the spinner (1rem) after its gap: draw nothing rather than
     a clipped icon, but keep the status in the accessibility tree. */
  @container (width < 1.5rem) {
    .save-status.saving {
      clip-path: inset(50%);
    }
  }

  .save-label {
    white-space: nowrap;
  }

  /* In flow, so it starts where the trailing area's padding used to put it, on the
     button row or at the start of a row of its own. */
  .save-status.error {
    padding-inline: calc(2 * var(--space-2xs)) var(--space-2xs);
    color: var(--color-error);
  }

  /* Pages container - scrollable area */
  .novel-pages-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-m) var(--space-m) var(--space-l);
    background: transparent;
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

  /* Beat mode: a shorter sheet so a short beat doesn't open onto blank paper. */
  .beat-mode .novel-page {
    min-height: 12rem;
  }
  .beat-mode .editor-wrapper,
  .beat-mode :global(.novel-editor-content) {
    min-height: 8rem;
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

  /* Screenplay page: monospace, a blank line between elements, and the
     standard element indents on a 6in text column (cue 2.2in, dialogue 1in
     wide 3.5in, parenthetical 1.6in, transitions flush right). */
  .screenplay :global(.novel-editor-content) {
    font-family: var(--font-mono);
    font-size: var(--text-ui);
    line-height: var(--leading);
    white-space: pre-wrap;
  }
  .screenplay :global(.novel-editor-content p),
  .screenplay :global(.novel-editor-content p:first-child) {
    text-indent: 0;
    margin: 0 0 1lh;
  }
  .screenplay :global(.novel-editor-content p.sp-heading) {
    text-transform: uppercase;
  }
  .screenplay :global(.novel-editor-content p.sp-cue) {
    margin: 0 0 0 37%;
  }
  .screenplay :global(.novel-editor-content p.sp-parenthetical) {
    margin: 0 0 0 27%;
    max-width: 33%;
  }
  .screenplay :global(.novel-editor-content p.sp-dialogue) {
    margin: 0 17% 1lh 17%;
  }
  .screenplay :global(.novel-editor-content p.sp-dialogue:has(+ p.sp-parenthetical)) {
    margin-bottom: 0;
  }
  .screenplay :global(.novel-editor-content p.sp-transition) {
    text-align: right;
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
