<script lang="ts">
  import type { Editor } from '@tiptap/core';
  import type { Snippet } from 'svelte';
  import type { ShortcutAdapter } from './types';
  import { editorIcons } from './icons';
  let { editor = null, revision = 0, readonly = false, shortcuts, children }: {
    editor?: Editor | null; revision?: number; readonly?: boolean;
    shortcuts?: ShortcutAdapter; children?: Snippet;
  } = $props();
  const groups = [
    [{id:'bold', label:'Bold',icon:'bold'}, {id:'italic',label:'Italic',icon:'italic'},
     {id:'underline',label:'Underline',icon:'underline'}, {id:'code',label:'Monospace',icon:'code'}],
    [{id:'align_left',label:'Align left',icon:'align-left'}, {id:'align_center',label:'Align center',icon:'align-center'},
     {id:'align_right',label:'Align right',icon:'align-right'}, {id:'align_justify',label:'Justify',icon:'align-justify'},
     {id:'blockquote',label:'Blockquote',icon:'quote'}, {id:'indent',label:'Indent',icon:'indent-increase'}],
    [{id:'undo',label:'Undo',icon:'undo-2'}, {id:'redo',label:'Redo',icon:'redo-2'}]
  ];
  const state = $derived.by(() => {
    void revision;
    if (!editor || editor.isDestroyed) return { active: {} as Record<string,boolean>, undo:false, redo:false };
    const active: Record<string,boolean> = {};
    for (const group of groups) for (const {id} of group) {
      active[id] = id.startsWith('align_') ? editor.isActive({textAlign:id.slice(6)}) : editor.isActive(id);
    }
    return { active, undo:editor.can().undo(), redo:editor.can().redo() };
  });
  function run(id: string) {
    if (!editor || !editor.isEditable) return;
    const chain = editor.chain().focus();
    if (id.startsWith('align_')) chain.setTextAlign(id.slice(6)).run();
    else if (id==='bold') chain.toggleBold().run();
    else if (id==='italic') chain.toggleItalic().run();
    else if (id==='underline') chain.toggleUnderline().run();
    else if (id==='code') chain.toggleCode().run();
    else if (id==='blockquote') chain.toggleBlockquote().run();
    else if (id==='indent') chain.insertContent('\t').run();
    else if (id==='undo') chain.undo().run();
    else if (id==='redo') chain.redo().run();
  }
</script>

<div class="kp-toolbar" role="toolbar" aria-label="Prose formatting">
  {#if !readonly}
    {#each groups as group}
      <div class="kp-tool-group">
        {#each group as tool}
          <button type="button" aria-label={tool.label}
            title={[tool.label, shortcuts?.label(tool.id)].filter(Boolean).join(' ')}
            aria-pressed={['undo','redo','indent'].includes(tool.id) ? undefined : !!state.active[tool.id]}
            disabled={!editor || (tool.id==='undo' && !state.undo) || (tool.id==='redo' && !state.redo)}
            onmousedown={(event)=>event.preventDefault()} onclick={()=>run(tool.id)}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              {#each editorIcons[tool.icon as keyof typeof editorIcons] as [tag, attrs]}
                <svelte:element this={tag as string} {...attrs as Record<string,string>}/>
              {/each}
            </svg>
          </button>
        {/each}
      </div>
    {/each}
  {/if}
  <div class="kp-tool-status">{@render children?.()}</div>
</div>

<style>
  .kp-toolbar{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:8px;background:var(--color-surface-sunken);border-bottom:1px solid var(--color-border);font:14px/1.5 var(--font-ui);color:var(--color-text)}
  .kp-tool-group{display:flex;flex-wrap:wrap;gap:8px;max-width:100%}
  .kp-tool-group+.kp-tool-group{padding-left:8px;border-left:1px solid var(--color-border)}
  button{display:flex;align-items:center;justify-content:center;flex:none;width:44px;height:44px;padding:8px;border:1px solid transparent;border-radius:var(--radius-s);background:transparent;color:inherit;cursor:pointer}
  button[aria-pressed=true]{background:var(--color-surface);color:var(--color-accent-text);border-color:var(--color-accent-text)}
  button:focus-visible{outline:2px solid var(--color-accent-text);outline-offset:2px}
  button:disabled{opacity:.45;cursor:default}
  .kp-tool-status{display:flex;flex-wrap:wrap;gap:16px;align-items:center;margin-left:auto;padding:8px}
  @media(hover:hover){button:not(:disabled):hover{background:var(--color-surface);color:var(--color-accent-text)}}
  @media(max-width:767px){.kp-tool-group+.kp-tool-group{padding-left:0;border-left:0}.kp-tool-status{margin-left:0}}
</style>
