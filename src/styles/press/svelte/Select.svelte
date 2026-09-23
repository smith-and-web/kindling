<script lang="ts">
import type { Option } from './types';
let {label,value=$bindable(''),options,helper='',disabled=false,onChange}: {label:string;value?:string;options:Option[];helper?:string;disabled?:boolean;onChange?:(value:string)=>void}=$props();
const id=$props.id();
</script>
<div class="ka-field od-field"><label for={id}>{label}</label><select {id} bind:value {disabled} onchange={()=>onChange?.(value)} aria-describedby={helper ? `${id}-help` : undefined}>{#each options as option (option.value)}<option value={option.value} disabled={option.disabled}>{option.label}</option>{/each}</select>{#if helper}<p id={`${id}-help`} class="ka-help">{helper}</p>{/if}</div>
