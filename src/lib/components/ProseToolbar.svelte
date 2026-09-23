<script lang="ts">
  import { shortcuts } from "../stores/shortcuts.svelte";
  import type { Snippet } from "svelte";
  import type { Editor } from "@tiptap/core";
  import {
    Bold,
    Italic,
    Underline,
    Code,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    Quote,
    IndentIncrease,
    Undo2,
    Redo2,
  } from "lucide-svelte";
  let {
    editor,
    revision = 0,
    readonly = false,
    children,
  }: {
    editor?: Editor | null;
    revision?: number;
    readonly?: boolean;
    children?: Snippet;
  } = $props();
  const active = $derived.by(() => {
    void revision;
    return {
      bold: editor?.isActive("bold"),
      italic: editor?.isActive("italic"),
      underline: editor?.isActive("underline"),
      code: editor?.isActive("code"),
      blockquote: editor?.isActive("blockquote"),
      left: editor?.isActive({ textAlign: "left" }),
      center: editor?.isActive({ textAlign: "center" }),
      right: editor?.isActive({ textAlign: "right" }),
      justify: editor?.isActive({ textAlign: "justify" }),
    };
  });
  // Undo/redo reflect whether history is available; unknown means available.
  const history = $derived.by(() => {
    void revision;
    const can = editor?.can?.();
    return { undo: can?.undo?.() ?? true, redo: can?.redo?.() ?? true };
  });
</script>

<div class="prose-toolbar" role="toolbar" aria-label="Text formatting">
  {#if !readonly}
    <div class="group">
      <button
        title={`Bold ${shortcuts.label("bold")}`.trim()}
        aria-label="Bold"
        aria-pressed={!!active.bold}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleBold().run()}><Bold size={20} /></button
      >
      <button
        title={`Italic ${shortcuts.label("italic")}`.trim()}
        aria-label="Italic"
        aria-pressed={!!active.italic}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleItalic().run()}><Italic size={20} /></button
      >
      <button
        title={`Underline ${shortcuts.label("underline")}`.trim()}
        aria-label="Underline"
        aria-pressed={!!active.underline}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleUnderline().run()}
        ><Underline size={20} /></button
      >
      <button
        title={`Monospace ${shortcuts.label("code")}`.trim()}
        aria-label="Monospace"
        aria-pressed={!!active.code}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleCode().run()}><Code size={20} /></button
      >
    </div>
    <div class="group separated">
      {#each [{ id: "left", icon: AlignLeft }, { id: "center", icon: AlignCenter }, { id: "right", icon: AlignRight }, { id: "justify", icon: AlignJustify }] as alignment}
        <button
          title={`Align ${alignment.id} ${shortcuts.label(`align_${alignment.id}`)}`.trim()}
          aria-label={`Align ${alignment.id}`}
          aria-pressed={!!active[alignment.id as "left" | "center" | "right" | "justify"]}
          onmousedown={(e) => e.preventDefault()}
          onclick={() => editor?.chain().focus().setTextAlign(alignment.id).run()}
          ><alignment.icon size={20} /></button
        >
      {/each}
      <button
        title={`Blockquote ${shortcuts.label("blockquote")}`.trim()}
        aria-label="Blockquote"
        aria-pressed={!!active.blockquote}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleBlockquote().run()}><Quote size={20} /></button
      >
      <button
        title="Indent"
        aria-label="Indent"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().insertContent("\t").run()}
        ><IndentIncrease size={20} /></button
      >
    </div>
    <div class="group separated">
      <button
        title="Undo"
        aria-label="Undo"
        disabled={!history.undo}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().undo().run()}><Undo2 size={20} /></button
      >
      <button
        title="Redo"
        aria-label="Redo"
        disabled={!history.redo}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().redo().run()}><Redo2 size={20} /></button
      >
    </div>
  {/if}
  <div class="extra">{@render children?.()}</div>
</div>

<style>
  /* Press ProseToolbar: 44px targets, hairline-separated groups. */
  .prose-toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    padding: var(--space-2xs);
    background: var(--color-surface-sunken);
    border-bottom: 1px solid var(--color-border);
    font: var(--text-small) / 1.5 var(--font-ui);
    color: var(--color-text);
    /* A divider sits in the gap before its group. When a group wraps to the
       start of a row, that divider falls in the clipped padding strip. */
    clip-path: inset(0 0 0 calc(var(--space-2xs) - 1px));
  }
  .group {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    max-width: 100%;
  }
  .separated {
    position: relative;
  }
  .separated::before {
    content: "";
    position: absolute;
    top: var(--space-2xs);
    bottom: var(--space-2xs);
    left: calc(-1 * var(--space-2xs) / 2 - 0.5px);
    width: 1px;
    background: var(--color-border);
  }
  /* Rings sit inside the button so the toolbar's clip never cuts them. */
  button:focus-visible {
    outline-offset: -2px;
  }
  button {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: var(--control-target);
    height: var(--control-target);
    padding: var(--space-2xs);
    border: 1px solid transparent;
    border-radius: var(--radius-s);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button[aria-pressed="true"] {
    background: var(--color-surface);
    color: var(--color-accent-text);
    border-color: var(--color-accent-text);
  }
  button:focus-visible {
    outline: 2px solid var(--color-accent-text);
    outline-offset: 2px;
  }
  button:disabled {
    background: transparent !important;
    border-color: transparent !important;
    color: var(--color-disabled-text) !important;
    cursor: default;
  }
  @media (hover: hover) {
    button:not(:disabled):hover {
      background: var(--color-surface);
      color: var(--color-accent-text);
    }
  }
  .extra {
    display: flex;
    align-items: center;
    gap: var(--space-s);
    margin-left: auto;
    padding: 0 var(--space-2xs);
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }
</style>
