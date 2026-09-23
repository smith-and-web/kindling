<script lang="ts">
import type {Snippet} from 'svelte'; import IconButton from './IconButton.svelte';
let {open=$bindable(false),title,subtitle='',wide=false,children,footer,onClose}:{open?:boolean;title:string;subtitle?:string;wide?:boolean;children?:Snippet;footer?:Snippet;onClose?:()=>void}=$props();
const id=$props.id(); let el:HTMLDialogElement;
$effect(()=>{if(!el)return;if(open&&!el.open)el.showModal();else if(!open&&el.open)el.close();});
</script>
<dialog bind:this={el} class="ka-dialog press-app" class:ka-dialog-wide={wide} aria-labelledby={id} onclose={()=>{open=false;onClose?.();}} oncancel={()=>{open=false;}}>
<header class="ka-dialog-header ka-between"><div class="od-field"><h3 {id}>{title}</h3>{#if subtitle}<p class="ka-help">{subtitle}</p>{/if}</div><IconButton label={`Close ${title}`} icon="x" onclick={()=>open=false}/></header>
<div class="ka-dialog-body">{@render children?.()}</div>{#if footer}<footer class="ka-dialog-footer">{@render footer()}</footer>{/if}
</dialog>
