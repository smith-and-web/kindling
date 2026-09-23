<script lang="ts">
import Dialog from './Dialog.svelte'; import type {Command} from './types';
let {open=$bindable(false),commands,onSelect,onClose}:{open?:boolean;commands:Command[];onSelect?:(id:string)=>void;onClose?:()=>void}=$props();
let query=$state('');let selected=$state(0);const id=$props.id();
const filtered=$derived(commands.filter(c=>!c.disabled&&`${c.label} ${c.keywords?.join(' ')||''}`.toLowerCase().includes(query.toLowerCase())));
const current=$derived(Math.min(selected,Math.max(0,filtered.length-1)));
function run(i:number){const c=filtered[i];if(c){open=false;onSelect?.(c.id);}}
function keys(e:KeyboardEvent){if(e.key==='ArrowDown'||e.key==='ArrowUp'){e.preventDefault();selected=filtered.length?(current+(e.key==='ArrowDown'?1:-1)+filtered.length)%filtered.length:0;}else if(e.key==='Enter'){e.preventDefault();run(current);}}
</script>
<Dialog bind:open title="Command palette" onClose={()=>{query='';selected=0;onClose?.();}}><div class="ka-field od-field"><label for={id}>Find a command</label><input {id} type="search" bind:value={query} role="combobox" aria-expanded="true" aria-autocomplete="list" aria-controls={`${id}-list`} aria-activedescendant={filtered.length?`${id}-${current}`:undefined} oninput={()=>selected=0} onkeydown={keys}/></div><div id={`${id}-list`} role="listbox" aria-label="Commands" class="ka-command-list">{#each filtered as c,i}<div id={`${id}-${i}`} role="option" aria-selected={i===current} class:ka-command-selected={i===current}><button type="button" tabindex="-1" onclick={()=>run(i)}><span class="od-field"><span>{c.label}</span>{#if c.group}<small>{c.group}</small>{/if}</span>{#if c.shortcut}<kbd>{c.shortcut}</kbd>{/if}</button></div>{:else}<p class="ka-help">No matching commands. Try “scene” or clear the search.</p>{/each}</div><p class="ka-help">↑ ↓ to move · Enter to run · Escape to close</p></Dialog>
