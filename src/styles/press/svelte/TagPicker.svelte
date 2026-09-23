<script lang="ts">
import type {Option} from './types'; import Icon from './Icon.svelte'; import Field from './Field.svelte'; import Checkbox from './Checkbox.svelte';
let {label='Tags',options,value=$bindable([]),onChange}:{label?:string;options:Option[];value?:string[];onChange?:(value:string[])=>void}=$props();
let query=$state('');let open=$state(false);let status=$state('');
const filtered=$derived(options.filter(o=>o.label.toLowerCase().includes(query.toLowerCase())));
function toggle(id:string){value=value.includes(id)?value.filter(x=>x!==id):[...value,id];onChange?.(value);status=`${value.length} tags selected.`;}
</script>
<div class="ka-tag-picker od-stack"><p class="ka-label">{label}</p><div class="ka-row">{#each options.filter(o=>value.includes(o.value)) as o}<button class="ka-tag" type="button" aria-label={`Remove ${o.label}`} onclick={()=>toggle(o.value)}>{o.label}<Icon name="x" size={16}/></button>{/each}</div><details class="ka-disclosure" bind:open><summary><Icon name="plus" size={16}/> Choose tags</summary><div class="od-stack"><Field label="Filter tags" type="search" bind:value={query}/>{#each filtered as o}<Checkbox label={o.label} checked={value.includes(o.value)} disabled={o.disabled} onChange={()=>toggle(o.value)}/>{:else}<p class="ka-help">No matching tags. Try a different name.</p>{/each}</div></details><p class="ka-sr" role="status">{status}</p></div>
