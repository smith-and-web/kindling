<script lang="ts">
let {items}:{items:{label:string;value:string|number;unit?:string}[]}=$props();
</script>
<dl class="ka-stats">{#each items as item}<div class="od-stat"><dt>{item.label}</dt><dd><span class="od-nowrap">{typeof item.value==='number'?item.value.toLocaleString():item.value}{#if item.unit}<small> {item.unit}</small>{/if}</span></dd></div>{/each}</dl>
