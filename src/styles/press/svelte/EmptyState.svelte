<script lang="ts">
import type {Snippet} from 'svelte'; import Icon from './Icon.svelte';
let {title,description,icon='file-text',children}:{title:string;description:string;icon?:string;children?:Snippet}=$props();
</script>
<div class="ka-empty od-stack"><Icon name={icon} size={28}/><h4>{title}</h4><p>{description}</p>{@render children?.()}</div>
