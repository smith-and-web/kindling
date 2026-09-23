<script lang="ts">
import type {Snippet} from 'svelte';
let {title,subtitle='',children,actions}:{title:string;subtitle?:string;children?:Snippet;actions?:Snippet}=$props();
const id=$props.id();
</script>
<section class="ka-panel" aria-labelledby={id}><header class="ka-between"><div class="od-field"><h4 {id}>{title}</h4>{#if subtitle}<p class="ka-help">{subtitle}</p>{/if}</div>{#if actions}<div class="ka-row">{@render actions()}</div>{/if}</header><div class="ka-panel-body">{@render children?.()}</div></section>
