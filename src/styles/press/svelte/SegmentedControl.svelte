<script lang="ts">
import type { Option } from './types';
let {label,options,value=$bindable(''),onChange}: {label:string;options:Option[];value?:string;onChange?:(value:string)=>void}=$props();
const id=$props.id();
</script>
<fieldset class="ka-segments"><legend>{label}</legend><div class="ka-segment-track">{#each options as o}<label class:ka-selected={value===o.value} class="ka-segment"><input type="radio" name={id} value={o.value} bind:group={value} disabled={o.disabled} onchange={()=>onChange?.(value)}/><span>{o.label}</span></label>{/each}</div></fieldset>
