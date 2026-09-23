<script lang="ts">
import type { Snippet } from 'svelte';
import type { Option } from './types';
let {label,items,value=$bindable(''),onChange,children}: {label:string;items:Option[];value?:string;onChange?:(value:string)=>void;children?:Snippet<[string]>}=$props();
const id=$props.id();
const active=$derived(items.some(x=>x.value===value && !x.disabled) ? value : items.find(x=>!x.disabled)?.value || '');
function choose(v:string) {value=v;onChange?.(v);}
function keys(e:KeyboardEvent) {
 const enabled=items.filter(x=>!x.disabled); let i=enabled.findIndex(x=>x.value===active);
 if(e.key==='ArrowRight') i=(i+1)%enabled.length; else if(e.key==='ArrowLeft') i=(i-1+enabled.length)%enabled.length; else if(e.key==='Home') i=0; else if(e.key==='End') i=enabled.length-1; else return;
 if(!enabled.length)return; e.preventDefault(); choose(enabled[i].value); document.getElementById(`${id}-${items.findIndex(x=>x.value===enabled[i].value)}`)?.focus();
}
</script>
<div class="ka-tabs"><div class="ka-tablist" role="tablist" aria-label={label}>{#each items as item,index}<button id={`${id}-${index}`} type="button" role="tab" disabled={item.disabled} aria-selected={active===item.value} aria-controls={`${id}-panel`} tabindex={active===item.value?0:-1} onclick={()=>choose(item.value)} onkeydown={keys}>{item.label}</button>{/each}</div><div id={`${id}-panel`} class="ka-tabpanel" role="tabpanel" aria-labelledby={`${id}-${items.findIndex(x=>x.value===active)}`} tabindex="0">{@render children?.(active)}</div></div>
