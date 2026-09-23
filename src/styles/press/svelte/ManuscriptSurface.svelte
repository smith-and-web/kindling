<script lang="ts">
import type {Snippet} from 'svelte';
let {title,subtitle='',children}:{title:string;subtitle?:string;children?:Snippet}=$props();
</script>
<article class="ka-manuscript"><header><h3>{title}</h3>{#if subtitle}<p>{subtitle}</p>{/if}</header><div class="ka-manuscript-prose">{@render children?.()}</div></article>
