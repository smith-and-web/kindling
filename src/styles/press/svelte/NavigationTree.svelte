<script lang="ts">
import type {TreeNode} from './types'; import Icon from './Icon.svelte';
let {label,nodes,selected=$bindable(''),onSelect}:{label:string;nodes:TreeNode[];selected?:string;onSelect?:(id:string)=>void}=$props();
function select(id:string){selected=id;onSelect?.(id);}
</script>
{#snippet branch(items:TreeNode[])}
<ul class="ka-tree-list">{#each items as item (item.id)}<li>{#if item.children?.length}<details open={item.expanded ?? true}><summary><Icon name="chevron-down"/><span class="od-fill">{item.label}</span>{#if item.meta}<small>{item.meta}</small>{/if}</summary>{@render branch(item.children)}</details>{:else}<button type="button" class:ka-tree-selected={selected===item.id} aria-current={selected===item.id?'page':undefined} disabled={item.disabled} onclick={()=>select(item.id)}><span class="od-field od-fill"><span>{item.label}</span>{#if item.description}<small>{item.description}</small>{/if}</span>{#if item.meta}<small class="od-nowrap">{item.meta}</small>{/if}</button>{/if}</li>{/each}</ul>
{/snippet}
<nav class="ka-tree" aria-label={label}>{@render branch(nodes)}</nav>
