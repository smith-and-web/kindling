<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Underline from '@tiptap/extension-underline';
  import TextAlign from '@tiptap/extension-text-align';
  import ProseToolbar from './ProseToolbar.svelte';
  import { createKeyboardFormatting } from './keyboardFormatting';
  import { defaultBindings, shortcutFromEvent, formatShortcut } from './keyboardShortcuts';
  import type { ShortcutAdapter, EditorAttachment, SaveStatus } from './types';

  let { content, label='Scene prose', placeholder='Write your prose…', readonly=false,
    saveStatus='idle', onUpdate, onEditorReady, onAttach, shortcuts,
    projectId, sceneId, beatId=null }: {
    content: string; label?: string; placeholder?: string; readonly?: boolean;
    saveStatus?: SaveStatus; onUpdate?: (html:string)=>void;
    onEditorReady?: (editor:Editor)=>void;
    onAttach?: (attachment:EditorAttachment)=>(()=>void)|void;
    shortcuts?: ShortcutAdapter; projectId?:string; sceneId?:string; beatId?:string|null;
  } = $props();
  const uid=$props.id();
  let element:HTMLDivElement;
  let scroller:HTMLDivElement;
  let exitControl:HTMLButtonElement;
  let editor:Editor|null=$state.raw(null);
  let revision=$state(0), wordCount=$state(0), empty=$state(true), ready=$state(false);
  let lastExternal='', lastEmitted='';
  const defaults:ShortcutAdapter={
    match(event){const key=shortcutFromEvent(event);return key ? Object.keys(defaultBindings).find(id=>defaultBindings[id]===key)??null:null;},
    label(id){return formatShortcut(defaultBindings[id]||'');}
  };
  const activeShortcuts=$derived(shortcuts??defaults);
  function refresh(instance:Editor){
    untrack(()=>{revision++;wordCount=instance.getText().trim().split(/\s+/).filter(Boolean).length;empty=instance.isEmpty;});
  }

  onMount(()=>{
    const instance=new Editor({
      element,
      extensions:[
        createKeyboardFormatting(()=>activeShortcuts),
        // StarterKit 3 includes underline. Register it once, matching the app schema.
        StarterKit.configure({heading:false,bulletList:false,orderedList:false,listItem:false,
          codeBlock:false,horizontalRule:false,underline:false,link:{openOnClick:false}}),
        Underline, TextAlign.configure({types:['paragraph']})
      ],
      content:content||'', editable:!readonly,
      editorProps:{
        attributes:{class:'kp-content',role:'textbox','aria-multiline':'true','aria-label':label,
          'aria-describedby':`${uid}-help`,'aria-readonly':String(readonly),spellcheck:'true'},
        handleKeyDown:(_view,event)=>{
          if(event.key==='Escape'){event.preventDefault();exitControl.focus();return true;}
          if(event.key==='Tab'&&!event.shiftKey&&!event.ctrlKey&&!event.metaKey&&!event.altKey&&instance.isEditable){
            event.preventDefault();instance.chain().insertContent('\t').run();return true;
          }
          return false;
        }
      },
      onTransaction:({editor:current})=>refresh(current),
      onUpdate:({editor:current})=>{
        const html=current.getHTML();
        if(html!==lastEmitted){lastEmitted=html;onUpdate?.(html);}
      }
    });
    editor=instance;lastExternal=content||'';lastEmitted=instance.getHTML();
    refresh(instance);ready=true;onEditorReady?.(instance);
    return ()=>{ready=false;instance.destroy();};
  });

  $effect(()=>{
    const html=content||''; const instance=editor;
    if(!instance||instance.isDestroyed)return;
    untrack(()=>{
      if(html!==lastExternal&&html!==lastEmitted&&html!==instance.getHTML()){
        instance.commands.setContent(html,{emitUpdate:false});
        lastEmitted=instance.getHTML();refresh(instance);
      }
      lastExternal=html;
    });
  });
  $effect(()=>{
    const instance=editor;
    if(instance&&!instance.isDestroyed){instance.setEditable(!readonly,false);
      instance.view.dom.setAttribute('aria-readonly',String(readonly));
      instance.view.dom.setAttribute('aria-label',label);}
  });
  $effect(()=>{
    const instance=editor, attach=onAttach, project=projectId, scene=sceneId, beat=beatId;
    if(instance&&attach) return untrack(()=>attach({editor:instance,scroller,projectId:project,sceneId:scene,beatId:beat}));
  });
  export function getSplitBeforeParagraph():number|null {
    if(!editor||editor.isDestroyed)return null;
    const position=editor.state.doc.resolve(editor.state.selection.from);
    return position.depth<1?0:position.index(0);
  }
</script>

<div class="kp-editor" class:kp-readonly={readonly}>
  <ProseToolbar {editor} {revision} {readonly} shortcuts={activeShortcuts}>
    {#if saveStatus==='saving'}<span role="status">Saving…</span>
    {:else if saveStatus==='error'}<span role="alert">Error saving. Try saving again.</span>{/if}
    <span class="kp-count">{wordCount} {wordCount===1?'word':'words'}</span>
    {#if readonly}<span>Read only</span>{/if}
  </ProseToolbar>
  <div class="kp-pages" bind:this={scroller}>
    <div class="kp-sheet" class:kp-empty={empty} data-placeholder={placeholder}>
      <div class="kp-editor-mount" bind:this={element} aria-busy={!ready}></div>
    </div>
  </div>
  <div class="kp-help od-row">
    <p id={`${uid}-help`} class="od-fill">{readonly?'Reading view. Editing is locked.':'Write on the page. Select text to format. Tab indents; Escape leaves the editor.'}</p>
    <button class="kp-exit od-fixed" bind:this={exitControl} type="button" onclick={()=>exitControl.focus()}>Leave editor</button>
  </div>
</div>

<style>
  .kp-editor{--kp-gutter:24px;--kp-page-pad:var(--space-xl);--kp-page-min:40rem;--kp-line-indent:1.5em;display:flex;flex-direction:column;min-width:0;color:var(--color-text);background:var(--color-surface)}
  .kp-pages{display:flex;flex-direction:column;align-items:center;min-width:0;max-height:56rem;overflow-y:auto;overscroll-behavior:contain;padding:var(--kp-gutter);background:var(--color-surface-sunken)}
  .kp-sheet{box-sizing:border-box;position:relative;isolation:isolate;flex:none;width:100%;max-width:calc(var(--measure) + 2 * var(--kp-page-pad));min-height:var(--kp-page-min);padding:var(--kp-page-pad);background:var(--color-prose-bg);color:var(--color-prose-text);border-radius:var(--radius-xs);box-shadow:var(--shadow-prose)}
  .kp-sheet::before{content:'';position:absolute;inset:0;pointer-events:none;z-index:-1;background-image:var(--grain-tile);background-size:160px 160px;opacity:var(--grain-strength);border-radius:inherit}
  .kp-editor-mount{min-width:0;width:100%;max-width:var(--measure);margin:auto}
  .kp-editor-mount :global(.kp-content){min-height:calc(var(--kp-page-min) - 2 * var(--kp-page-pad));outline:none;white-space:pre-wrap;overflow-wrap:anywhere;tab-size:4;cursor:text;font:var(--text-body)/var(--leading-relaxed) var(--font-body);color:var(--color-prose-text)}
  .kp-editor-mount :global(.kp-content:focus-visible){outline:2px solid var(--color-prose-blockquote-border);outline-offset:8px}
  .kp-editor-mount :global(.kp-content p){margin:0!important;font:inherit;color:inherit;text-indent:var(--kp-line-indent);max-width:none}
  .kp-editor-mount :global(.kp-content p:first-child){text-indent:0}
  .kp-empty .kp-editor-mount :global(.kp-content p:first-child::before){content:attr(data-placeholder);color:var(--color-prose-placeholder);pointer-events:none}
  .kp-sheet.kp-empty::after{content:attr(data-placeholder);position:absolute;top:var(--kp-page-pad);left:var(--kp-page-pad);max-width:calc(100% - 2 * var(--kp-page-pad));font:italic var(--text-body)/var(--leading-relaxed) var(--font-body);color:var(--color-prose-placeholder);pointer-events:none}
  .kp-editor-mount :global(.kp-content blockquote){margin:1em 0;padding:var(--space-s);background:var(--color-prose-callout-bg);border-left:4px solid var(--color-prose-blockquote-border);border-radius:0 var(--radius-m) var(--radius-m) 0;font-style:italic;color:var(--color-prose-blockquote-text)}
  .kp-editor-mount :global(.kp-content code){font:var(--text-ui) var(--font-mono);background:var(--color-prose-code-bg);padding:.125em .25em;border-radius:var(--radius-xs)}
  .kp-editor-mount :global(.kp-content strong){font-weight:600}
  .kp-editor-mount :global(.kp-content em){font-style:italic}
  .kp-editor-mount :global(.kp-content u){text-decoration:underline}
  .kp-editor-mount :global(.kp-content a){color:var(--color-prose-blockquote-border);text-decoration:underline}
  .kp-count{white-space:nowrap;font-variant-numeric:tabular-nums}
  .kp-help{padding:8px 16px;gap:16px;align-items:center;background:var(--color-surface)}
  .kp-help p{font:14px/1.5 var(--font-ui);margin:0!important;overflow-wrap:anywhere}
  .kp-exit{min-height:44px;padding:8px;color:var(--color-accent-text);border:0;background:transparent;font:14px/1.5 var(--font-ui);text-decoration:underline;cursor:pointer}
  .kp-exit:focus-visible{outline:2px solid var(--color-accent-text);outline-offset:2px}
  @media(hover:hover){.kp-exit:hover{text-decoration-thickness:2px}}
  @media(max-width:1023px){.kp-editor{--kp-page-pad:32px}}
  @media(max-width:767px){.kp-editor{--kp-gutter:8px;--kp-page-pad:24px}.kp-help{flex-wrap:wrap}}
  @media(max-width:374px){.kp-editor{--kp-page-pad:16px}}
  @media(prefers-reduced-motion:reduce){.kp-editor{scroll-behavior:auto}}
</style>
