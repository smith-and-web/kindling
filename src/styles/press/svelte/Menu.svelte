<script lang="ts">
import {tick} from 'svelte'; import type {MenuItem} from './types'; import Icon from './Icon.svelte';
let {label='More actions',items,onSelect}:{label?:string;items:MenuItem[];onSelect?:(id:string)=>void}=$props();
let open=$state(false);let root:HTMLDetailsElement;let trigger:HTMLElement;
async function opened(e:Event){open=(e.currentTarget as HTMLDetailsElement).open;if(open){await tick();root.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus();}}
function keys(e:KeyboardEvent){if(e.key==='Escape'){open=false;trigger.focus();return;}const buttons=[...root.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')];const i=buttons.indexOf(document.activeElement as HTMLButtonElement);let n=i;if(e.key==='ArrowDown')n=(i+1)%buttons.length;else if(e.key==='ArrowUp')n=(i-1+buttons.length)%buttons.length;else if(e.key==='Home')n=0;else if(e.key==='End')n=buttons.length-1;else return;e.preventDefault();buttons[n]?.focus();}
</script>
<svelte:window onclick={(e)=>{if(open&&root&&!root.contains(e.target as Node))open=false;}}/>
<details class="ka-menu" bind:this={root} bind:open ontoggle={opened}><summary bind:this={trigger} aria-haspopup="menu"><Icon name="ellipsis"/>{label}</summary><div class="ka-menu-list" role="menu" aria-label={label} onkeydown={keys} tabindex="-1">{#each items as item}<button role="menuitem" type="button" disabled={item.disabled} class:ka-error={item.danger} onclick={()=>{open=false;trigger.focus();onSelect?.(item.id);}}><span>{item.label}</span>{#if item.shortcut}<kbd>{item.shortcut}</kbd>{/if}</button>{/each}</div></details>
