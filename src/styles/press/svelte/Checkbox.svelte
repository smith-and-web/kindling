<script lang="ts">
let { label, checked = $bindable(false), disabled=false, helper='', onChange }: {label:string;checked?:boolean;disabled?:boolean;helper?:string;onChange?:(value:boolean)=>void}=$props();
const id=$props.id();
</script>
<div class="ka-check od-row"><input {id} type="checkbox" bind:checked {disabled} aria-describedby={helper ? `${id}-help` : undefined} onchange={()=>onChange?.(checked)}/><div class="od-field od-fill"><label for={id}>{label}</label>{#if helper}<p id={`${id}-help`} class="ka-help">{helper}</p>{/if}</div></div>
