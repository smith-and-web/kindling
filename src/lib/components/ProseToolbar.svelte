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
</script>

<div class="prose-toolbar" role="toolbar" aria-label="Text formatting">
  {#if !readonly}
    <div class="group">
      <button
        title={`Bold ${shortcuts.label("bold")}`.trim()}
        aria-label="Bold"
        aria-pressed={!!active.bold}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleBold().run()}><Bold size={16} /></button
      >
      <button
        title={`Italic ${shortcuts.label("italic")}`.trim()}
        aria-label="Italic"
        aria-pressed={!!active.italic}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleItalic().run()}><Italic size={16} /></button
      >
      <button
        title={`Underline ${shortcuts.label("underline")}`.trim()}
        aria-label="Underline"
        aria-pressed={!!active.underline}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleUnderline().run()}
        ><Underline size={16} /></button
      >
      <button
        title={`Monospace ${shortcuts.label("code")}`.trim()}
        aria-label="Monospace"
        aria-pressed={!!active.code}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleCode().run()}><Code size={16} /></button
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
          ><alignment.icon size={16} /></button
        >
      {/each}
      <button
        title={`Blockquote ${shortcuts.label("blockquote")}`.trim()}
        aria-label="Blockquote"
        aria-pressed={!!active.blockquote}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().toggleBlockquote().run()}><Quote size={16} /></button
      >
      <button
        title="Indent"
        aria-label="Indent"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().insertContent("\t").run()}
        ><IndentIncrease size={16} /></button
      >
    </div>
    <div class="group separated">
      <button
        title="Undo"
        aria-label="Undo"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().undo().run()}><Undo2 size={16} /></button
      >
      <button
        title="Redo"
        aria-label="Redo"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => editor?.chain().focus().redo().run()}><Redo2 size={16} /></button
      >
    </div>
  {/if}
  <div class="extra">{@render children?.()}</div>
</div>

<style>
  .prose-toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2xs);
    padding: var(--space-2xs) var(--space-xs);
    background: var(--color-surface-sunken);
    border-bottom: 1px solid var(--color-border);
    font-family: var(--font-ui);
    font-size: var(--text-small);
  }
  .group {
    display: flex;
    align-items: center;
    gap: var(--space-3xs);
  }
  .separated {
    border-left: 1px solid var(--color-border);
    padding-left: var(--space-2xs);
  }
  button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-2xs);
    border: 0;
    border-radius: var(--radius-s);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }
  button:hover {
    background: var(--color-surface);
    color: var(--color-text);
  }
  button[aria-pressed="true"] {
    background: var(--color-accent-wash);
    color: var(--color-accent-text);
  }
  .extra {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin-left: auto;
  }
</style>
