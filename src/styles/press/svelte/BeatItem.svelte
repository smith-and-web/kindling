<script lang="ts">
import type {Snippet} from 'svelte'; import Icon from './Icon.svelte';
let {number,title,open=$bindable(false),children}:{number:number;title:string;open?:boolean;children?:Snippet}=$props();
</script>
<details class="ka-beat" bind:open><summary><Icon name={open?'chevron-down':'chevron-right'}/><span class="ka-beat-number">{number}</span><span class="od-fill">{title}</span></summary><div class="ka-beat-body">{@render children?.()}</div></details>
